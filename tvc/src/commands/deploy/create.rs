//! Deploy create command - creates a deployment from a config file.

use crate::client::build_client;
use crate::config::deploy::DeployConfig;
use crate::pull_secret::encrypt_pivot_pull_secret;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcDeploymentIntent, ValidateTvcImageRequest};

/// Create a new TVC deployment from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the deployment configuration file (JSON).
    pub config_file: PathBuf,

    /// Path to an unencrypted pivot container pull secret file.
    ///
    /// The content will be encrypted based on the active org's API environment and
    /// override `pivotContainerEncryptedPullSecret` from the config file.
    #[arg(long, alias = "pull-secret", value_name = "PATH")]
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

/// Run the deploy create command.
pub async fn run(args: Args) -> Result<()> {
    // Read and parse config file
    let config_content = std::fs::read_to_string(&args.config_file)
        .with_context(|| format!("failed to read config file: {}", args.config_file.display()))?;

    let deploy_config: DeployConfig = serde_json::from_str(&config_content).with_context(|| {
        format!(
            "failed to parse config file: {}",
            args.config_file.display()
        )
    })?;

    // Validate config
    if deploy_config.has_placeholders() {
        anyhow::bail!(
            "Config file contains placeholder values (<FILL_IN_...>). \
             Please edit {} and fill in all required values.",
            args.config_file.display()
        );
    }

    println!("Creating deployment for app '{}'...", deploy_config.app_id);

    // Build authenticated client
    let auth = build_client().await?;

    let pivot_container_encrypted_pull_secret = match args.pivot_pull_secret.as_ref() {
        Some(path) => {
            let pull_secret = std::fs::read_to_string(path).with_context(|| {
                format!("failed to read pivot pull secret file: {}", path.display())
            })?;

            if pull_secret.trim().is_empty() {
                anyhow::bail!(
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
    println!("Config: {}", args.config_file.display());
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
