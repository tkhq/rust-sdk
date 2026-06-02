//! App init command - generates a template config file.

use crate::config::app::AppConfig;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::prompts;
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Generate a template app configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(
        short,
        long,
        value_name = "PATH",
        default_value = "app.json",
        env = "TVC_APP_CONFIG_OUT"
    )]
    pub output: PathBuf,

    /// Walk through prompts for each field and write a filled config instead
    /// of a placeholder template.
    #[arg(long)]
    pub interactive: bool,
}

/// Run the app init command.
pub async fn run(args: Args) -> Result<()> {
    if args.interactive && prompts::non_interactive_forced() {
        bail!(
            "--interactive conflicts with {}=1",
            prompts::NON_INTERACTIVE_ENV
        );
    }

    // Check if file already exists
    if args.output.exists() {
        bail!("File already exists: {}", args.output.display());
    }

    // Try to load operator public key from config
    let operator_public_key = load_operator_public_key().await;

    // Generate template (optionally walking prompts to fill it in)
    let mut config = AppConfig::template(operator_public_key.as_deref());
    if args.interactive {
        config.fill_interactively(operator_public_key.as_deref())?;
    }

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&args.output, json)
        .with_context(|| format!("failed to write file: {}", args.output.display()))?;

    if args.interactive {
        println!("Created app config: {}", args.output.display());
        println!();
        println!(
            "Run: tvc app create --config-file {}",
            args.output.display()
        );
    } else {
        println!("Created app config template: {}", args.output.display());
        println!();
        println!("Edit the file to fill in your values, then run:");
        println!("  tvc app create --config-file {}", args.output.display());
    }

    Ok(())
}

/// Load the operator public key from the active org's config
async fn load_operator_public_key() -> Option<String> {
    // Load config (return None on error)
    let config = Config::load().await.ok()?;

    // Get active org config
    let (_, org_config) = config.active_org_config()?;
    let operator_key = StoredQosOperatorKey::load(org_config).await.ok()??;
    Some(operator_key.public_key)
}
