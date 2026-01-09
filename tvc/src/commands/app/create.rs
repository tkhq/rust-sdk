//! App create command - creates an app from a config file.

use crate::config::app::AppConfig;
use crate::client::build_client;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{
    CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams,
};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    pub config_file: PathBuf,
}

/// Run the app create command.
pub async fn run(args: Args, cli_config: &crate::cli::GlobalConfig) -> Result<()> {
    // Read and parse config file
    let config_content = std::fs::read_to_string(&args.config_file)
        .with_context(|| format!("failed to read config file: {}", args.config_file.display()))?;

    let app_config: AppConfig = serde_json::from_str(&config_content)
        .with_context(|| format!("failed to parse config file: {}", args.config_file.display()))?;

    // Validate config
    if app_config.has_placeholders() {
        anyhow::bail!(
            "Config file contains placeholder values (<FILL_IN_...>). \
             Please edit {} and fill in all required values.",
            args.config_file.display()
        );
    }

    // Validate operator set config
    if app_config.manifest_set_id.is_some() && app_config.manifest_set_params.is_some() {
        anyhow::bail!("Cannot specify both manifestSetId and manifestSetParams");
    }
    if app_config.manifest_set_id.is_none() && app_config.manifest_set_params.is_none() {
        anyhow::bail!("Must specify either manifestSetId or manifestSetParams");
    }
    if app_config.share_set_id.is_some() && app_config.share_set_params.is_some() {
        anyhow::bail!("Cannot specify both shareSetId and shareSetParams");
    }
    if app_config.share_set_id.is_none() && app_config.share_set_params.is_none() {
        anyhow::bail!("Must specify either shareSetId or shareSetParams");
    }

    println!("Creating app '{}'...", app_config.name);

    // Build authenticated client
    let auth = build_client(&cli_config.api_base_url).await?;

    // Convert config to API intent
    let intent = CreateTvcAppIntent {
        name: app_config.name.clone(),
        quorum_public_key: app_config.quorum_public_key.clone(),
        external_connectivity: app_config.external_connectivity,
        manifest_set_id: app_config.manifest_set_id.clone(),
        manifest_set_params: app_config.manifest_set_params.as_ref().map(|p| {
            TvcOperatorSetParams {
                name: p.name.clone(),
                threshold: p.threshold,
                new_operators: p
                    .new_operators
                    .iter()
                    .map(|o| TvcOperatorParams {
                        name: o.name.clone(),
                        public_key: o.public_key.clone(),
                    })
                    .collect(),
                existing_operator_ids: p.existing_operator_ids.clone(),
            }
        }),
        share_set_id: app_config.share_set_id.clone(),
        share_set_params: app_config.share_set_params.as_ref().map(|p| {
            TvcOperatorSetParams {
                name: p.name.clone(),
                threshold: p.threshold,
                new_operators: p
                    .new_operators
                    .iter()
                    .map(|o| TvcOperatorParams {
                        name: o.name.clone(),
                        public_key: o.public_key.clone(),
                    })
                    .collect(),
                existing_operator_ids: p.existing_operator_ids.clone(),
            }
        }),
    };

    // Get timestamp
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    // Create the app
    let result = auth
        .client
        .create_tvc_app(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to create TVC app")?;

    println!();
    println!("App created successfully!");
    println!();
    println!("App ID: {}", result.result.app_id);
    println!("Name: {}", app_config.name);
    println!("Config: {}", args.config_file.display());

    Ok(())
}
