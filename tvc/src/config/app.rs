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
    ("dev-share-operator-1", "0405c071cb82f6e47e0379a65019a0db27f4f554ddc494b8b24191a737603015a19b7a3668d0a98aa5a08af8ec509bfc26ad3763f33f2b2a1c854beb16898731cc04b39440511740326e9077e720233f22120c387505ce21747c7dc48f1d67e4a0e6424eae6b2396777ffb436f10615d045bd4b46dae1fc925698b50bc642af388f7"),
    ("dev-share-operator-2", "04b58d66efd7c0eee569c74fae47f956b4628d455077eb5a78a24c69c0e67a7d1b743c0fd5d823659c6c75d367455bfe1f83c5a19bc71e2e511fc6b028ea1f536a0485873f2949c66f4a6aa05c9cfcc18e3e6e4585a7967a5408e4f22306547cc0a8128b1bae0b84b52abf29424bd101a52e8d2f552e33e2c3bb304b3d30eeee75c5"),
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
