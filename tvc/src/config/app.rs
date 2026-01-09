//! App configuration file format for `tvc app create`.

use serde::{Deserialize, Serialize};

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

impl AppConfig {
    /// Generate a default template config with placeholders.
    pub fn template() -> Self {
        Self {
            name: "<FILL_IN_APP_NAME>".to_string(),
            quorum_public_key: "<FILL_IN_QUORUM_PUBLIC_KEY>".to_string(),
            external_connectivity: Some(false),
            manifest_set_id: None,
            manifest_set_params: Some(OperatorSetParams {
                name: "<FILL_IN_MANIFEST_SET_NAME>".to_string(),
                threshold: 1,
                new_operators: vec![OperatorParams {
                    name: "operator-1".to_string(),
                    public_key: "<FILL_IN_OPERATOR_PUBLIC_KEY>".to_string(),
                }],
                existing_operator_ids: vec![],
            }),
            share_set_id: None,
            share_set_params: Some(OperatorSetParams {
                name: "<FILL_IN_SHARE_SET_NAME>".to_string(),
                threshold: 1,
                new_operators: vec![OperatorParams {
                    name: "operator-1".to_string(),
                    public_key: "<FILL_IN_OPERATOR_PUBLIC_KEY>".to_string(),
                }],
                existing_operator_ids: vec![],
            }),
        }
    }

    /// Check if config contains placeholder values.
    pub fn has_placeholders(&self) -> bool {
        self.name.starts_with("<FILL_IN")
            || self.quorum_public_key.starts_with("<FILL_IN")
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
}
