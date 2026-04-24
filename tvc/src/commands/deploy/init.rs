//! Deploy init command - generates a template config file.

use crate::config::deploy::DeployConfig;
use crate::config::turnkey;
use crate::prompts;
use crate::replay::ReplayHint;
use anyhow::{bail, Context, Result};
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

    /// Walk through prompts for each field and write a filled config instead
    /// of a placeholder template.
    #[arg(long)]
    pub interactive: bool,
}

/// Run the deploy init command.
pub async fn run(args: Args) -> Result<()> {
    if args.interactive && prompts::non_interactive_forced() {
        bail!(
            "--interactive conflicts with {}=1",
            prompts::NON_INTERACTIVE_ENV
        );
    }

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
    let config = turnkey::Config::load().await?;
    let last_app_id = config.get_last_app_id();

    // Generate template (optionally walking prompts to fill it in)
    let template = DeployConfig::template(last_app_id.as_deref());
    let config = if args.interactive {
        template.fill_interactively(last_app_id.as_deref())?
    } else {
        template
    };

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    if args.interactive {
        println!("Created deployment config: {}", output.display());
        println!();
        println!("Run: tvc deploy create {}", output.display());
    } else {
        println!("Created deployment config template: {}", output.display());
        println!();
        println!("Edit the file to fill in your values, then run:");
        println!("  tvc deploy create {}", output.display());
    }

    let mut hint = ReplayHint::new("deploy init").literal("--output", output.display().to_string());
    if args.interactive {
        hint = hint.flag("--interactive");
    }
    hint.print();

    Ok(())
}
