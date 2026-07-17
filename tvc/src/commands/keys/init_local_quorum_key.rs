//! Local quorum key init command - generates a template quorum key config file.

use crate::config::quorum_key::QuorumKeyConfig;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::output::StdCtx;
use crate::shell_println;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::path::PathBuf;

/// Generate a template quorum key configuration file.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Output file path.
    #[arg(
        short,
        long,
        value_name = "PATH",
        default_value = "quorum_key.json",
        env = "TVC_QUORUM_KEY_CONFIG_OUT"
    )]
    pub output: PathBuf,
}

/// Run the quorum key init command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<()> {
    if args.output.exists() {
        anyhow::bail!("File already exists: {}", args.output.display());
    }

    let operator_public_key = load_operator_public_key().await;

    let config = QuorumKeyConfig::template(operator_public_key.as_deref());
    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    std::fs::write(&args.output, json)
        .with_context(|| format!("failed to write file: {}", args.output.display()))?;

    shell_println!(
        ctx,
        "Created quorum key config template: {}",
        args.output.display()
    )?;
    shell_println!(ctx)?;
    // Constraints inherited from qos_crypto::shamir::shares_generate.
    shell_println!(ctx, "Constraints (see qos_crypto/src/shamir.rs):")?;
    shell_println!(ctx, "  shares    : 1..=255")?;
    shell_println!(ctx, "  threshold : >= 2 and <= shares")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Edit the file to fill in your values, then run:")?;
    shell_println!(
        ctx,
        "  tvc keys generate-local-quorum-key --config-file {}",
        args.output.display()
    )?;

    Ok(())
}

/// Load the operator public key from the active org's config.
async fn load_operator_public_key() -> Option<String> {
    let config = Config::load().await.ok()?;
    let (alias, org_config) = config.active_org_config()?;
    let local = org_config.select_local_record(alias).ok()?;
    let operator_key = StoredQosOperatorKey::load(&local.key_path).await.ok()??;
    Some(operator_key.public_key)
}
