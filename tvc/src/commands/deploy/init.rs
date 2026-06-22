//! Deploy init command - generates a template config file.

use super::PORT_GUIDANCE;
use crate::config::deploy::DeployConfig;
use crate::config::turnkey;
use crate::output::{Message, Shell};
use crate::prompts::{bail_interactive_conflicts_with_non_interactive, ensure_stdin_is_tty};
use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::io::Write;
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
pub async fn run<O: Write, E: Write>(
    args: Args,
    is_non_interactive: bool,
    shell: &mut Shell<O, E>,
) -> Result<()> {
    if args.interactive {
        if is_non_interactive {
            bail_interactive_conflicts_with_non_interactive()?;
        } else {
            ensure_stdin_is_tty()?;
        }
    }
    execute(args, shell).await
}

async fn execute<O: Write, E: Write>(args: Args, shell: &mut Shell<O, E>) -> Result<()> {
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
        config.fill_interactively(last_app_id.as_deref(), shell)?;
    }

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    shell.emit(&DeploymentConfigCreated {
        command: "deploy init",
        path: output.display().to_string(),
        template: !args.interactive,
        interactive: args.interactive,
    })?;

    Ok(())
}

#[derive(Serialize)]
struct DeploymentConfigCreated {
    command: &'static str,
    path: String,
    template: bool,
    interactive: bool,
}

impl Message for DeploymentConfigCreated {
    fn reason(&self) -> &'static str {
        "deployment-config-created"
    }

    fn human_message(&self) -> String {
        if self.interactive {
            format!(
                "Created deployment config: {}\n\nRun: tvc deploy create --config-file {}\n\n{PORT_GUIDANCE}",
                self.path, self.path
            )
        } else {
            format!(
                "Created deployment config template: {}\n\nEdit the file to fill in your values, then run:\n  tvc deploy create --config-file {}\n\n{PORT_GUIDANCE}",
                self.path, self.path
            )
        }
    }
}
