//! Deploy init command - generates a template config file.

use super::PORT_GUIDANCE;
use crate::{
    client::{build_client, fetch_tvc_deployment},
    config::{deploy::DeployConfig, turnkey},
    output::{Ctx, Message},
    prompts::{bail_interactive_conflicts_with_non_interactive, ensure_stdin_is_tty},
};
use anyhow::{Context, Result, bail};
use chrono::Local;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::io::Write;
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
pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> Result<()> {
    if args.interactive {
        if ctx.is_non_interactive() {
            bail_interactive_conflicts_with_non_interactive()?;
        } else {
            ensure_stdin_is_tty()?;
        }
    }
    execute(ctx, args).await
}

async fn execute<W: Write>(ctx: &mut Ctx<W>, args: Args) -> Result<()> {
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
            // TODO (TVC-154):
            // TL;DR split fetching the data/resources separately
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
        config.fill_interactively(ctx, saved_app_id.as_deref())?;
    }

    let needs_pull_secret = config.pull_secret_is_placeholder();

    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    // Write to file
    std::fs::write(&output, json)
        .with_context(|| format!("failed to write file: {}", output.display()))?;

    ctx.shell().emit(&DeploymentConfigCreated {
        command: "deploy init",
        path: output.display().to_string(),
        template: !is_interactive,
        interactive: is_interactive,
        from_deployment: is_from_deployment,
        needs_pull_secret,
    })?;

    Ok(())
}

#[derive(Serialize)]
struct DeploymentConfigCreated {
    command: &'static str,
    path: String,
    template: bool,
    interactive: bool,
    from_deployment: bool,
    needs_pull_secret: bool,
}

impl Message for DeploymentConfigCreated {
    fn reason(&self) -> &'static str {
        "deployment-config-created"
    }

    fn human_message(&self) -> String {
        let mut message = if self.interactive {
            format!("Created deployment config: {}\n", self.path)
        } else {
            format!("Created deployment config template: {}\n", self.path)
        };

        // The digest and debug mode were copied from the source manifest (the
        // digest is image-coupled, so it must be recomputed if the image
        // changes), and a pull secret (if the source used one) cannot be
        // recovered and must be re-supplied.
        if self.from_deployment {
            message.push_str(
                "\nSeeded from an existing deployment. Before deploying:\n  \
                 - expectedPivotDigest and debug mode were copied from the source deployment.\n    \
                 The digest is tied to the container image, so if you change\n    \
                 pivotContainerImageUrl you MUST recompute the digest to match.\n",
            );

            if self.needs_pull_secret {
                message.push_str(
                    "  - The source deployment used a private image. Its pull secret cannot be\n    \
                     recovered; pass `--pivot-pull-secret <PATH>` to `tvc deploy create` (or\n    \
                     remove pivotContainerEncryptedPullSecret if the new image is public).\n",
                );
            }
        }

        if self.interactive {
            message.push_str(&format!(
                "\nRun: tvc deploy create --config-file {}\n\n{PORT_GUIDANCE}",
                self.path
            ));
        } else {
            message.push_str(&format!(
                "\nEdit the file to fill in your values, then run:\n  tvc deploy create --config-file {}\n\n{PORT_GUIDANCE}",
                self.path
            ));
        }

        message
    }
}
