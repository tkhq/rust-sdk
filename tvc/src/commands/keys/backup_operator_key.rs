//! Operator key backup command - copies a local operator key file to a
//! user-chosen destination.

use crate::{
    commands::{Run, login::find_org},
    config::turnkey::{
        Config, QosOperatorPublicKey, SelectLocalOperatorError, StoredQosOperatorKey,
    },
    outcome::Outcome,
    output::StdCtx,
    prompts::{self, error_required_in_non_interactive},
};
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

/// Back up a local operator key by copying its key file to a chosen
/// destination.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID whose operator key to back up.
    /// Defaults to the active organization.
    #[arg(long, env = "TVC_ORG", value_name = "ORG")]
    org: Option<String>,
    /// Destination file for the backup copy.
    #[arg(short, long, value_name = "PATH", env = "TVC_OPERATOR_KEY_BACKUP_OUT")]
    output: Option<PathBuf>,
    /// Overwrite the destination if it already exists.
    #[arg(long)]
    overwrite: bool,
}

impl Run for Args {
    type Outcome = OperatorKeyBackedUp;

    async fn run(self, ctx: &mut StdCtx, config: Config) -> Result<OperatorKeyBackedUp> {
        // Reject before loading config or resolving the organization when
        // there is no way to prompt for the destination: --non-interactive,
        // JSON mode, or a non-TTY stdin.
        let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

        if !can_prompt && self.output.is_none() {
            return Err(error_required_in_non_interactive("--output"));
        }

        let (alias, org_config) = match &self.org {
            Some(query) => find_org(&config, query).ok_or_else(|| {
                anyhow!(
                    "Login profile '{query}' not found. \
                     Run `tvc login` to see configured profiles."
                )
            })?,
            None => config
                .active_org_config()
                .ok_or_else(|| anyhow!("No active organization. Run `tvc login` first."))?,
        };

        let (_, local) = org_config
            .select_local_operator()
            .map_err(|error| match error {
                // Nothing exportable exists for a hosted-only org; explain
                // that instead of leaving a bare missing-operator error.
                SelectLocalOperatorError::NoLocalOperator => {
                    anyhow::Error::new(error).context(format!(
                        "org '{alias}' has no local operator key file to back up; hosted \
                         operators' private keys are held by Turnkey and cannot be exported"
                    ))
                }
                SelectLocalOperatorError::MultipleLocalOperators => {
                    anyhow::Error::new(error).context(format!("org '{alias}'"))
                }
            })?;
        let source = &local.key_path;

        let destination = match self.output {
            // --output is a CLI argument: validate it, honoring --overwrite
            // and the non-interactive fence.
            Some(output) => {
                if output.is_dir() {
                    bail!(
                        "destination {} is a directory; include a file name",
                        output.display()
                    );
                }

                if output.exists() && !self.overwrite {
                    if !can_prompt {
                        bail!(
                            "destination {} already exists; pass --overwrite to replace it",
                            output.display()
                        );
                    }

                    prompts::confirm_or_bail(
                        &format!("Overwrite {}?", output.display()),
                        "backup",
                    )?;
                }

                output
            }
            // No --output: the shared interactive flow. Declining the
            // overwrite cancels the command - backing up is all it does.
            None => prompt_for_backup_destination(alias)?
                .ok_or_else(|| anyhow!("operation cancelled by user: backup"))?,
        };

        back_up(alias.to_string(), source.clone(), destination).await
    }
}

// PURE-DEPS-REVIEW T22 (medium): prompts + fs probes (is_dir/exists) below two
// different entrypoints (also called from login.rs). Keep as shared prompt-layer
// code, but extract the path validation into a pure fn both callers use.
/// Prompt for a backup destination: the default file name, the
/// directory-destination rejection, and the overwrite question live here, so
/// every interactive caller asks them the same way. Returns `None` when the
/// user declines the overwrite; callers own what declining means.
pub(crate) fn prompt_for_backup_destination(alias: &str) -> Result<Option<PathBuf>> {
    let destination: PathBuf = prompts::text(
        "Backup file path",
        Some(&format!("operator-{alias}-backup.json")),
    )?
    .into();

    if destination.is_dir() {
        bail!(
            "destination {} is a directory; include a file name",
            destination.display()
        );
    }

    if destination.exists()
        && !prompts::confirm(&format!("Overwrite {}?", destination.display()), false)?
    {
        return Ok(None);
    }

    Ok(Some(destination))
}

