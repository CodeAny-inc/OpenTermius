use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::host::AuthMethod;

/// A reusable identity (like Termius "Identity"). An identity bundles a
/// username + authentication method + optional SSH key, so the user can
/// create it once and apply it to many hosts.
///
/// When a host has `identity_id` set, the connection logic resolves the
/// identity at connect time to get the username, auth method, and key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: Uuid,
    pub label: String,
    pub username: String,
    /// Authentication method + key reference (same enum as Host).
    pub auth: AuthMethod,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub key_id: Option<Uuid>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Identity {
    pub fn new(label: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            label: label.into(),
            username: username.into(),
            auth: AuthMethod::Agent,
            key_id: None,
            tags: Vec::new(),
        }
    }
}
