//! Deploy create command - creates a deployment from a config file.

use crate::client::build_client;
use crate::config::deploy::DeployConfig;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::CreateTvcDeploymentIntent;

/// Create a new TVC deployment from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the deployment configuration file (JSON).
    pub config_file: PathBuf,
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

    // Convert config to API intent
    let intent = CreateTvcDeploymentIntent {
        app_id: deploy_config.app_id.clone(),
        qos_version: deploy_config.qos_version.clone(),
        pivot_container_image_url: deploy_config.pivot_container_image_url.clone(),
        pivot_path: deploy_config.pivot_path.clone(),
        pivot_args: deploy_config.pivot_args.clone(),
        expected_pivot_digest: deploy_config.expected_pivot_digest.clone(),
        host_container_image_url: deploy_config.host_container_image_url.clone(),
        host_path: deploy_config.host_path.clone(),
        host_args: deploy_config.host_args.clone(),
        nonce: None
    };

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
