//! Stored passkey session for Turnkey authentication.

use super::{OrgConfig, default_passkey_session_path_for_org};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPasskeySession {
    /// Signed Turnkey session JWT returned by stamp_login.
    pub session: String,
}

impl StoredPasskeySession {
    pub async fn load(alias: &str, org_config: &OrgConfig) -> Result<Option<Self>> {
        let path = passkey_session_path(alias, org_config)?;
        debug!(passkey_session_path = %path.display(), "loading stored passkey session");
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read passkey session: {}", path.display()))?;
        let session = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse passkey session: {}", path.display()))?;
        Ok(Some(session))
    }

    pub async fn save(&self, alias: &str, org_config: &OrgConfig) -> Result<()> {
        let path = passkey_session_path(alias, org_config)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(self).context("failed to serialize passkey session")?;
        tokio::fs::write(&path, content)
            .await
            .with_context(|| format!("failed to write passkey session: {}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&path, permissions)
                .with_context(|| format!("failed to chmod passkey session: {}", path.display()))?;
        }

        Ok(())
    }
}

fn passkey_session_path(alias: &str, org_config: &OrgConfig) -> Result<PathBuf> {
    match &org_config.passkey_session_path {
        Some(path) => Ok(path.clone()),
        None => default_passkey_session_path_for_org(alias),
    }
}
