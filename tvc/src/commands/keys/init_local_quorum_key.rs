//! Local quorum key init command - generates a template quorum key config file.

use crate::config::quorum_key::QuorumKeyConfig;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::outcome::Outcome;
use crate::output::StdCtx;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
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
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    if args.output.exists() {
        anyhow::bail!("File already exists: {}", args.output.display());
    }

    let operator_public_key = load_operator_public_key().await;

    let config = QuorumKeyConfig::template(operator_public_key.as_deref());
    let json = serde_json::to_string_pretty(&config).context("failed to serialize config")?;

    std::fs::write(&args.output, json)
        .with_context(|| format!("failed to write file: {}", args.output.display()))?;

    Ok(Outcome::KeysInitQuorumKey(QuorumKeyConfigCreated {
        path: args.output.display().to_string(),
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumKeyConfigCreated {
    path: String,
}

impl Display for QuorumKeyConfigCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Constraints inherited from qos_crypto::shamir::shares_generate.
        write!(
            f,
            r#"Created quorum key config template: {}

Constraints (see qos_crypto/src/shamir.rs):
  shares    : 1..=255
  threshold : >= 2 and <= shares

Edit the file to fill in your values, then run:
  tvc keys generate-local-quorum-key --config-file {}"#,
            self.path, self.path
        )
    }
}

/// Load the operator public key from the active org's config.
async fn load_operator_public_key() -> Option<String> {
    let config = Config::load().await.ok()?;
    let (alias, org_config) = config.active_org_config()?;
    let local = org_config.select_local_record(alias).ok()?;
    let operator_key = StoredQosOperatorKey::load(&local.key_path).await.ok()??;
    Some(operator_key.public_key)
}
