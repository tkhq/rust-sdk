//! Deployment configuration file format for `tvc deploy create`.

use crate::output::Ctx;
use crate::prompts;
use crate::shell_println;
use anyhow::{Context, Result, anyhow};
use qos_core::protocol::services::boot::VersionedManifest;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::io::Write;
use thiserror::Error;
use turnkey_client::generated::{
    external::data::v1::{TvcContainerSpec, TvcDeployment},
    immutable::common::v1::TvcHealthCheckType,
};
use uuid::Uuid;

/// Sentinel written by `tvc deploy init` to remind the user to either remove
/// the field (public image) or replace it with an encrypted pull secret
/// (private image). Treated as a placeholder by [`DeployConfig::has_placeholders`].
const PULL_SECRET_PLACEHOLDER: &str = "<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>";

/// Default QOS version selected for new deployments.
pub const DEFAULT_QOS_VERSION: &str = "0.12.1";

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
    /// Deploy in debug mode. A deployment with debug enabled is permanently
    /// marked insecure: enclave logs are forwarded to the host and attestation
    /// PCRs are zeroed, so anything the enclave processed may have leaked.
    #[serde(default)]
    pub dangerous_deploy_debug_mode: bool,
    #[serde(default)]
    pub pivot_container_encrypted_pull_secret: Option<String>,
    pub health_check_type: TvcHealthCheckType,
    pub health_check_port: u16,
    pub public_ingress_port: u16,
}

/// Build a config seeded from an existing deployment, for use as a template.
///
/// Copies every recoverable field. `expected_pivot_digest` and
/// `dangerous_deploy_debug_mode` are not on the fetched container spec — they
/// live in the QOS manifest — so the manifest bytes are decoded and both are
/// copied from it: the digest via `pivot_hash()`, debug mode via `debug_mode()`.
/// The digest is coupled to the container image, so if the user repoints the
/// config at a different image they must recompute it.
///
/// Errors if the deployment has no manifest or it cannot be decoded. The
/// manifest is only `Option` because of how the proto is defined; a real
/// deployment always carries one.
///
/// The pull-secret plaintext is unrecoverable (it is encrypted for the
/// enclave), so a source deployment that used one gets the placeholder to
/// prompt the user to re-supply it via `--pivot-pull-secret`.
impl TryFrom<TvcDeployment> for DeployConfig {
    type Error = anyhow::Error;

    fn try_from(deployment: TvcDeployment) -> std::result::Result<Self, Self::Error> {
        let TvcDeployment {
            id,
            app_id,
            qos_version,
            pivot_container,
            manifest,
            ..
        } = deployment;

        let tvc_manifest =
            manifest.ok_or_else(|| anyhow!("deployment {id} has no manifest to seed from"))?;
        let manifest = VersionedManifest::try_from_slice_compat(&tvc_manifest.manifest)
            .with_context(|| format!("failed to decode QOS manifest for deployment {id}"))?;

        let TvcContainerSpec {
            container_url,
            path,
            args,
            has_pull_secret,
            health_check_type,
            health_check_port,
            public_ingress_port,
        } = pivot_container
            .ok_or_else(|| anyhow!("deployment {id} has no pivot container spec"))?;

        let health_check_port = u16::try_from(health_check_port)
            .context("deployment health check port does not fit in u16")?;
        let public_ingress_port = u16::try_from(public_ingress_port)
            .context("deployment public ingress port does not fit in u16")?;

        Ok(Self {
            app_id,
            qos_version,
            pivot_container_image_url: container_url,
            pivot_path: path,
            pivot_args: args,
            expected_pivot_digest: hex::encode(manifest.pivot_hash()),
            dangerous_deploy_debug_mode: manifest.debug_mode(),
            pivot_container_encrypted_pull_secret: has_pull_secret
                .then(|| PULL_SECRET_PLACEHOLDER.to_string()),
            health_check_type,
            health_check_port,
            public_ingress_port,
        })
    }
}

