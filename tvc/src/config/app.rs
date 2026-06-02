//! App configuration file format for `tvc app create`.

use crate::prompts;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const MIN_SHARE_SET_THRESHOLD: u32 = 2;

/// App configuration loaded from JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub name: String,
    pub quorum_public_key: String,
    #[serde(default)]
    pub external_connectivity: Option<bool>,
    #[serde(default)]
    pub manifest_set_id: Option<String>,
    #[serde(default)]
    pub manifest_set_params: Option<OperatorSetParams>,
    #[serde(default)]
    pub share_set_id: Option<String>,
    #[serde(default)]
    pub share_set_params: Option<OperatorSetParams>,
    /// Whether this app permits debug-mode deployments. Must be set at app
    /// creation and cannot be changed after. Setting this true means the app's
    /// quorum key is considered permanently insecure.
    #[serde(default)]
    pub dangerous_enable_debug_mode_deployments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorSetParams {
    pub name: String,
    pub threshold: u32,
    #[serde(default)]
    pub new_operators: Vec<OperatorParams>,
    #[serde(default)]
    pub existing_operator_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorParams {
    pub name: String,
    pub public_key: String,
}

/// Known share set public keys. This is for known share keys
/// that encrypt known operator keys. Assume the secrets are well known
pub const KNOWN_SHARE_SET_KEYS: [(&str, &str); 2] = [
    ("1", "044af8b082b9ef41a238037811a188309d8c8b00b6d49c0574538d7746d7383739e67e1107f134bc102a48301b07e7c53280decbe9c16c9fc1f19b9832018e1485048139aa5de49d9505465bcf1a879954c51ba7b258b669f4e42697088cbbca54aeb888d61e65b2602ce92ae945a0160533acc94942511f8e5b1940ed89cc8f141f"),
    ("2", "04c1c4b4eb784505f167affae00e18b1521e7a0bfa3be46e6a6b43ba1f386afce48d964c885480cb197e3538fd30ebe38a07f76b6a286b37ba6d2abddbbd6c9c8304e492ca7bce95912a7b2565c8553e38cf3a4b1f858171900ed81888282db13d41e214dd6def2de2aacb1fcf92e3ae5a83e1b0ffa660fc59b9dd10e277cfd128dc"),
];

/// Well known Quorum Key. This is for applications that do not need secure quorum keys
pub const KNOWN_QUORUM_KEY: &str = "04451028fc9d42cef6d8f2a3ebe17d65783c470dbc6f04663d500c12009930cf9b209e733f6ac6103cc28f07ecde2dbb55095738b828d6b7a55caf4ddf9d67f2ae047827dcd2325b8d58694c2ea14e8f1e1f8a36c84438d291ff9b1b067debdb3e2ba3822984cde8bed4de2c237bd323526da4961d368bcc63cbd2d37d00e936683e";

impl AppConfig {
    /// Generate a default template config with placeholders.
    pub fn template(operator_public_key: Option<&str>) -> Self {
        Self {
            name: "<FILL_IN_APP_NAME>".to_string(),
            quorum_public_key: KNOWN_QUORUM_KEY.to_string(),
            external_connectivity: Some(false),
            manifest_set_id: None,
            manifest_set_params: Some(OperatorSetParams {
                name: "<FILL_IN_MANIFEST_SET_NAME>".to_string(),
                threshold: 1,
                new_operators: vec![OperatorParams {
                    name: "operator-1".to_string(),
                    public_key: operator_public_key
                        .unwrap_or("<FILL_IN_OPERATOR_PUBLIC_KEY>")
                        .to_string(),
                }],
                existing_operator_ids: vec![],
            }),
            share_set_id: None,
            share_set_params: None,
            dangerous_enable_debug_mode_deployments: false,
        }
    }

    /// Get the hardcoded share set params using known share set keys.
    pub fn share_set_params() -> OperatorSetParams {
        OperatorSetParams {
            name: "dev-known-share-set".to_string(),
            threshold: 2,
            new_operators: KNOWN_SHARE_SET_KEYS
                .iter()
                .map(|(name, key)| OperatorParams {
                    name: name.to_string(),
                    public_key: key.to_string(),
                })
                .collect(),
            existing_operator_ids: vec![],
        }
    }

    /// Walk the user through any placeholder fields and fill them in.
    /// Non-placeholder fields are preserved unchanged so partial edits work.
    ///
    /// `saved_operator_public_key` is offered as the default when prompting
    /// for a `<FILL_IN>` operator public key.
    pub fn fill_interactively(&mut self, saved_operator_public_key: Option<&str>) -> Result<()> {
        if self.name.starts_with("<FILL_IN") {
            self.name = prompts::required_text("App name", None)?;
        }
        if let Some(set_params) = self.manifest_set_params.as_mut() {
            if set_params.name.starts_with("<FILL_IN") {
                set_params.name = prompts::required_text("Manifest set name", None)?;
            }
            for op in set_params.new_operators.iter_mut() {
                if op.public_key.starts_with("<FILL_IN") {
                    let prompt = format!("Operator '{}' public key", op.name);
                    op.public_key = prompts::required_text(&prompt, saved_operator_public_key)?;
                }
            }
        }
        if let Some(set_params) = self.share_set_params.as_mut() {
            if set_params.name.starts_with("<FILL_IN") {
                set_params.name = prompts::required_text("Share set name", None)?;
            }
            for op in set_params.new_operators.iter_mut() {
                if op.public_key.starts_with("<FILL_IN") {
                    let prompt = format!("Share set operator '{}' public key", op.name);
                    op.public_key = prompts::required_text(&prompt, None)?;
                }
            }
        }
        Ok(())
    }

    /// Check if config contains placeholder values.
    pub fn has_placeholders(&self) -> bool {
        self.name.starts_with("<FILL_IN")
            || self.manifest_set_params.as_ref().is_some_and(|p| {
                p.name.starts_with("<FILL_IN")
                    || p.new_operators
                        .iter()
                        .any(|o| o.public_key.starts_with("<FILL_IN"))
            })
            || self.share_set_params.as_ref().is_some_and(|p| {
                p.name.starts_with("<FILL_IN")
                    || p.new_operators
                        .iter()
                        .any(|o| o.public_key.starts_with("<FILL_IN"))
            })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.has_placeholders() {
            anyhow::bail!("config contains placeholder values (<FILL_IN_...)");
        }

        if self.manifest_set_id.is_some() && self.manifest_set_params.is_some() {
            anyhow::bail!("Cannot specify both manifestSetId and manifestSetParams");
        }
        if self.manifest_set_id.is_none() && self.manifest_set_params.is_none() {
            anyhow::bail!("Must specify either manifestSetId or manifestSetParams");
        }
        if self.share_set_id.is_some() && self.share_set_params.is_some() {
            anyhow::bail!("Cannot specify both shareSetId and shareSetParams");
        }
        // It is fine if both share set id and params are none since we support a default dev share set

        if let Some(params) = &self.share_set_params {
            if params.threshold < MIN_SHARE_SET_THRESHOLD {
                anyhow::bail!(
                    "shareSetParams.threshold must be >= {MIN_SHARE_SET_THRESHOLD}, got {}",
                    params.threshold
                );
            }
        }

        Ok(())
    }

    pub fn effective_share_set_params(&self) -> Option<OperatorSetParams> {
        if self.share_set_id.is_some() {
            None
        } else {
            Some(
                self.share_set_params
                    .clone()
                    .unwrap_or_else(Self::share_set_params),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_config_json() -> serde_json::Value {
        json!({
            "name": "test-app",
            "quorumPublicKey": KNOWN_QUORUM_KEY,
            "manifestSetParams": {
                "name": "manifest-set",
                "threshold": 1,
                "newOperators": [{
                    "name": "operator-1",
                    "publicKey": "operator-public-key"
                }]
            }
        })
    }

    #[test]
    fn validate_accepts_omitted_share_set_params() {
        let config: AppConfig = serde_json::from_value(valid_config_json()).unwrap();

        config.validate().unwrap();
        assert_eq!(config.effective_share_set_params().unwrap().threshold, 2);
    }

    #[test]
    fn validate_accepts_share_set_id() {
        let mut json = valid_config_json();
        json["shareSetId"] = json!("share-set-id");
        let config: AppConfig = serde_json::from_value(json).unwrap();

        config.validate().unwrap();
        assert_eq!(config.share_set_id.as_deref(), Some("share-set-id"));
        assert!(config.effective_share_set_params().is_none());
    }

    #[test]
    fn validate_rejects_share_set_id_and_params() {
        let mut json = valid_config_json();
        json["shareSetId"] = json!("share-set-id");
        json["shareSetParams"] = json!({
            "name": "custom-share-set",
            "threshold": 2,
            "newOperators": []
        });
        let config: AppConfig = serde_json::from_value(json).unwrap();

        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("Cannot specify both shareSetId and shareSetParams"));
    }

    #[test]
    fn validate_rejects_low_share_set_threshold() {
        let mut json = valid_config_json();
        json["shareSetParams"] = json!({
            "name": "custom-share-set",
            "threshold": 1,
            "newOperators": []
        });
        let config: AppConfig = serde_json::from_value(json).unwrap();

        assert!(config
            .validate()
            .unwrap_err()
            .to_string()
            .contains("shareSetParams.threshold must be"));
    }
}
