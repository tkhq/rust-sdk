//! Resolve an operator's QOS key pair from a seed SOURCE or the active org config.

use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::pair::LocalPair;
use anyhow::{Context, anyhow};
use std::io::Read;

/// Load the operator's QOS key pair from a seed `source` spec, or fall back to
/// the operator key stored under the active org config when `source` is `None`.
///
/// The source is flexible so the seed needn't be written to disk:
///   - `env:NAME`             read the hex seed from environment variable `NAME`
///   - `stdin` (or `-`)       read the hex seed from stdin
///   - `file://<path>` / `file:<path>` / bare `<path>`  read the hex seed from a file
pub async fn load_operator_pair(source: Option<&str>) -> anyhow::Result<LocalPair> {
    match source {
        Some(spec) => LocalPair::from_hex_seed(read_seed_source(spec)?.trim()),
        None => {
            let tvc_config = Config::load().await?;
            let (alias, org_config) = tvc_config.active_org_config().ok_or_else(|| {
                anyhow!("No active organization. Run `tvc login` first or provide --operator-seed.")
            })?;

            let operator_key = StoredQosOperatorKey::load(org_config)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No operator key found for org '{alias}'. \
                         Run `tvc login` first or provide --operator-seed."
                    )
                })?;

            LocalPair::from_hex_seed(&operator_key.private_key)
        }
    }
}

/// Read a hex seed from a source spec (see [`load_operator_pair`]). A bare value
/// with no recognized scheme is treated as a file path (back-compatible with the
/// previous `--operator-seed <PATH>` behavior).
fn read_seed_source(spec: &str) -> anyhow::Result<String> {
    if spec == "stdin" || spec == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .context("read operator seed from stdin")?;
        Ok(s)
    } else if let Some(name) = spec.strip_prefix("env:") {
        std::env::var(name).with_context(|| format!("operator seed env var '{name}' is not set"))
    } else {
        let path = spec
            .strip_prefix("file://")
            .or_else(|| spec.strip_prefix("file:"))
            .unwrap_or(spec);
        std::fs::read_to_string(path).with_context(|| format!("read operator seed file '{path}'"))
    }
}