impl DeployConfig {
    /// Generate a default template config with placeholders.
    // Future: Could auto-fill appId if there's only one app in the org
    pub fn template(app_id: Option<&str>) -> Self {
        Self {
            app_id: app_id.unwrap_or("<FILL_IN_APP_ID>").to_string(),
            qos_version: DEFAULT_QOS_VERSION.to_string(),
            pivot_container_image_url: "<FILL_IN_PIVOT_CONTAINER_IMAGE_URL>".to_string(),
            pivot_path: "<FILL_IN_PIVOT_PATH>".to_string(),
            pivot_args: vec![],
            expected_pivot_digest: "<FILL_IN_EXPECTED_PIVOT_DIGEST>".to_string(),
            dangerous_deploy_debug_mode: false,
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
    pub fn fill_interactively<W: Write, W2: Write>(
        &mut self,
        ctx: &mut Ctx<W, W2>,
        saved_app_id: Option<&str>,
    ) -> Result<()> {
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
            let is_public = prompts::confirm("Is the container image in a public registry?", true)?;
            self.pivot_container_encrypted_pull_secret = None;
            if !is_public {
                shell_println!(
                    ctx,
                    "Note: pass `--pivot-pull-secret <PATH>` when running \
                     `tvc deploy create` to encrypt and attach the pull secret."
                )?;
            }
        }
        Ok(())
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

    /// Return the user-facing flag names of fields still containing
    /// `<FILL_IN...>` placeholders, in struct order. Empty if complete.
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.app_id.starts_with("<FILL_IN") {
            missing.push("--app-id");
        }
        if self.qos_version.starts_with("<FILL_IN") {
            missing.push("--qos-version");
        }
        if self.pivot_container_image_url.starts_with("<FILL_IN") {
            missing.push("--pivot-image-url");
        }
        if self.pivot_path.starts_with("<FILL_IN") {
            missing.push("--pivot-path");
        }
        if self.expected_pivot_digest.starts_with("<FILL_IN") {
            missing.push("--expected-pivot-digest");
        }
        missing
    }

    pub fn pull_secret_is_placeholder(&self) -> bool {
        self.pivot_container_encrypted_pull_secret.as_deref() == Some(PULL_SECRET_PLACEHOLDER)
    }

    pub fn validate(&self) -> Result<(), DeployConfigValidationErrors> {
        let mut errors = Vec::new();
        if self.app_id.starts_with("<FILL_IN") {
            errors.push(DeployConfigValidationError::Placeholder {
                field: "app_id",
                placeholder: self.app_id.clone(),
            });
        } else if Uuid::try_parse(&self.app_id).is_err() {
            errors.push(DeployConfigValidationError::InvalidAppId {
                value: self.app_id.clone(),
            });
        }
        if self.qos_version.starts_with("<FILL_IN") {
            errors.push(DeployConfigValidationError::Placeholder {
                field: "qos_version",
                placeholder: self.qos_version.clone(),
            });
        }
        if self.pivot_container_image_url.starts_with("<FILL_IN") {
            errors.push(DeployConfigValidationError::Placeholder {
                field: "pivot_container_image_url",
                placeholder: self.pivot_container_image_url.clone(),
            });
        }
        if self.pivot_path.starts_with("<FILL_IN") {
            errors.push(DeployConfigValidationError::Placeholder {
                field: "pivot_path",
                placeholder: self.pivot_path.clone(),
            });
        }
        if self.expected_pivot_digest.starts_with("<FILL_IN") {
            errors.push(DeployConfigValidationError::Placeholder {
                field: "expected_pivot_digest",
                placeholder: self.expected_pivot_digest.clone(),
            });
        }
        if self.pull_secret_is_placeholder() {
            errors.push(DeployConfigValidationError::PullSecretPlaceholder {
                placeholder: PULL_SECRET_PLACEHOLDER.to_string(),
            });
        }

        DeployConfigValidationErrors::ok_or_errors(errors)
    }
}

#[derive(Debug, Clone, Error)]
pub enum DeployConfigValidationError {
    #[error("{field} contains placeholder value {placeholder}")]
    Placeholder {
        field: &'static str,
        placeholder: String,
    },
    #[error("app_id is not a valid UUID: {value}")]
    InvalidAppId { value: String },
    #[error(
        "pivotContainerEncryptedPullSecret contains placeholder value {placeholder}; pass \
         --pivot-pull-secret <PATH> or remove pivotContainerEncryptedPullSecret for public images"
    )]
    PullSecretPlaceholder { placeholder: String },
}

impl DeployConfigValidationError {
    pub fn is_placeholder(&self) -> bool {
        matches!(
            self,
            Self::Placeholder { .. } | Self::PullSecretPlaceholder { .. }
        )
    }
}

