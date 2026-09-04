use crate::{CoreError, Result};
use russh_keys::key;
use russh_keys::{decode_secret_key, PublicKeyBase64};
use serde::{Deserialize, Serialize};
use ssh_key::PrivateKey;
use uuid::Uuid;
use zeroize::Zeroize;

/// Metadata about a stored key. The private material itself lives in the
/// encrypted vault, referenced by `id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMeta {
    pub id: Uuid,
    pub label: String,
    pub key_type: KeyType,
    pub fingerprint: String,
    pub public_key_base64: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KeyType {
    Ed25519,
    Rsa,
    Ecdsa,
}

/// In-memory holder for decrypted private key material. Zeroizes on drop.
#[derive(Debug)]
pub struct PrivateKeyMaterial {
    pub id: Uuid,
    pub openssh: Vec<u8>,
}

impl Drop for PrivateKeyMaterial {
    fn drop(&mut self) {
        self.openssh.zeroize();
    }
}

/// Parse an OpenSSH-formatted private key string, returning metadata + the
/// russh keypair. Does NOT persist anything.
pub fn parse_openssh_private(
    openssh: &str,
    passphrase: Option<&str>,
) -> Result<(KeyMeta, key::KeyPair)> {
    let pair = decode_secret_key(openssh, passphrase)
        .map_err(|e| CoreError::Key(format!("parse: {e}")))?;
    let public = pair
        .clone_public_key()
        .map_err(|e| CoreError::Key(format!("clone public: {e}")))?;
    let fingerprint = public.fingerprint();
    let public_key_base64 = public.public_key_base64();
    let key_type = match &pair {
        key::KeyPair::Ed25519 { .. } => KeyType::Ed25519,
        key::KeyPair::RSA { .. } => KeyType::Rsa,
        key::KeyPair::EC { .. } => KeyType::Ecdsa,
    };
    let meta = KeyMeta {
        id: Uuid::new_v4(),
        label: String::new(),
        key_type,
        fingerprint,
        public_key_base64,
    };
    Ok((meta, pair))
}

/// Generate a new Ed25519 keypair. Returns (private OpenSSH PEM, public base64).
pub fn generate_ed25519() -> Result<(String, String)> {
    let private = PrivateKey::random(&mut rand::rngs::OsRng, ssh_key::Algorithm::Ed25519)
        .map_err(|e| CoreError::Key(format!("generate: {e}")))?;
    let pem = private
        .to_openssh(ssh_key::LineEnding::LF)
        .map_err(|e| CoreError::Key(format!("serialize: {e}")))?;
    let public_b64 = private
        .public_key()
        .to_openssh()
        .map_err(|e| CoreError::Key(format!("public serialize: {e}")))?;
    // Extract just the base64 part from "ssh-ed25519 AAAA... comment"
    let public_b64 = public_b64
        .split_whitespace()
        .nth(1)
        .unwrap_or(&public_b64)
        .to_string();
    Ok((pem.to_string(), public_b64))
}
