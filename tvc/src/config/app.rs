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

/// Known share set public keys. This is for known share keys
/// that encrypt known operator keys. Assume the secrets are well known
pub const KNOWN_SHARE_SET_KEYS: [(&str, &str); 2] = [
    ("dev-share-operator-1", "044af8b082b9ef41a238037811a188309d8c8b00b6d49c0574538d7746d7383739e67e1107f134bc102a48301b07e7c53280decbe9c16c9fc1f19b9832018e1485048139aa5de49d9505465bcf1a879954c51ba7b258b669f4e42697088cbbca54aeb888d61e65b2602ce92ae945a0160533acc94942511f8e5b1940ed89cc8f141f"),
    ("dev-share-operator-2", "04c1c4b4eb784505f167affae00e18b1521e7a0bfa3be46e6a6b43ba1f386afce48d964c885480cb197e3538fd30ebe38a07f76b6a286b37ba6d2abddbbd6c9c8304e492ca7bce95912a7b2565c8553e38cf3a4b1f858171900ed81888282db13d41e214dd6def2de2aacb1fcf92e3ae5a83e1b0ffa660fc59b9dd10e277cfd128dc"),
];

/// Well known Quorum Key. This is for applications that do not need secure quorum keys
pub const KNOWN_QUORUM_KEY: &str = "04451028fc9d42cef6d8f2a3ebe17d65783c470dbc6f04663d500c12009930cf9b209e733f6ac6103cc28f07ecde2dbb55095738b828d6b7a55caf4ddf9d67f2ae047827dcd2325b8d58694c2ea14e8f1e1f8a36c84438d291ff9b1b067debdb3e2ba3822984cde8bed4de2c237bd323526da4961d368bcc63cbd2d37d00e936683e";

impl AppConfig {
    /// Generate a default template config with placeholders.
    pub fn template() -> Self {
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
                    public_key: "<FILL_IN_OPERATOR_PUBLIC_KEY>".to_string(),
                }],
                existing_operator_ids: vec![],
            }),
            share_set_id: None,
            share_set_params: Some(OperatorSetParams {
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
            }),
        }
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
    }
}
