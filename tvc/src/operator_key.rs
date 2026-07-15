//! Resolve an operator's QOS key pair from CLI args or the active org config.

use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::pair::{HexSeed, LocalPair};
use anyhow::{anyhow, bail};
use std::path::PathBuf;

/// An explicit operator master seed given on the command line.
#[derive(Debug)]
pub enum OperatorSeedSource {
    /// The seed itself, already validated by clap.
    Value(HexSeed),
    /// Path to a file containing the hex seed.
    Path(PathBuf),
}

impl OperatorSeedSource {
    /// Convert the raw `--operator-seed` / `--operator-seed-path` args into a
    /// seed source. `None` when neither flag was given (the caller falls back
    /// to the stored operator key); an error when both were given.
    pub fn from_args(seed: Option<HexSeed>, path: Option<PathBuf>) -> anyhow::Result<Option<Self>> {
        match (seed, path) {
            (Some(_), Some(_)) => {
                bail!(
                    "--operator-seed and --operator-seed-path are mutually exclusive, \
                     please provide only one"
                )
            }
            (Some(seed), None) => Ok(Some(Self::Value(seed))),
            (None, Some(path)) => Ok(Some(Self::Path(path))),
            (None, None) => Ok(None),
        }
    }
}

/// Load the operator's QOS key pair, preferring an explicit seed source and
/// falling back to the operator key stored under the active org config.
pub async fn load_operator_pair(source: Option<OperatorSeedSource>) -> anyhow::Result<LocalPair> {
    match source {
        Some(OperatorSeedSource::Value(seed)) => LocalPair::from_seed(&seed),
        Some(OperatorSeedSource::Path(path)) => LocalPair::from_master_seed(&path).await,
        None => {
            let tvc_config = Config::load().await?;
            let (alias, org_config) = tvc_config.active_org_config().ok_or_else(|| {
                anyhow!(
                    "No active organization. Run `tvc login` first or provide \
                     --operator-seed or --operator-seed-path."
                )
            })?;

            let operator_key = StoredQosOperatorKey::load(org_config)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No operator key found for org '{alias}'. Run `tvc login` first \
                         or provide --operator-seed or --operator-seed-path."
                    )
                })?;

            LocalPair::from_hex_seed(&operator_key.private_key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed() -> HexSeed {
        "ab".repeat(32).parse().unwrap()
    }

    #[test]
    fn from_args_with_neither_is_none() {
        assert!(OperatorSeedSource::from_args(None, None).unwrap().is_none());
    }

    #[test]
    fn from_args_with_seed_is_value() {
        let source = OperatorSeedSource::from_args(Some(seed()), None).unwrap();
        assert!(matches!(source, Some(OperatorSeedSource::Value(_))));
    }

    #[test]
    fn from_args_with_path_is_path() {
        let source = OperatorSeedSource::from_args(None, Some(PathBuf::from("seed.hex"))).unwrap();
        assert!(matches!(source, Some(OperatorSeedSource::Path(_))));
    }

    #[test]
    fn from_args_with_both_is_an_error() {
        let err = OperatorSeedSource::from_args(Some(seed()), Some(PathBuf::from("seed.hex")))
            .unwrap_err();
        assert!(err.to_string().contains("mutually exclusive"));
    }
}