/// Copy the operator key at `source` to `destination` byte-for-byte and
/// return the backup report.
///
/// The source is parsed to validate it and capture the public key, but the
/// original bytes are written verbatim so any unknown fields survive the
/// copy. This is the only place an [`OperatorKeyBackedUp`] is constructed.
// PURE-DEPS-REVIEW T29 (low, borderline): dependencies ARE parameters here
// (good), but key-file validation (the serde parse whose error message is
// user-facing policy) is fused with create_dir_all + copy, so the parse step
// needs a real file to exercise.
pub(crate) async fn back_up(
    alias: String,
    source: PathBuf,
    destination: PathBuf,
) -> Result<OperatorKeyBackedUp> {
    let bytes = tokio::fs::read(&source).await.map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => anyhow!(
            "No operator key found at {}. Run `tvc login` first.",
            source.display()
        ),
        _ => anyhow::Error::new(e)
            .context(format!("failed to read operator key: {}", source.display())),
    })?;

    let key: StoredQosOperatorKey = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "operator key at {} is not a valid operator key file",
            source.display()
        )
    })?;

    // A bare-filename destination has the empty path as its parent, which
    // `create_dir_all` accepts as a no-op.
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create backup directory: {}", parent.display()))?;
    }

    // Written with default (umask) permissions, matching
    // `StoredQosOperatorKey::save`; tightening both is tracked by TVC-241.
    tokio::fs::copy(&source, &destination)
        .await
        .with_context(|| format!("failed to write backup: {}", destination.display()))?;

    Ok(OperatorKeyBackedUp {
        alias,
        public_key: key.public_key,
        source_path: source,
        backup_path: destination,
    })
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct OperatorKeyBackedUp {
    alias: String,
    public_key: QosOperatorPublicKey,
    source_path: PathBuf,
    backup_path: PathBuf,
}

impl From<OperatorKeyBackedUp> for Outcome {
    fn from(backed_up: OperatorKeyBackedUp) -> Self {
        Outcome::OperatorKeyBackedUp(backed_up)
    }
}

impl Display for OperatorKeyBackedUp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Operator key backed up!

Org:        {}
Public key: {}
Source:     {}
Backup:     {}

The backup contains the PRIVATE key. Store it somewhere safe - a password
manager or an encrypted offline drive - never in source control or chat.

To restore: copy the backup file back to the source path above, then run
`tvc login`."#,
            self.alias,
            self.public_key,
            self.source_path.display(),
            self.backup_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn backs_up_key_bytes_verbatim() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");
        let destination = temp.path().join("backups/operator-backup.json");
        // Unknown fields must survive the copy: the file is written verbatim,
        // not re-serialized.
        let public_hex = hex::encode(
            qos_p256::P256Pair::generate()
                .unwrap()
                .public_key()
                .to_bytes(),
        );
        let content = format!(
            r#"{{
  "public_key": "{public_hex}",
  "private_key": "priv-hex",
  "future_field": 42
}}"#
        );
        std::fs::write(&source, &content).unwrap();

        let report = back_up("default".to_string(), source.clone(), destination.clone())
            .await
            .unwrap();

        assert_eq!(std::fs::read_to_string(&destination).unwrap(), content);
        assert_eq!(report.public_key.to_string(), public_hex);
    }

    #[tokio::test]
    async fn missing_source_names_path() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");

        let error = back_up(
            "default".to_string(),
            source.clone(),
            temp.path().join("out.json"),
        )
        .await
        .expect_err("missing source must fail");

        assert_eq!(
            error.to_string(),
            format!(
                "No operator key found at {}. Run `tvc login` first.",
                source.display()
            )
        );
    }

    #[tokio::test]
    async fn malformed_source_names_path() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");
        std::fs::write(&source, "not json").unwrap();

        let error = back_up(
            "default".to_string(),
            source.clone(),
            temp.path().join("out.json"),
        )
        .await
        .expect_err("malformed source must fail");

        assert_eq!(
            error.to_string(),
            format!(
                "operator key at {} is not a valid operator key file",
                source.display()
            )
        );
    }

    #[test]
    fn outcome_serializes_expected_json() {
        let public_key = QosOperatorPublicKey::default();
        let outcome = Outcome::from(OperatorKeyBackedUp {
            alias: "default".to_string(),
            public_key,
            source_path: PathBuf::from("/keys/operator.json"),
            backup_path: PathBuf::from("/backups/operator-backup.json"),
        });

        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            serde_json::json!({
                "reason": "operator_key_backed_up",
                "alias": "default",
                "publicKey": public_key.to_string(),
                "sourcePath": "/keys/operator.json",
                "backupPath": "/backups/operator-backup.json",
            })
        );
    }
}
