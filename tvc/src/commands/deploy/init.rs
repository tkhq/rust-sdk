//! Deploy init command - generates a template config file.

use super::PORT_GUIDANCE;
use crate::{
    client::{build_client, fetch_tvc_deployment},
    config::{deploy::DeployConfig, turnkey},
    prompts::{bail_interactive_conflicts_with_non_interactive, ensure_stdin_is_tty},
};
use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::Args as ClapArgs;
use std::path::PathBuf;

pub(crate) const LONG_ABOUT: &str = r#"
Generate a deployment config file to edit, then pass to `tvc deploy create`.

--from-deployment <DEPLOY_ID> copies every field from that deployment, including
the expected pivot digest and debug mode (read from its manifest); a private-image
pull secret cannot be recovered and must be re-supplied. Without the flag, a blank
placeholder template is written."#;

/// Generate a template deployment configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(short, long, value_name = "PATH", env = "TVC_DEPLOY_CONFIG_OUT")]
    pub output: Option<PathBuf>,

    /// Seed the config from an existing deployment instead of a blank template.
    ///
    /// Fetches the deployment and copies every recoverable field, including the
    /// expected pivot digest and debug mode (read from its manifest). The digest
    /// is tied to the container image, so recompute it if you change the image. A
    /// pull secret, if the source used one, must be re-supplied.
    #[arg(long, value_name = "DEPLOY_ID", env = "TVC_FROM_DEPLOYMENT")]
    pub from_deployment: Option<String>,

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
    let Args {
        output,
        from_deployment,
        interactive: is_interactive,
    } = args;

    // Generate output filename with timestamp if not provided
    let output = output.unwrap_or_else(|| {
        let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
        PathBuf::from(format!("deploy-{timestamp}.json"))
    });

    // Check if file already exists
    if output.exists() {
        bail!("File already exists: {}", output.display());
    }

    let is_from_deployment = from_deployment.is_some();

    // Seed the config either from an existing deployment or a blank template.
    let mut config = match from_deployment {
        Some(deploy_id) => {
            // TODO:
            // see other TODO. TL;DR split fetching the data/resources separately
            // from building the client
            let auth = build_client().await?;
            let org_id = auth.org_id.clone();
            let deployment = fetch_tvc_deployment(&auth, org_id, deploy_id).await?;
            DeployConfig::try_from(deployment)?
        }
        None => {
            // Try to get the last created app ID to prefill the template.
            let last_app_id = turnkey::Config::load()
                .await
                .ok()
                .and_then(|config| config.get_last_app_id());
            DeployConfig::template(last_app_id.as_deref())
        }
    };

    // Optionally walk prompts to fill any remaining placeholders (e.g. the
    // expected pivot digest that `--from-deployment` deliberately leaves blank).
    // In `--from-deployment` mode the app ID is already set, so the saved-app-id
    // default is only consulted for a blank template.
    if is_interactive {
        let saved_app_id = turnkey::Config::load()
            .await
            .ok()
            .and_then(|config| config.get_last_app_id());
        config.fill_interactively(saved_app_id.as_deref())?;
    }

    let needs_pull_secret = config.pull_secret_is_placeholder();

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    if is_interactive {
        println!("Created deployment config: {}", output.display());
    } else {
        println!("Created deployment config template: {}", output.display());
    }
    println!();

    // Print the reminders specific to seeding a config from an existing deployment:
    // the digest and debug mode were copied from the source manifest (the digest is
    // image-coupled, so it must be recomputed if the image changes), and a pull secret
    // (if the source used one) cannot be recovered and must be re-supplied.
    if is_from_deployment {
        println!("Seeded from an existing deployment. Before deploying:");
        println!(
            "  - expectedPivotDigest and debug mode were copied from the source deployment.\n    \
             The digest is tied to the container image, so if you change\n    \
             pivotContainerImageUrl you MUST recompute the digest to match."
        );

        if needs_pull_secret {
            println!(
                "  - The source deployment used a private image. Its pull secret cannot be\n    \
                 recovered; pass `--pivot-pull-secret <PATH>` to `tvc deploy create` (or\n    \
                 remove pivotContainerEncryptedPullSecret if the new image is public)."
            );
        }

        println!();
    }

    if is_interactive {
        println!("Run: tvc deploy create --config-file {}", output.display());
    } else {
        println!("Edit the file to fill in your values, then run:");
        println!("  tvc deploy create --config-file {}", output.display());
    }
    println!();
    println!("{PORT_GUIDANCE}");

    Ok(())
}
