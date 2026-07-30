//! Operator key backup command - copies a local operator key file to a
//! user-chosen destination.

use crate::commands::Run;
use crate::commands::login::resolve_org_query;
use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::prompts::{self, error_required_in_non_interactive};
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

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
    force: bool,
}

impl Run for Args {
    type Outcome = OperatorKeyBackedUp;

    async fn run(self, ctx: &mut StdCtx) -> Result<OperatorKeyBackedUp> {
        // Non-interactive mode cannot prompt for the destination; reject
        // before loading config or resolving the organization.
        if ctx.is_non_interactive() && self.output.is_none() {
            return Err(error_required_in_non_interactive("--output"));
        }

        let config = Config::load().await?;

        let alias = match &self.org {
            Some(query) => resolve_org_query(ctx, &config, query)?,
            None => match config.active_org_config() {
                Some((alias, _)) => alias.clone(),
                None => bail!("No active organization. Run `tvc login` first."),
            },
        };
        let org_config = config
            .orgs
            .get(&alias)
            .expect("alias was resolved from this config");
        let source = &org_config.select_local_record(&alias)?.key_path;

        let destination = match self.output {
            Some(path) => path,
            None => PathBuf::from(prompts::text(
                "Backup file path",
                Some(&format!("operator-{alias}-backup.json")),
            )?),
        };

        if destination.is_dir() {
            bail!(
                "destination {} is a directory; include a file name",
                destination.display()
            );
        }

        if destination.exists() && !self.force {
            if ctx.is_non_interactive() {
                bail!(
                    "destination {} already exists; pass --force to overwrite",
                    destination.display()
                );
            }

            prompts::confirm_or_bail(&format!("Overwrite {}?", destination.display()), "backup")?;
        }

        back_up_key(&alias, source, &destination).await
    }
}

/// Copy the operator key at `source` to `destination` byte-for-byte.
///
/// The source is parsed to validate it and capture its public key, but the
/// original bytes are written verbatim so any unknown fields survive the
/// copy.
pub(crate) async fn back_up_key(
    alias: &str,
    source: &Path,
    destination: &Path,
) -> Result<OperatorKeyBackedUp> {
    let bytes = match tokio::fs::read(source).await {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => bail!(
            "No operator key found at {} for org '{alias}'. Run `tvc login` first.",
            source.display()
        ),
        Err(e) => {
            return Err(e)
                .with_context(|| format!("failed to read operator key: {}", source.display()));
        }
    };

    let key: StoredQosOperatorKey = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "operator key at {} is not a valid operator key file",
            source.display()
        )
    })?;

    if let Some(parent) = destination.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("failed to create backup directory: {}", parent.display()))?;
    }

    // Written with default (umask) permissions, matching
    // `StoredQosOperatorKey::save`; tightening both is tracked by TVC-241.
    tokio::fs::write(destination, &bytes)
        .await
        .with_context(|| format!("failed to write backup: {}", destination.display()))?;

    Ok(OperatorKeyBackedUp {
        alias: alias.to_string(),
        public_key: key.public_key,
        source_path: source.display().to_string(),
        backup_path: destination.display().to_string(),
    })
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct OperatorKeyBackedUp {
    alias: String,
    public_key: String,
    source_path: String,
    backup_path: String,
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
            self.alias, self.public_key, self.source_path, self.backup_path
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
        let content = r#"{
  "public_key": "pub-hex",
  "private_key": "priv-hex",
  "future_field": 42
}"#;
        std::fs::write(&source, content).unwrap();

        let backed_up = back_up_key("default", &source, &destination).await.unwrap();

        assert_eq!(std::fs::read_to_string(&destination).unwrap(), content);
        assert_eq!(backed_up.public_key, "pub-hex");
    }

    #[tokio::test]
    async fn missing_source_names_org_and_path() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");

        let error = back_up_key("default", &source, &temp.path().join("out.json"))
            .await
            .expect_err("missing source must fail");

        assert_eq!(
            error.to_string(),
            format!(
                "No operator key found at {} for org 'default'. Run `tvc login` first.",
                source.display()
            )
        );
    }

    #[tokio::test]
    async fn malformed_source_names_path() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");
        std::fs::write(&source, "not json").unwrap();

        let error = back_up_key("default", &source, &temp.path().join("out.json"))
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
        let outcome = Outcome::from(OperatorKeyBackedUp {
            alias: "default".to_string(),
            public_key: "pub-hex".to_string(),
            source_path: "/keys/operator.json".to_string(),
            backup_path: "/backups/operator-backup.json".to_string(),
        });

        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            serde_json::json!({
                "reason": "operator_key_backed_up",
                "alias": "default",
                "publicKey": "pub-hex",
                "sourcePath": "/keys/operator.json",
                "backupPath": "/backups/operator-backup.json",
            })
        );
    }
}
