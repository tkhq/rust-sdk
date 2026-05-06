//! Deploy create command - creates a deployment from a config file or CLI flags.

use crate::client::build_client;
use crate::config::deploy::DeployConfig;
use crate::pull_secret::encrypt_pivot_pull_secret;
use anyhow::{bail, Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcDeploymentIntent, ValidateTvcImageRequest};

/// Create a new TVC deployment from a config file or CLI flags.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the deployment configuration file (JSON).
    /// Optional when all required fields are provided via flags or env.
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_DEPLOY_CONFIG")]
    pub config_file: Option<PathBuf>,

    /// Override the appId field.
    #[arg(long, env = "TVC_APP_ID")]
    pub app_id: Option<String>,

    /// Override the qosVersion field.
    #[arg(long, env = "TVC_QOS_VERSION")]
    pub qos_version: Option<String>,

    /// Override the pivotContainerImageUrl field.
    #[arg(long, env = "TVC_PIVOT_IMAGE_URL")]
    pub pivot_image_url: Option<String>,

    /// Override the expectedPivotDigest field.
    #[arg(long, env = "TVC_EXPECTED_PIVOT_DIGEST")]
    pub expected_pivot_digest: Option<String>,

    /// Override the pivotPath field.
    #[arg(long, env = "TVC_PIVOT_PATH")]
    pub pivot_path: Option<String>,

    /// Override pivotArgs (replaces the file's list entirely; not appended).
    #[arg(long, value_name = "ARG", value_delimiter = ',', env = "TVC_PIVOT_ARGS")]
    pub pivot_args: Vec<String>,

    /// Enable debug mode. One-way: cannot disable a `true` set earlier via the file.
    #[arg(long, env = "TVC_DEBUG_MODE")]
    pub debug_mode: bool,

    /// Override the healthCheckPort field.
    #[arg(long, env = "TVC_HEALTH_CHECK_PORT")]
    pub health_check_port: Option<u16>,

    /// Override the publicIngressPort field.
    #[arg(long, env = "TVC_PUBLIC_INGRESS_PORT")]
    pub public_ingress_port: Option<u16>,

    /// Path to an unencrypted pivot container pull secret file.
    ///
    /// The content will be encrypted based on the active org's API environment and
    /// override `pivotContainerEncryptedPullSecret` from the config file.
    #[arg(long, value_name = "PATH", env = "TVC_PIVOT_PULL_SECRET")]
    pub pivot_pull_secret: Option<PathBuf>,
}

fn build_validate_image_request(
    organization_id: &str,
    image_url: &str,
    pivot_container_encrypted_pull_secret: Option<String>,
) -> ValidateTvcImageRequest {
    ValidateTvcImageRequest {
        organization_id: organization_id.to_string(),
        pivot_container_image_url: image_url.to_string(),
        pivot_container_encrypted_pull_secret,
    }
}

fn build_create_intent(
    deploy_config: &DeployConfig,
    pivot_container_image_url: String,
    pivot_container_encrypted_pull_secret: Option<String>,
) -> CreateTvcDeploymentIntent {
    CreateTvcDeploymentIntent {
        app_id: deploy_config.app_id.clone(),
        qos_version: deploy_config.qos_version.clone(),
        pivot_container_image_url,
        pivot_path: deploy_config.pivot_path.clone(),
        pivot_args: deploy_config.pivot_args.clone(),
        expected_pivot_digest: deploy_config.expected_pivot_digest.clone(),
        pivot_container_encrypted_pull_secret,
        debug_mode: deploy_config.debug_mode,
        nonce: None,
        health_check_type: deploy_config.health_check_type,
        health_check_port: deploy_config.health_check_port as u32,
        public_ingress_port: deploy_config.public_ingress_port as u32,
    }
}

fn pin_image_url(image_url: &str, resolved_digest: &str) -> String {
    if image_url.contains("@") {
        image_url.to_string()
    } else {
        format!("{image_url}@{resolved_digest}") // works for docker pull even if image_url included a :tag
    }
}

