//! Deployment configuration file format for `tvc deploy create`.

use crate::prompts;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use turnkey_client::generated::immutable::common::v1::TvcHealthCheckType;

/// Sentinel written by `tvc deploy init` to remind the user to either remove
/// the field (public image) or replace it with an encrypted pull secret
/// (private image). Treated as a placeholder by [`DeployConfig::has_placeholders`].
const PULL_SECRET_PLACEHOLDER: &str = "<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>";

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
    #[serde(default)]
    pub debug_mode: Option<bool>,
    #[serde(default)]
    pub pivot_container_encrypted_pull_secret: Option<String>,
    pub health_check_type: TvcHealthCheckType,
    pub health_check_port: u16,
    pub public_ingress_port: u16,
}

impl DeployConfig {
    /// Generate a default template config with placeholders.
    // Future: Could auto-fill appId if there's only one app in the org
    pub fn template(app_id: Option<&str>) -> Self {
        Self {
            app_id: app_id.unwrap_or("<FILL_IN_APP_ID>").to_string(),
            qos_version: "<FILL_IN_QOS_VERSION>".to_string(),
            pivot_container_image_url: "<FILL_IN_PIVOT_CONTAINER_IMAGE_URL>".to_string(),
            pivot_path: "<FILL_IN_PIVOT_PATH>".to_string(),
            pivot_args: vec![],
            expected_pivot_digest: "<FILL_IN_EXPECTED_PIVOT_DIGEST>".to_string(),
            debug_mode: Some(false),
            pivot_container_encrypted_pull_secret: Some(PULL_SECRET_PLACEHOLDER.to_string()),
            health_check_type: TvcHealthCheckType::Http,
            health_check_port: 3000,
            public_ingress_port: 3000,
        }
    }

    /// Walk the user through any placeholder fields and fill them in.
    /// Non-placeholder fields are preserved unchanged so partial edits work.
    ///
    /// `saved_app_id` is offered as the default for the App ID prompt when set.
    pub fn fill_interactively(mut self, saved_app_id: Option<&str>) -> Result<Self> {
        if self.app_id.starts_with("<FILL_IN") {
            self.app_id = prompts::required_text("App ID", saved_app_id)?;
        }
        if self.qos_version.starts_with("<FILL_IN") {
            self.qos_version = prompts::required_text("QOS version", None)?;
        }
        if self.pivot_container_image_url.starts_with("<FILL_IN") {
            self.pivot_container_image_url =
                prompts::required_text("Pivot container image URL", None)?;
        }
        if self.pivot_path.starts_with("<FILL_IN") {
            self.pivot_path = prompts::required_text("Pivot path (inside container)", None)?;
        }
        if self.expected_pivot_digest.starts_with("<FILL_IN") {
            self.expected_pivot_digest =
                prompts::required_text("Expected pivot digest (sha256:...)", None)?;
        }
        if self.pivot_container_encrypted_pull_secret.as_deref() == Some(PULL_SECRET_PLACEHOLDER) {
            let is_public =
                prompts::confirm("Is the container image in a public registry?", true)?;
            self.pivot_container_encrypted_pull_secret = None;
            if !is_public {
                println!(
                    "Note: pass `--pivot-pull-secret <PATH>` when running \
                     `tvc deploy create` to encrypt and attach the pull secret."
                );
            }
        }
        Ok(self)
    }

    /// Check if config contains placeholder values.
    pub fn has_placeholders(&self) -> bool {
        self.app_id.starts_with("<FILL_IN")
            || self.qos_version.starts_with("<FILL_IN")
            || self.pivot_container_image_url.starts_with("<FILL_IN")
            || self.pivot_path.starts_with("<FILL_IN")
            || self.expected_pivot_digest.starts_with("<FILL_IN")
            || self.pivot_container_encrypted_pull_secret.as_deref()
                == Some(PULL_SECRET_PLACEHOLDER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_template_is_all_placeholders() {
        let config = DeployConfig::template(None);
        assert!(config.has_placeholders());
    }

    #[test]
    fn filled_config_with_pull_secret_still_placeholder_is_detected() {
        // Previously this case slipped through: all FILL_IN fields replaced,
        // but the pull-secret sentinel left intact.
        let mut config = DeployConfig::template(None);
        config.app_id = "app_123".into();
        config.qos_version = "0.6.1".into();
        config.pivot_container_image_url = "ghcr.io/x/y:v1".into();
        config.pivot_path = "/bin/pivot".into();
        config.expected_pivot_digest = "sha256:abc".into();
        assert!(
            config.has_placeholders(),
            "pull-secret sentinel must count as a placeholder"
        );
    }

    #[test]
    fn fill_interactively_is_noop_when_config_has_no_placeholders() {
        // If every field is already set, fill_interactively must not attempt
        // to prompt — this is the contract that lets unit tests run without
        // injecting stdin.
        let mut config = DeployConfig::template(None);
        config.app_id = "app_xyz".into();
        config.qos_version = "0.6.1".into();
        config.pivot_container_image_url = "ghcr.io/x/y:v1".into();
        config.pivot_path = "/bin/pivot".into();
        config.expected_pivot_digest = "sha256:abc".into();
        config.pivot_container_encrypted_pull_secret = None;

        let filled = config.clone().fill_interactively(None).unwrap();
        assert_eq!(filled.app_id, "app_xyz");
        assert_eq!(filled.qos_version, "0.6.1");
        assert_eq!(filled.pivot_container_image_url, "ghcr.io/x/y:v1");
        assert_eq!(filled.pivot_path, "/bin/pivot");
        assert_eq!(filled.expected_pivot_digest, "sha256:abc");
        assert_eq!(filled.pivot_container_encrypted_pull_secret, None);
    }

    #[test]
    fn fully_filled_config_has_no_placeholders() {
        let mut config = DeployConfig::template(None);
        config.app_id = "app_123".into();
        config.qos_version = "0.6.1".into();
        config.pivot_container_image_url = "ghcr.io/x/y:v1".into();
        config.pivot_path = "/bin/pivot".into();
        config.expected_pivot_digest = "sha256:abc".into();
        config.pivot_container_encrypted_pull_secret = None;
        assert!(!config.has_placeholders());
    }
}
