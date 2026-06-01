//! Deploy init command - generates a template config file.

use crate::config::deploy::DeployConfig;
use crate::config::turnkey;
use crate::prompts::{bail_interactive_conflicts_with_non_interactive, ensure_stdin_is_tty};
use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Generate a template deployment configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(short, long, value_name = "PATH", env = "TVC_DEPLOY_CONFIG_OUT")]
    pub output: Option<PathBuf>,

    /// Walk through prompts for each field and write a filled config instead
    /// of a placeholder template.
    #[arg(long)]
    pub interactive: bool,
}

/// Run the deploy init command.
pub async fn run(args: Args, is_non_interactive: bool) -> Result<()> {
    if args.interactive {
        if is_non_interactive {
            bail_interactive_conflicts_with_non_interactive()?;
        } else {
            ensure_stdin_is_tty()?;
        }
    }
    execute(args).await
}

async fn execute(args: Args) -> Result<()> {
    // Generate output filename with timestamp if not provided
    let output = args.output.unwrap_or_else(|| {
        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
        PathBuf::from(format!("deploy-{timestamp}.json"))
    });

    // Check if file already exists
    if output.exists() {
        bail!("File already exists: {}", output.display());
    }

    // Try to get the last created app ID
    let last_app_id = turnkey::Config::load()
        .await
        .ok()
        .and_then(|config| config.get_last_app_id());

    // Generate template (optionally walking prompts to fill it in)
    let mut config = DeployConfig::template(last_app_id.as_deref());
    if args.interactive {
        config.fill_interactively(last_app_id.as_deref())?;
    }

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    if args.interactive {
        println!("Created deployment config: {}", output.display());
        println!();
        println!("Run: tvc deploy create --config-file {}", output.display());
    } else {
        println!("Created deployment config template: {}", output.display());
        println!();
        println!("Edit the file to fill in your values, then run:");
        println!("  tvc deploy create --config-file {}", output.display());
    }

    Ok(())
}