#[derive(Debug)]
pub struct DeployConfigValidationErrors(Vec<DeployConfigValidationError>);

impl DeployConfigValidationErrors {
    fn ok_or_errors(errors: Vec<DeployConfigValidationError>) -> Result<(), Self> {
        if errors.is_empty() {
            return Ok(());
        }

        Err(Self(errors))
    }

    pub fn has_non_placeholder_error(&self) -> bool {
        self.0.iter().any(|e| !e.is_placeholder())
    }
}

impl Display for DeployConfigValidationErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = self
            .0
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; ");

        Display::fmt(&s, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::EmptyShell;
    use turnkey_client::generated::external::data::v1::{
        TvcContainerSpec, TvcDeployment, TvcManifest,
    };

    /// A real (JSON) QOS manifest fixture, shared with the approve tests.
    const MANIFEST_JSON: &str = include_str!("../../fixtures/manifest.json");
    /// The `pivot.hash` value inside [`MANIFEST_JSON`].
    const FIXTURE_PIVOT_HASH: &str =
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    /// Build a `TvcManifest` from the fixture, optionally flipping debug mode on.
    fn manifest(debug_mode: bool) -> TvcManifest {
        let json = if debug_mode {
            MANIFEST_JSON.replace(r#""debugMode": false"#, r#""debugMode": true"#)
        } else {
            MANIFEST_JSON.to_string()
        };
        TvcManifest {
            id: "manifest-1".into(),
            manifest: json.into_bytes(),
            created_at: None,
            updated_at: None,
        }
    }

    fn sample_deployment(has_pull_secret: bool) -> TvcDeployment {
        TvcDeployment {
            id: "deploy-123".into(),
            organization_id: "org-1".into(),
            app_id: "app-1".into(),
            manifest_set: None,
            share_set: None,
            manifest: Some(manifest(false)),
            manifest_approvals: vec![],
            qos_version: "0.6.1".into(),
            pivot_container: Some(TvcContainerSpec {
                container_url: "ghcr.io/x/y@sha256:img".into(),
                path: "/bin/pivot".into(),
                args: vec!["--flag".into(), "value".into()],
                has_pull_secret,
                health_check_type: TvcHealthCheckType::Http,
                health_check_port: 8080,
                public_ingress_port: 9090,
            }),
            created_at: None,
            updated_at: None,
            delete: false,
            debug_mode: false,
        }
    }

    #[test]
    fn from_deployment_copies_recoverable_fields() {
        let config = DeployConfig::try_from(sample_deployment(false)).unwrap();
        assert_eq!(config.app_id, "app-1");
        assert_eq!(config.qos_version, "0.6.1");
        assert_eq!(config.pivot_container_image_url, "ghcr.io/x/y@sha256:img");
        assert_eq!(config.pivot_path, "/bin/pivot");
        assert_eq!(config.pivot_args, vec!["--flag", "value"]);
        assert_eq!(config.health_check_type, TvcHealthCheckType::Http);
        assert_eq!(config.health_check_port, 8080);
        assert_eq!(config.public_ingress_port, 9090);
    }

    #[test]
    fn from_deployment_copies_digest_and_debug_from_manifest() {
        let config = DeployConfig::try_from(sample_deployment(false)).unwrap();
        // Digest and debug mode are copied from the QOS manifest, not left blank.
        assert_eq!(config.expected_pivot_digest, FIXTURE_PIVOT_HASH);
        assert!(!config.dangerous_deploy_debug_mode);
        // A seeded config therefore has no remaining placeholder fields.
        assert!(config.missing_required_fields().is_empty());
    }

    #[test]
    fn from_deployment_copies_debug_mode_when_source_is_debug() {
        let mut deployment = sample_deployment(false);
        deployment.manifest = Some(manifest(true));
        let config = DeployConfig::try_from(deployment).unwrap();
        // Debug mode is carried over as-is from the source manifest.
        assert!(config.dangerous_deploy_debug_mode);
    }

    #[test]
    fn from_deployment_sets_pull_secret_placeholder_only_for_private_images() {
        let public = DeployConfig::try_from(sample_deployment(false)).unwrap();
        assert_eq!(public.pivot_container_encrypted_pull_secret, None);

        let private = DeployConfig::try_from(sample_deployment(true)).unwrap();
        assert!(private.pull_secret_is_placeholder());
    }

    #[test]
    fn from_deployment_errors_without_container_spec() {
        let mut deployment = sample_deployment(false);
        deployment.pivot_container = None;
        assert!(DeployConfig::try_from(deployment).is_err());
    }

    #[test]
    fn from_deployment_errors_when_port_exceeds_u16() {
        let mut deployment = sample_deployment(false);
        deployment
            .pivot_container
            .as_mut()
            .unwrap()
            .public_ingress_port = 70_000;
        assert!(DeployConfig::try_from(deployment).is_err());
    }

    #[test]
    fn from_deployment_errors_without_manifest() {
        let mut deployment = sample_deployment(false);
        deployment.manifest = None;
        assert!(DeployConfig::try_from(deployment).is_err());
    }

    #[test]
    fn from_deployment_errors_with_undecodable_manifest() {
        let mut deployment = sample_deployment(false);
        deployment.manifest = Some(TvcManifest {
            id: "manifest-1".into(),
            manifest: b"not a valid manifest".to_vec(),
            created_at: None,
            updated_at: None,
        });
        assert!(DeployConfig::try_from(deployment).is_err());
    }

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

        let shell = EmptyShell::default();
        let mut ctx = Ctx::new(shell, false);
        config.fill_interactively(&mut ctx, None).unwrap();
        assert_eq!(config.app_id, "app_xyz");
        assert_eq!(config.qos_version, "0.6.1");
        assert_eq!(config.pivot_container_image_url, "ghcr.io/x/y:v1");
        assert_eq!(config.pivot_path, "/bin/pivot");
        assert_eq!(config.expected_pivot_digest, "sha256:abc");
        assert_eq!(config.pivot_container_encrypted_pull_secret, None);
    }

    #[test]
    fn fully_filled_config_has_no_placeholders() {
        let mut config = DeployConfig::template(None);
        config.app_id = "651b573c-861b-4f10-a478-cbcfe0c226af".into();
        config.qos_version = "0.6.1".into();
        config.pivot_container_image_url = "ghcr.io/x/y:v1".into();
        config.pivot_path = "/bin/pivot".into();
        config.expected_pivot_digest = "sha256:abc".into();
        config.pivot_container_encrypted_pull_secret = None;
        assert!(!config.has_placeholders());
    }

    /// Build a config with every required field filled by a valid (non-placeholder)
    /// value, so `validate()` returns `Ok` unless a test perturbs one field.
    fn valid_config() -> DeployConfig {
        let mut config = DeployConfig::template(None);
        config.app_id = "651b573c-861b-4f10-a478-cbcfe0c226af".into();
        config.qos_version = "0.6.1".into();
        config.pivot_container_image_url = "ghcr.io/x/y:v1".into();
        config.pivot_path = "/bin/pivot".into();
        config.expected_pivot_digest = "sha256:abc".into();
        config.pivot_container_encrypted_pull_secret = None;
        config
    }

    #[test]
    fn validate_rejects_non_uuid_app_id() {
        let mut config = valid_config();
        config.app_id = "not-a-uuid".into();
        let errors = config.validate().unwrap_err();
        assert!(errors.to_string().contains("not a valid UUID"), "{errors}");
        // A malformed app_id is a hard error, so interactive mode bails instead
        // of prompting the user to "fill in" an already-present value.
        assert!(errors.has_non_placeholder_error());
    }

    #[test]
    fn validate_accepts_well_formed_uuid() {
        // Turnkey resource IDs come in both UUID v4 and v7 forms; accept either.
        for app_id in [
            "651b573c-861b-4f10-a478-cbcfe0c226af", // v4
            "019660f7-801d-75d8-a40e-e4f69944b711", // v7
        ] {
            let mut config = valid_config();
            config.app_id = app_id.into();
            assert!(config.validate().is_ok(), "should accept {app_id}");
        }
    }

    /// The placeholder branch takes precedence over the UUID check, so an
    /// unfilled `<FILL_IN_APP_ID>` is reported as a placeholder (which interactive
    /// mode prompts to fill), not as an "invalid UUID" hard error.
    #[test]
    fn validate_reports_placeholder_app_id_as_placeholder() {
        let config = DeployConfig::template(None);
        let errors = config.validate().unwrap_err();
        let msg = errors.to_string();
        assert!(msg.contains("placeholder"), "{msg}");
        assert!(!msg.contains("not a valid UUID"), "{msg}");
        assert!(!errors.has_non_placeholder_error());
    }
}
