use crate::host::{AuthMethod, Host};
use crate::known_hosts::KnownHosts;
use crate::vault::Vault;
use crate::{CoreError, Result};
use russh::client::{self, Config, Handle};
use russh::keys::key;
use russh_keys::decode_secret_key;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handler that verifies the server key against the known_hosts store (TOFU).
/// On first sight: record + accept. On match: accept. On mismatch: reject.
pub struct SshHandler {
    host: String,
    port: u16,
    known_hosts: Arc<Mutex<KnownHosts>>,
}

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = CoreError;

    async fn check_server_key(
        &mut self,
        server_public_key: &key::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        let mut kh = self.known_hosts.lock().await;
        let trusted = kh.verify(&self.host, self.port, server_public_key)?;
        if !trusted {
            tracing::warn!(
                "host key mismatch for {}:{} — rejecting",
                self.host,
                self.port
            );
        }
        Ok(trusted)
    }
}

/// Open an authenticated SSH session and return the handle.
/// The caller is responsible for opening a channel and starting the shell.
pub async fn connect(
    host: &Host,
    known_hosts: Arc<Mutex<KnownHosts>>,
    vault: Option<&Vault>,
    passphrase: Option<&str>,
    password: Option<&str>,
) -> Result<Handle<SshHandler>> {
    let config = Arc::new(Config::default());
    let handler = SshHandler {
        host: host.hostname.clone(),
        port: host.port,
        known_hosts,
    };

    let addr = format!("{}:{}", host.hostname, host.port);
    let mut session = client::connect(config, &addr, handler)
        .await
        .map_err(|e| CoreError::Ssh(format!("connect {addr}: {e}")))?;

    let auth_ok = match &host.auth {
        AuthMethod::Agent => {
            // Agent auth: try with the agent. For now, fall back to none.
            // Full agent support requires connecting to the SSH agent socket.
            session.authenticate_password(&host.username, "").await
        }
        AuthMethod::Password { .. } => {
            let pw = password.ok_or_else(|| {
                CoreError::InvalidInput("password required but not provided".into())
            })?;
            session.authenticate_password(&host.username, pw).await
        }
        AuthMethod::PublicKey => {
            let key_id = host.key_id.ok_or_else(|| {
                CoreError::InvalidInput("publickey auth but no key_id set".into())
            })?;
            let passphrase = passphrase.ok_or_else(|| {
                CoreError::InvalidInput("vault passphrase required for key auth".into())
            })?;
            let vault = vault.ok_or_else(|| {
                CoreError::InvalidInput("vault required for key auth".into())
            })?;
            let private_pem = vault.get_key(passphrase, &key_id.to_string())?;
            let pem_str = String::from_utf8(private_pem)
                .map_err(|e| CoreError::Key(format!("utf8: {e}")))?;
            let pair = decode_secret_key(&pem_str, None)
                .map_err(|e| CoreError::Key(format!("decode: {e}")))?;
            session
                .authenticate_publickey(&host.username, Arc::new(pair))
                .await
        }
    }
    .map_err(|e| CoreError::Ssh(format!("auth: {e}")))?;

    if !auth_ok {
        return Err(CoreError::Ssh("authentication rejected by server".into()));
    }
    Ok(session)
}
