//! Operator key backup command - copies a local operator key file to a
//! user-chosen destination.

use crate::{
    commands::{Run, login::resolve_org_query},
    config::turnkey::{Config, OrgQuery, StoredQosOperatorKey},
    outcome::Outcome,
    output::StdCtx,
    prompts::{self, error_required_in_non_interactive},
};
use anyhow::{Context, Result, anyhow, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};

/// Back up a local operator key by copying its key file to a chosen
/// destination.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Organization alias or ID whose operator key to back up.
    /// Defaults to the active organization.
    #[arg(long, env = "TVC_ORG", value_name = "ORG", value_parser = OrgQuery::from_str)]
    org: Option<OrgQuery>,
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

        let (alias, org_config) = self
            .org
            .as_ref()
            .map(|query| resolve_org_query(ctx, &config, query))
            .unwrap_or_else(|| {
                config
                    .active_org_config()
                    .map(|(alias, org)| (alias.as_str(), org))
                    .ok_or_else(|| anyhow!("No active organization. Run `tvc login` first."))
            })?;

        let source = &org_config.select_local_record(alias)?.key_path;

        let destination: PathBuf = self.output.map(Ok).unwrap_or_else(|| {
            prompts::text(
                "Backup file path",
                Some(&format!("operator-{alias}-backup.json")),
            )
            .map(Into::into)
        })?;

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

        let public_key = back_up(source, &destination).await?;

        Ok(OperatorKeyBackedUp::new(
            alias.to_string(),
            public_key,
            source,
            &destination,
        ))
    }
}

/// Copy the operator key at `source` to `destination` byte-for-byte and
/// return its public key.
///
/// The source is parsed to validate it and capture the public key, but the
/// original bytes are written verbatim so any unknown fields survive the
/// copy.
pub(crate) async fn back_up(source: &Path, destination: &Path) -> Result<String> {
    let bytes = tokio::fs::read(source).await.map_err(|e| match e.kind() {
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
    tokio::fs::write(destination, &bytes)
        .await
        .with_context(|| format!("failed to write backup: {}", destination.display()))?;

    Ok(key.public_key)
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

impl OperatorKeyBackedUp {
    /// The paths are projected to display strings here: the payload is a
    /// serialization shape, not a working set of paths.
    pub(crate) fn new(alias: String, public_key: String, source: &Path, backup: &Path) -> Self {
        Self {
            alias,
            public_key,
            source_path: source.display().to_string(),
            backup_path: backup.display().to_string(),
        }
    }
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

        let public_key = back_up(&source, &destination).await.unwrap();

        assert_eq!(std::fs::read_to_string(&destination).unwrap(), content);
        assert_eq!(public_key, "pub-hex");
    }

    #[tokio::test]
    async fn missing_source_names_path() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("operator.json");

        let error = back_up(&source, &temp.path().join("out.json"))
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

        let error = back_up(&source, &temp.path().join("out.json"))
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
        let outcome = Outcome::from(OperatorKeyBackedUp::new(
            "default".to_string(),
            "pub-hex".to_string(),
            Path::new("/keys/operator.json"),
            Path::new("/backups/operator-backup.json"),
        ));

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
