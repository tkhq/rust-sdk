//! App init command - generates a template config file.

use crate::config::app::AppConfig;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::prompts::{bail_interactive_conflicts_with_non_interactive, ensure_stdin_is_tty};
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
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
pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    if args.interactive {
        if ctx.is_non_interactive() {
            bail_interactive_conflicts_with_non_interactive()?;
        } else {
            ensure_stdin_is_tty()?;
        }
    }
    execute(args).await
}

async fn execute(args: Args) -> Result<Outcome> {
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

    Ok(Outcome::AppInit(AppConfigCreated {
        command: "app init",
        path: args.output.display().to_string(),
        template: !args.interactive,
        interactive: args.interactive,
    }))
}

#[derive(Default, Serialize)]
pub struct AppConfigCreated {
    command: &'static str,
    path: String,
    template: bool,
    interactive: bool,
}

impl Display for AppConfigCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.interactive {
            write!(
                f,
                r#"Created app config: {}

Run: tvc app create --config-file {}"#,
                self.path, self.path
            )
        } else {
            write!(
                f,
                r#"Created app config template: {}

Edit the file to fill in your values, then run:
  tvc app create --config-file {}"#,
                self.path, self.path
            )
        }
    }
}

/// Load the operator public key from the active org's config
async fn load_operator_public_key() -> Option<String> {
    // Load config (return None on error)
    let config = Config::load().await.ok()?;

    // Get active org config
    let (alias, org_config) = config.active_org_config()?;
    let local = org_config.select_local_record(alias).ok()?;
    let operator_key = StoredQosOperatorKey::load(&local.key_path).await.ok()??;
    Some(operator_key.public_key)
}
