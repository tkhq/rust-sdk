//! Deploy init command - generates a template config file.

use crate::config::deploy::DeployConfig;
use crate::config::turnkey;
use anyhow::{Context, Result};
use chrono::Local;
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Generate a template deployment configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Run the deploy init command.
pub async fn run(args: Args) -> Result<()> {
    // Generate output filename with timestamp if not provided
    let output = args.output.unwrap_or_else(|| {
        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
        PathBuf::from(format!("deploy-{timestamp}.json"))
    });

    // Check if file already exists
    if output.exists() {
        anyhow::bail!("File already exists: {}", output.display());
    }

    // Try to get the last created app ID
    let config = turnkey::Config::load().await?;
    let last_app_id = config.get_last_app_id();

    // Generate template
    let config = DeployConfig::template(last_app_id.as_deref());
    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    println!("Created deployment config template: {}", output.display());
    println!();
    println!("Edit the file to fill in your values, then run:");
    println!("  tvc deploy create {}", output.display());

    Ok(())
}
