//! Deploy create command - creates a deployment from a config file.

use crate::client::build_client;
use crate::config::deploy::DeployConfig;
use crate::output::Output;
use crate::pull_secret::encrypt_pivot_pull_secret;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::CreateTvcDeploymentIntent;

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

#[derive(Serialize)]
struct DeployCreateOutput {
    deployment_id: String,
    app_id: String,
    config_file: String,
}

/// Run the deploy create command.
pub async fn run(args: Args, global: &crate::cli::GlobalOpts) -> Result<()> {
    let output = Output::new(global);
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

    output.status(&format!(
        "Creating deployment for app '{}'...",
        deploy_config.app_id
    ));

    // Build authenticated client
    let auth = build_client(&global.client_overrides()).await?;

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

    // Convert config to API intent
    let intent = CreateTvcDeploymentIntent {
        app_id: deploy_config.app_id.clone(),
        qos_version: deploy_config.qos_version.clone(),
        pivot_container_image_url: deploy_config.pivot_container_image_url.clone(),
        pivot_path: deploy_config.pivot_path.clone(),
        pivot_args: deploy_config.pivot_args.clone(),
        expected_pivot_digest: deploy_config.expected_pivot_digest.clone(),
        pivot_container_encrypted_pull_secret,
        debug_mode: deploy_config.debug_mode,
        nonce: None,
        health_check_type: deploy_config.health_check_type,
        health_check_port: deploy_config.health_check_port as u32,
        public_ingress_port: deploy_config.public_ingress_port as u32,
    };

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Create the deployment
    let api_result = auth
        .client
        .create_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC deployment")?;

    let deployment_id = api_result.result.deployment_id;

    let result_data = DeployCreateOutput {
        deployment_id: deployment_id.clone(),
        app_id: deploy_config.app_id.clone(),
        config_file: args.config_file.display().to_string(),
    };

    output.result(&result_data, || {
        println!();
        println!("Deployment created successfully!");
        println!();
        println!("Deployment ID: {deployment_id}");
        println!("App ID: {}", deploy_config.app_id);
        println!("Config: {}", args.config_file.display());
        println!();
        println!("Next steps:");
        println!(
            "  - Run `tvc deploy status --deploy-id {deployment_id}` to check deployment status"
        );
        println!(
            "  - Run `tvc deploy approve --deploy-id {deployment_id}` to approve the manifest"
        );
    })?;

    Ok(())
}
