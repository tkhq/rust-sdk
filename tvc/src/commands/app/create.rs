//! App create command - creates an app from a config file.

use crate::client::build_client_with_overrides;
use crate::config::app::AppConfig;
use crate::config::turnkey;
use crate::output::Output;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    pub config_file: PathBuf,
}

#[derive(Serialize)]
struct AppCreateOutput {
    app_id: String,
    name: String,
    manifest_set_id: String,
    manifest_set_operator_ids: Vec<String>,
    config_file: String,
}

/// Run the app create command.
pub async fn run(args: Args, global: &crate::cli::GlobalOpts) -> Result<()> {
    let output = Output::new(global);
    // Read and parse config file
    let config_content = std::fs::read_to_string(&args.config_file)
        .with_context(|| format!("failed to read config file: {}", args.config_file.display()))?;

    let app_config: AppConfig = serde_json::from_str(&config_content).with_context(|| {
        format!(
            "failed to parse config file: {}",
            args.config_file.display()
        )
    })?;

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

    output.status(&format!("Creating app '{}'...", app_config.name));

    // Build authenticated client
    let auth = build_client_with_overrides(global).await?;

    // Convert config to API intent
    let intent = CreateTvcAppIntent {
        name: app_config.name.clone(),
        quorum_public_key: app_config.quorum_public_key.clone(),
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
        share_set_id: None,
        share_set_params: {
            let p = AppConfig::share_set_params();
            Some(TvcOperatorSetParams {
                name: p.name,
                threshold: p.threshold,
                new_operators: p
                    .new_operators
                    .into_iter()
                    .map(|o| TvcOperatorParams {
                        name: o.name,
                        public_key: o.public_key,
                    })
                    .collect(),
                existing_operator_ids: p.existing_operator_ids,
            })
        },
        enable_egress: app_config.external_connectivity,
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

    let app_id = result.result.app_id;
    let operator_ids = result.result.manifest_set_operator_ids;
    let manifest_set_id = result.result.manifest_set_id;

    // save the app ID and operator_ids to config
    let mut config = turnkey::Config::load().await?;
    config.set_last_app_id(&app_id)?;
    config.set_last_operator_ids(&operator_ids)?;
    config.save().await?;

    let result_data = AppCreateOutput {
        app_id: app_id.clone(),
        name: app_config.name.clone(),
        manifest_set_id: manifest_set_id.clone(),
        manifest_set_operator_ids: operator_ids.clone(),
        config_file: args.config_file.display().to_string(),
    };

    output.result(&result_data, || {
        println!();
        println!("App created successfully!");
        println!();
        println!("App ID: {app_id}");
        println!("Name: {}", app_config.name);
        println!("Manifest Set ID: {manifest_set_id}");
        if !operator_ids.is_empty() {
            println!("Manifest Set Operator IDs: {}", operator_ids.join(", "));
        }
        println!("Config: {}", args.config_file.display());
        println!();
        println!(
            "Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`"
        );
    })?;

    Ok(())
}
