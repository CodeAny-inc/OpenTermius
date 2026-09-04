use crate::keys::KeyMeta;
use crate::{CoreError, Result};
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zeroize::Zeroize;

/// Encrypted-at-rest vault for private key material.
///
/// Layout on disk (JSON):
///   { "salt": "<b64>", "ciphertext": "<b64>", "keys_meta": [...] }
///
/// The master key is derived from a user passphrase via Argon2id. The OS
/// keychain stores the passphrase (set by the UI layer), never the derived key.
/// Plaintext private keys only ever exist in memory and are zeroized on drop.
#[derive(Serialize, Deserialize)]
pub struct VaultFile {
    salt: String,
    ciphertext: String,
    pub keys_meta: Vec<KeyMeta>,
}

pub struct Vault {
    path: PathBuf,
    file: VaultFile,
}

#[derive(Serialize, Deserialize)]
struct VaultPayload {
    keys: Vec<(String, String)>, // (key_id, openssh private)
}

impl Vault {
    pub fn open(path: PathBuf) -> Result<Self> {
        let file = if path.exists() {
            let data = std::fs::read_to_string(&path)?;
            serde_json::from_str(&data)?
        } else {
            VaultFile {
                salt: String::new(),
                ciphertext: String::new(),
                keys_meta: Vec::new(),
            }
        };
        Ok(Self { path, file })
    }

    pub fn is_initialized(&self) -> bool {
        !self.file.ciphertext.is_empty()
    }

    pub fn keys_meta(&self) -> &[KeyMeta] {
        &self.file.keys_meta
    }

    pub fn initialize(&mut self, passphrase: &str) -> Result<()> {
        let mut salt = [0u8; 16];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut salt);
        let payload = VaultPayload { keys: Vec::new() };
        let plaintext = serde_json::to_vec(&payload)?;
        let ciphertext = seal(passphrase, &salt, &plaintext)?;
        self.file.salt = base64(salt);
        self.file.ciphertext = base64(ciphertext);
        self.file.keys_meta.clear();
        self.save()
    }

    pub fn add_key(
        &mut self,
        passphrase: &str,
        meta: KeyMeta,
        private_openssh: &str,
    ) -> Result<()> {
        let salt = unbase64(&self.file.salt)?;
        let mut pt = open(passphrase, &salt, &unbase64(&self.file.ciphertext)?)?;
        let mut payload: VaultPayload = serde_json::from_slice(&pt)?;
        payload
            .keys
            .push((meta.id.to_string(), private_openssh.to_string()));
        pt.zeroize();
        let new_pt = serde_json::to_vec(&payload)?;
        let ct = seal(passphrase, &salt, &new_pt)?;
        self.file.ciphertext = base64(ct);
        self.file.keys_meta.push(meta);
        self.save()
    }

    pub fn remove_key(&mut self, passphrase: &str, key_id: &str) -> Result<()> {
        let salt = unbase64(&self.file.salt)?;
        let mut pt = open(passphrase, &salt, &unbase64(&self.file.ciphertext)?)?;
        let mut payload: VaultPayload = serde_json::from_slice(&pt)?;
        payload.keys.retain(|(id, _)| id != key_id);
        pt.zeroize();
        let new_pt = serde_json::to_vec(&payload)?;
        let ct = seal(passphrase, &salt, &new_pt)?;
        self.file.ciphertext = base64(ct);
        self.file
            .keys_meta
            .retain(|m| m.id.to_string() != key_id);
        self.save()
    }

    /// Decrypt and return a single key's private material. Caller is responsible
    /// for the `PrivateKeyMaterial` (it zeroizes on drop).
    pub fn get_key(&self, passphrase: &str, key_id: &str) -> Result<Vec<u8>> {
        let salt = unbase64(&self.file.salt)?;
        let pt = open(passphrase, &salt, &unbase64(&self.file.ciphertext)?)?;
        let payload: VaultPayload = serde_json::from_slice(&pt)?;
        payload
            .keys
            .into_iter()
            .find(|(id, _)| id == key_id)
            .map(|(_, k)| k.into_bytes())
            .ok_or_else(|| CoreError::Vault(format!("key {key_id} not found")))
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(&self.file)?;
        std::fs::write(&self.path, data)?;
        Ok(())
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let params = Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|e| CoreError::Vault(format!("argon2 params: {e}")))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; 32];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| CoreError::Vault(format!("kdf: {e}")))?;
    Ok(out)
}

fn seal(passphrase: &str, salt: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::Vault(format!("aes init: {e}")))?;
    let mut nonce_bytes = [0u8; 12];
    use rand::RngCore;
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::Vault(format!("encrypt: {e}")))?;
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    Ok(out)
}

fn open(passphrase: &str, salt: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(CoreError::Vault("ciphertext too short".into()));
    }
    let (nonce_bytes, ct) = ciphertext.split_at(12);
    let key = derive_key(passphrase, salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::Vault(format!("aes init: {e}")))?;
    cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ct)
        .map_err(|_| CoreError::Vault("decrypt failed (wrong passphrase or corrupted)".into()))
}

fn base64(b: impl AsRef<[u8]>) -> String {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.encode(b.as_ref())
}

fn unbase64(s: &str) -> Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine};
    STANDARD.decode(s).map_err(|e| CoreError::Vault(format!("b64: {e}")))
}
