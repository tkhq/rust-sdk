//! Deploy init command - generates a template config file.

use crate::config::deploy::DeployConfig;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Generate a template deployment configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(short, long, default_value = "deploy.json")]
    pub output: PathBuf,
}

/// Run the deploy init command.
pub async fn run(args: Args) -> Result<()> {
    // Check if file already exists
    if args.output.exists() {
        anyhow::bail!("File already exists: {}", args.output.display());
    }

    // Generate template
    let config = DeployConfig::template();
    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&args.output, json)
        .with_context(|| format!("failed to write file: {}", args.output.display()))?;

    println!(
        "Created deployment config template: {}",
        args.output.display()
    );
    println!();
    println!("Edit the file to fill in your values, then run:");
    println!("  tvc deploy create {}", args.output.display());

    Ok(())
}
