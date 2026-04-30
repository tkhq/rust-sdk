//! Quorum key configuration file format for `tvc keys generate-quorum-key`.

use serde::{Deserialize, Serialize};

/// Maximum number of shares and minimum threshold. Limits come from
/// `qos_crypto::shamir::shares_generate` (see qos_crypto/src/shamir.rs).
pub const MAX_SHARES: u32 = 255;
pub const MIN_THRESHOLD: u32 = 2;

/// Quorum key configuration loaded from JSON file.
///
/// Numeric constraints (`shares <= 255`, `threshold >= 2`) are inherited
/// from `qos_crypto::shamir::shares_generate`'s documented limitations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumKeyConfig {
    pub shares: u32,
    pub threshold: u32,
    pub operator_public_keys: Vec<String>,
}

impl QuorumKeyConfig {
    /// Generate a default template config with placeholders.
    pub fn template(operator_public_key: Option<&str>) -> Self {
        Self {
            shares: 2,
            threshold: 2,
            operator_public_keys: vec![
                operator_public_key
                    .unwrap_or("<FILL_IN_OPERATOR_PUBLIC_KEY_1>")
                    .to_string(),
                "<FILL_IN_OPERATOR_PUBLIC_KEY_2>".to_string(),
            ],
        }
    }

    /// True if any field still contains a placeholder.
    pub fn has_placeholders(&self) -> bool {
        self.operator_public_keys
            .iter()
            .any(|k| k.starts_with("<FILL_IN"))
    }

    /// Validate numeric constraints. Called by `generate_quorum_key`
    /// before consuming the config.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.has_placeholders() {
            anyhow::bail!("config contains placeholder operator public keys");
        }
        if self.shares == 0 || self.shares > MAX_SHARES {
            anyhow::bail!(
                "shares must be between 1 and {MAX_SHARES}, got {}",
                self.shares
            );
        }
        if self.threshold < MIN_THRESHOLD {
            anyhow::bail!(
                "threshold must be >= {MIN_THRESHOLD}, got {}",
                self.threshold
            );
        }
        if self.threshold > self.shares {
            anyhow::bail!(
                "threshold ({}) cannot exceed shares ({})",
                self.threshold,
                self.shares
            );
        }

        if self.operator_public_keys.len() != self.shares as usize {
            anyhow::bail!(
                "operatorPublicKeys length ({}) must equal shares ({})",
                self.operator_public_keys.len(),
                self.shares
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_config() -> QuorumKeyConfig {
        QuorumKeyConfig {
            shares: 2,
            threshold: 2,
            operator_public_keys: vec!["operator-key-1".to_string(), "operator-key-2".to_string()],
        }
    }

    #[test]
    fn validate_rejects_placeholders() {
        let config = QuorumKeyConfig::template(None);

        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("placeholder"));
    }

    #[test]
    fn validate_rejects_operator_key_count_mismatch() {
        let mut config = valid_config();
        config.shares = 3;

        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("must equal shares"));
    }

    #[test]
    fn validate_rejects_threshold_and_share_bounds() {
        let mut zero_shares = valid_config();
        zero_shares.shares = 0;
        assert!(zero_shares
            .validate()
            .unwrap_err()
            .to_string()
            .contains("shares must be between"));

        let mut low_threshold = valid_config();
        low_threshold.threshold = 1;
        assert!(low_threshold
            .validate()
            .unwrap_err()
            .to_string()
            .contains("threshold must be"));

        let mut high_threshold = valid_config();
        high_threshold.threshold = 3;
        assert!(high_threshold
            .validate()
            .unwrap_err()
            .to_string()
            .contains("cannot exceed shares"));
    }
}
