//! Stored QOS operator key for manifest signing.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Operator key stored in operator.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQosOperatorKey {
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Hex-encoded private key
    pub private_key: String,
}

impl StoredQosOperatorKey {
    /// Load an operator key from a registered `operator.json` path.
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        debug!(operator_key_path = %path.display(), "loading stored operator key");
        if !path.exists() {
            debug!(operator_key_path = %path.display(), "stored operator key not found");
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read operator key: {}", path.display()))?;

        let key: StoredQosOperatorKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse operator key: {}", path.display()))?;

        debug!(
            operator_key_path = %path.display(),
            has_public_key = !key.public_key.is_empty(),
            "loaded stored operator key"
        );

        Ok(Some(key))
    }

    /// Save an operator key to a registered `operator.json` path.
    pub async fn save(&self, path: &Path) -> Result<()> {
        debug!(operator_key_path = %path.display(), "saving stored operator key");

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

        debug!(operator_key_path = %path.display(), "saved stored operator key");

        Ok(())
    }
}
