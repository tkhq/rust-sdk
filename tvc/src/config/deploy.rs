//! Deployment configuration file format for `tvc deploy create`.

use serde::{Deserialize, Serialize};

/// Deployment configuration loaded from JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployConfig {
    pub app_id: String,
    pub qos_version: String,
    pub pivot_container_image_url: String,
    pub pivot_path: String,
    #[serde(default)]
    pub pivot_args: Vec<String>,
    pub expected_pivot_digest: String,
    pub host_container_image_url: String,
    pub host_path: String,
    #[serde(default)]
    pub host_args: Vec<String>,
}

impl DeployConfig {
    /// Generate a default template config with placeholders.
    // Future: Could auto-fill appId if there's only one app in the org
    pub fn template() -> Self {
        Self {
            app_id: "<FILL_IN_APP_ID>".to_string(),
            qos_version: "<FILL_IN_QOS_VERSION>".to_string(),
            pivot_container_image_url: "<FILL_IN_PIVOT_CONTAINER_IMAGE_URL>".to_string(),
            pivot_path: "<FILL_IN_PIVOT_PATH>".to_string(),
            pivot_args: vec![],
            expected_pivot_digest: "<FILL_IN_EXPECTED_PIVOT_DIGEST>".to_string(),
            host_container_image_url: "<FILL_IN_HOST_CONTAINER_IMAGE_URL>".to_string(),
            host_path: "<FILL_IN_HOST_PATH>".to_string(),
            host_args: vec![],
        }
    }

    /// Check if config contains placeholder values.
    pub fn has_placeholders(&self) -> bool {
        self.app_id.starts_with("<FILL_IN")
            || self.qos_version.starts_with("<FILL_IN")
            || self.pivot_container_image_url.starts_with("<FILL_IN")
            || self.pivot_path.starts_with("<FILL_IN")
            || self.expected_pivot_digest.starts_with("<FILL_IN")
            || self.host_container_image_url.starts_with("<FILL_IN")
            || self.host_path.starts_with("<FILL_IN")
    }
}