fn apply_overrides(config: &mut DeployConfig, args: &Args) {
    if let Some(v) = &args.app_id {
        config.app_id = v.clone();
    }
    if let Some(v) = &args.qos_version {
        config.qos_version = v.clone();
    }
    if let Some(v) = &args.pivot_image_url {
        config.pivot_container_image_url = v.clone();
    }
    if let Some(v) = &args.expected_pivot_digest {
        config.expected_pivot_digest = v.clone();
    }
    if let Some(v) = &args.pivot_path {
        config.pivot_path = v.clone();
    }
    if !args.pivot_args.is_empty() {
        config.pivot_args = args.pivot_args.clone();
    }
    // One-way: only ever flips false -> true.
    if args.debug_mode {
        config.debug_mode = Some(true);
    }
    if let Some(v) = args.health_check_port {
        config.health_check_port = v;
    }
    if let Some(v) = args.public_ingress_port {
        config.public_ingress_port = v;
    }
}

fn resolve_deploy_config(args: &Args) -> Result<DeployConfig> {
    let mut config = match &args.config_file {
        Some(path) => {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read config file: {}", path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("failed to parse config file: {}", path.display()))?
        }
        None => {
            let mut t = DeployConfig::template(None);
            // Strip the template's "<REMOVE_ME...>" hint so flag-only mode
            // doesn't ship it to the API for public images.
            t.pivot_container_encrypted_pull_secret = None;
            t
        }
    };

    apply_overrides(&mut config, args);

    let missing = config.missing_required_fields();
    if !missing.is_empty() {
        let suggestion = if args.config_file.is_some() {
            "Edit the config file or override via flag/TVC_* env."
        } else {
            "Provide via flag, TVC_* env, or --config-file."
        };
        bail!(
            "missing required values: {}. {suggestion}",
            missing.join(", ")
        );
    }

    Ok(config)
}

/// Run the deploy create command.
pub async fn run(args: Args) -> Result<()> {
    let deploy_config = resolve_deploy_config(&args)?;

    println!("Creating deployment for app '{}'...", deploy_config.app_id);

    // Build authenticated client
    let auth = build_client().await?;

    let pivot_container_encrypted_pull_secret = match args.pivot_pull_secret.as_ref() {
        Some(path) => {
            let pull_secret = std::fs::read_to_string(path).with_context(|| {
                format!("failed to read pivot pull secret file: {}", path.display())
            })?;

            if pull_secret.trim().is_empty() {
                bail!(
                    "pivot pull secret file is empty after trimming whitespace: {}",
                    path.display()
                );
            }

            Some(encrypt_pivot_pull_secret(&pull_secret, &auth.api_base_url)?)
        }
        None => deploy_config.pivot_container_encrypted_pull_secret.clone(),
    };

    let validate_image_request = build_validate_image_request(
        &auth.org_id,
        &deploy_config.pivot_container_image_url,
        pivot_container_encrypted_pull_secret.clone(),
    );

    let validate_image_response = auth
        .client
        .validate_tvc_image(validate_image_request)
        .await
        .context("failed to validate TVC image")?;

    let pinned_image_url = pin_image_url(
        &deploy_config.pivot_container_image_url,
        &validate_image_response.resolved_image_digest,
    );

    if pinned_image_url != deploy_config.pivot_container_image_url {
        println!("Using pinned image reference for deployment request: {pinned_image_url}");
    }

    let intent = build_create_intent(
        &deploy_config,
        pinned_image_url,
        pivot_container_encrypted_pull_secret,
    );

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Create the deployment
    let result = auth
        .client
        .create_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC deployment")?;

    println!();
    println!("Deployment created successfully!");
    println!();
    println!("Deployment ID: {}", result.result.deployment_id);
    println!("App ID: {}", deploy_config.app_id);
    if let Some(path) = &args.config_file {
        println!("Config: {}", path.display());
    }
    println!();
    println!("Next steps:");
    println!(
        "  - Run `WIP: tvc deploy status {}` to check deployment status",
        result.result.deployment_id
    );
    println!(
        "  - Run `tvc deploy approve --deploy-id {}` to approve the manifest",
        result.result.deployment_id
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::pin_image_url;

    #[test]
    fn pin_image_url_appends_digest_to_tagged_reference() {
        let image_url = "ghcr.io/team/app:latest";
        let pinned = pin_image_url(image_url, "sha256:abc123");

        assert_eq!(pinned, "ghcr.io/team/app:latest@sha256:abc123");
    }

    #[test]
    fn pin_image_url_appends_digest_to_untagged_reference() {
        let image_url = "ghcr.io/team/app";
        let pinned = pin_image_url(image_url, "sha256:abc123");

        assert_eq!(pinned, "ghcr.io/team/app@sha256:abc123");
    }
}
