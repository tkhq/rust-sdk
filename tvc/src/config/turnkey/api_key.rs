//! Stored API key for Turnkey authentication.

use super::OrgConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// API key stored in api_key.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredApiKey {
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Hex-encoded private key
    pub private_key: String,
    /// The elliptic curve used for this key
    pub curve: KeyCurve,
}

/// Supported elliptic curves for API keys
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyCurve {
    P256,
    Secp256k1,
}

impl StoredApiKey {
    /// Load API key from the path specified in org config
    pub async fn load(org_config: &OrgConfig) -> Result<Option<Self>> {
        let path = &org_config.api_key_path;
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read API key: {}", path.display()))?;

        let key: StoredApiKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse API key: {}", path.display()))?;

        Ok(Some(key))
    }

    /// Save API key to the path specified in org config
    pub async fn save(&self, org_config: &OrgConfig) -> Result<()> {
        let path = &org_config.api_key_path;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self).context("failed to serialize API key")?;

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write API key: {}", path.display()))?;

        Ok(())
    }
}
