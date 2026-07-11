//! Quorum key init command - generates a template quorum key config file.

use crate::config::quorum_key::QuorumKeyConfig;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::output::Ctx;
use crate::shell_line;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::io::Write;
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
pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> Result<()> {
    if args.output.exists() {
        anyhow::bail!("File already exists: {}", args.output.display());
    }

    let operator_public_key = load_operator_public_key().await;

    let config = QuorumKeyConfig::template(operator_public_key.as_deref());
    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    std::fs::write(&args.output, json)
        .with_context(|| format!("failed to write file: {}", args.output.display()))?;

    shell_line!(
        ctx,
        "Created quorum key config template: {}",
        args.output.display()
    )?;
    shell_line!(ctx)?;
    // Constraints inherited from qos_crypto::shamir::shares_generate.
    shell_line!(ctx, "Constraints (see qos_crypto/src/shamir.rs):")?;
    shell_line!(ctx, "  shares    : 1..=255")?;
    shell_line!(ctx, "  threshold : >= 2 and <= shares")?;
    shell_line!(ctx)?;
    shell_line!(ctx, "Edit the file to fill in your values, then run:")?;
    shell_line!(
        ctx,
        "  tvc keys generate-quorum-key --config-file {}",
        args.output.display()
    )?;

    Ok(())
}

/// Load the operator public key from the active org's config.
async fn load_operator_public_key() -> Option<String> {
    let config = Config::load().await.ok()?;
    let (_, org_config) = config.active_org_config()?;
    let operator_key = StoredQosOperatorKey::load(org_config).await.ok()??;
    Some(operator_key.public_key)
}
