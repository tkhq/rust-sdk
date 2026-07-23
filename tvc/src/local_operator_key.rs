//! Resolve local operator credentials from CLI args or the active org registry.

use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::pair::{HexSeed, LocalPair};
use anyhow::{Context, anyhow, bail};
use std::path::PathBuf;

/// An explicit operator master seed given on the command line.
#[derive(Debug)]
pub enum LocalOperatorSeedSource {
    /// The seed itself, already validated by clap.
    Value(HexSeed),
    /// Path to a file containing the raw hex seed.
    Path(PathBuf),
}

impl LocalOperatorSeedSource {
    /// Convert the raw `--operator-seed` / `--operator-seed-path` args into a
    /// seed source. `None` when neither flag was given; an error when both were
    /// given.
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

/// Format-specific input used to resolve a local operator.
#[derive(Debug)]
pub enum LocalCredentialSource {
    /// A registered `StoredQosOperatorKey` JSON document.
    RegisteredKeyFile(PathBuf),
    /// A one-shot raw seed value or raw-hex file from CLI arguments.
    Explicit(LocalOperatorSeedSource),
}

/// Resolve one local key pair. Explicit credentials bypass config entirely;
/// otherwise the sole active local registry entry supplies the JSON key path.
pub async fn resolve_local_operator(
    explicit: Option<LocalOperatorSeedSource>,
) -> anyhow::Result<LocalPair> {
    let source = match explicit {
        Some(source) => LocalCredentialSource::Explicit(source),
        None => {
            let config = Config::load().await?;
            let (alias, org_config) = config.active_org_config().ok_or_else(|| {
                anyhow!(
                    "No active organization. Run `tvc login` first or provide \
                     --operator-seed or --operator-seed-path."
                )
            })?;
            let local = org_config.select_local_record(alias)?;
            return resolve_registered_local_operator(local.key_path.clone()).await;
        }
    };

    resolve_local_credential(source).await
}

/// Resolve a registered local operator without loading its registry entry
/// again. The path points to `StoredQosOperatorKey` JSON.
pub(crate) async fn resolve_registered_local_operator(
    key_path: PathBuf,
) -> anyhow::Result<LocalPair> {
    resolve_local_credential(LocalCredentialSource::RegisteredKeyFile(key_path)).await
}

/// Load a local pair with the parser required by the credential source.
/// Registered paths contain JSON; explicit paths contain raw hex.
async fn resolve_local_credential(source: LocalCredentialSource) -> anyhow::Result<LocalPair> {
    match source {
        LocalCredentialSource::Explicit(LocalOperatorSeedSource::Value(seed)) => {
            LocalPair::from_seed(&seed)
        }
        LocalCredentialSource::Explicit(LocalOperatorSeedSource::Path(path)) => {
            LocalPair::from_master_seed(&path).await
        }
        // TODO(TVC-202): remove hardcoded user-facing messages mentioning other commands
        LocalCredentialSource::RegisteredKeyFile(path) => {
            let key = StoredQosOperatorKey::load(&path).await?.ok_or_else(|| {
                anyhow!(
                    "No operator key found at '{}'. Run `tvc login` first or provide \
                     --operator-seed or --operator-seed-path.",
                    path.display()
                )
            })?;
            LocalPair::from_hex_seed(&key.private_key)
                .with_context(|| format!("failed to load operator key: {}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::StoredQosOperatorKey;
    use crate::pair::Pair;
    use std::fs;
    use tempfile::TempDir;

    fn seed() -> HexSeed {
        "ab".repeat(32).parse().unwrap()
    }

    #[test]
    fn from_args_with_neither_is_none() {
        assert!(
            LocalOperatorSeedSource::from_args(None, None)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn from_args_with_seed_is_value() {
        let source = LocalOperatorSeedSource::from_args(Some(seed()), None).unwrap();
        assert!(matches!(source, Some(LocalOperatorSeedSource::Value(_))));
    }

    #[test]
    fn from_args_with_path_is_path() {
        let source =
            LocalOperatorSeedSource::from_args(None, Some(PathBuf::from("seed.hex"))).unwrap();
        assert!(matches!(source, Some(LocalOperatorSeedSource::Path(_))));
    }

    #[test]
    fn from_args_with_both_is_an_error() {
        let err = LocalOperatorSeedSource::from_args(Some(seed()), Some(PathBuf::from("seed.hex")))
            .unwrap_err();
        assert!(err.to_string().contains("mutually exclusive"));
    }

    #[tokio::test]
    async fn registered_source_parses_stored_key_json() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("operator.json");
        let private_key = "ab".repeat(32);
        fs::write(
            &path,
            serde_json::to_string(&StoredQosOperatorKey {
                public_key: "unused".to_string(),
                private_key: private_key.clone(),
            })
            .unwrap(),
        )
        .unwrap();

        let pair = resolve_local_credential(LocalCredentialSource::RegisteredKeyFile(path))
            .await
            .unwrap();
        let expected = LocalPair::from_hex_seed(&private_key).unwrap();
        assert_eq!(pair.public_key(), expected.public_key());
    }

    #[tokio::test]
    async fn explicit_raw_hex_path_uses_raw_seed_parser() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("seed.hex");
        let private_key = "cd".repeat(32);
        fs::write(&path, format!("  0x{private_key}\n")).unwrap();

        let pair = resolve_local_credential(LocalCredentialSource::Explicit(
            LocalOperatorSeedSource::Path(path),
        ))
        .await
        .unwrap();
        assert_eq!(
            pair.public_key(),
            LocalPair::from_hex_seed(&private_key).unwrap().public_key()
        );
    }

    #[tokio::test]
    async fn explicit_value_resolves_without_config() {
        resolve_local_operator(Some(LocalOperatorSeedSource::Value(seed())))
            .await
            .unwrap();
    }
}
