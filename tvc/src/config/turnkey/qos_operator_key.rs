//! Stored QOS operator key for manifest signing.

use super::OrgConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Operator key stored in operator.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQosOperatorKey {
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Hex-encoded private key
    pub private_key: String,
}

impl StoredQosOperatorKey {
    /// Load operator key from the path specified in org config
    pub async fn load(org_config: &OrgConfig) -> Result<Option<Self>> {
        let path = &org_config.operator_key_path;
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read operator key: {}", path.display()))?;

        let key: StoredQosOperatorKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse operator key: {}", path.display()))?;

        Ok(Some(key))
    }

    /// Save operator key to the path specified in org config
    pub async fn save(&self, org_config: &OrgConfig) -> Result<()> {
        let path = &org_config.operator_key_path;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(self).context("failed to serialize operator key")?;

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write operator key: {}", path.display()))?;

        Ok(())
    }
}
