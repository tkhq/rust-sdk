//! App create command - creates an app from a config file.

use crate::client::build_client;
use crate::config::app::AppConfig;
use crate::config::turnkey::{self, StoredQosOperatorKey};
use crate::prompts;
use crate::replay::ReplayHint;
use anyhow::{anyhow, Context, Result};
use clap::Args as ClapArgs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcAppIntent, TvcOperatorParams, TvcOperatorSetParams};

/// Create a new TVC application from a config file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the app configuration file (JSON).
    pub config_file: PathBuf,
}

/// Run the app create command.
pub async fn run(args: Args) -> Result<()> {
    let app_config = load_or_fill_config(&args.config_file).await?;

    // Validate operator set config
    if app_config.manifest_set_id.is_some() && app_config.manifest_set_params.is_some() {
        anyhow::bail!("Cannot specify both manifestSetId and manifestSetParams");
    }
    if app_config.manifest_set_id.is_none() && app_config.manifest_set_params.is_none() {
        anyhow::bail!("Must specify either manifestSetId or manifestSetParams");
    }

    println!("Creating app '{}'...", app_config.name);

    // Build authenticated client
    let auth = build_client().await?;

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

    // save the app ID and operator_ids to config
    let mut config = turnkey::Config::load().await?;
    config.set_last_app_id(&app_id)?;
    config.set_last_operator_ids(&operator_ids)?;
    config.save().await?;

    println!();
    println!("App created successfully!");
    println!();
    println!("App ID: {app_id}");
    println!("Name: {}", app_config.name);
    println!("Manifest Set ID: {}", result.result.manifest_set_id);
    if !operator_ids.is_empty() {
        println!("Manifest Set Operator IDs: {}", operator_ids.join(", "));
    }
    println!("Config: {}", args.config_file.display());
    println!();
    println!(
        "Use one of the Manifest Set Operator IDs above with `tvc deploy approve --operator-id`"
    );

    ReplayHint::new("app create")
        .positional(args.config_file.display().to_string())
        .print();

    Ok(())
}

/// Load the app config, walking placeholders interactively when allowed.
/// Mirrors the helper in `deploy/create.rs`.
async fn load_or_fill_config(path: &Path) -> Result<AppConfig> {
    let read = std::fs::read_to_string(path);
    let (config, file_existed) = match read {
        Ok(content) => {
            let config: AppConfig = serde_json::from_str(&content)
                .with_context(|| format!("failed to parse config file: {}", path.display()))?;
            (config, true)
        }
        Err(_) if prompts::is_interactive() => (AppConfig::template(None), false),
        Err(e) => {
            return Err(anyhow!(e))
                .with_context(|| format!("failed to read config file: {}", path.display()));
        }
    };

    if file_existed && !config.has_placeholders() {
        return Ok(config);
    }

    if !prompts::is_interactive() {
        anyhow::bail!(
            "Config file contains placeholder values (<FILL_IN_...>). \
             Please edit {} and fill in all required values.",
            path.display()
        );
    }

    let saved_operator_public_key = load_saved_operator_public_key().await;
    let filled = config.fill_interactively(saved_operator_public_key.as_deref())?;

    let save = prompts::confirm(
        &format!("Save filled config to {}?", path.display()),
        true,
    )?;
    if save {
        let json = serde_json::to_string_pretty(&filled).context("failed to serialize config")?;
        std::fs::write(path, json)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        println!("Wrote {}", path.display());
    }

    Ok(filled)
}

/// Best-effort load of the operator public key from the active org's config
/// so we can offer it as the default for new-operator prompts.
async fn load_saved_operator_public_key() -> Option<String> {
    let config = turnkey::Config::load().await.ok()?;
    let (_, org_config) = config.active_org_config()?;
    let operator_key = StoredQosOperatorKey::load(org_config).await.ok()??;
    Some(operator_key.public_key)
}
