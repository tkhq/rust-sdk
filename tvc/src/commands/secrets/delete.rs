//! Secret delete command - permanently deletes secrets from Turnkey secret
//! storage.

use crate::client::build_client;
use crate::config::turnkey::Config;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::prompts::{self, error_required_in_non_interactive};
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use tracing::instrument;
use turnkey_client::generated::ActivityStatus;

pub const LONG_ABOUT: &str = r#"Permanently delete secrets from Turnkey secret storage.

Deletion cannot be undone. Interactive runs ask for confirmation first; pass
--yes to skip it. Non-interactive runs require --yes."#;

/// Permanently delete secrets from Turnkey secret storage.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// ID of a secret to delete. Repeat to delete several in one activity.
    #[arg(long = "id", value_name = "SECRET_ID", required = true)]
    ids: Vec<String>,

    /// Skip the confirmation prompt (required in non-interactive mode).
    #[arg(long)]
    yes: bool,
}

/// Run the secret delete command.
#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args, config: Config) -> Result<Outcome> {
    // Validate before any config, credential, or network work:
    // a non-interactive run cannot prompt, so it requires --yes.
    if ctx.is_non_interactive() && !args.yes {
        return Err(error_required_in_non_interactive("--yes"));
    }
    if !args.yes {
        prompts::confirm_or_bail(
            &format!(
                "Permanently delete secrets {}? This cannot be undone.",
                args.ids.join(", "),
            ),
            "deletion",
        )?;
    }

    let auth = build_client(&config).await?;
    let result = auth
        .client
        .delete_secret(auth.org_id, &args.ids)
        .await
        .context("failed to delete secrets")?;

    Ok(Outcome::SecretsDeleted(SecretsDeleted {
        secret_ids: result.result,
        activity_id: result.activity_id,
        activity_status: result.status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsDeleted {
    secret_ids: Vec<String>,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for SecretsDeleted {
    fn default() -> Self {
        Self {
            secret_ids: Vec::default(),
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Display for SecretsDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Secrets deleted.

Secret IDs: {}
Activity ID: {}
Activity Status: {}"#,
            self.secret_ids.join(", "),
            self.activity_id,
            self.activity_status.as_str_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::secrets::test_support::{test_config, test_ctx};
    use tempfile::TempDir;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn args(ids: &[&str], yes: bool) -> Args {
        Args {
            ids: ids.iter().map(ToString::to_string).collect(),
            yes,
        }
    }

    #[tokio::test]
    async fn run_deletes_by_id_with_yes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/delete_secrets"))
            .and(body_partial_json(serde_json::json!({
                "type": "ACTIVITY_TYPE_DELETE_SECRETS",
                "parameters": {"secretIds": ["secret-abc", "secret-def"]},
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activity": {
                    "type": "ACTIVITY_TYPE_DELETE_SECRETS",
                    "status": "ACTIVITY_STATUS_COMPLETED",
                    "id": "activity-delete",
                    "organizationId": "org-1",
                    "fingerprint": "fingerprint",
                    "result": {
                        "deleteSecretsResult": {"secretIds": ["secret-abc", "secret-def"]},
                    }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let dir = TempDir::new().unwrap();
        let outcome = run(
            &mut test_ctx(),
            args(&["secret-abc", "secret-def"], true),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretsDeleted(deleted) = outcome else {
            panic!("expected SecretsDeleted");
        };
        assert_eq!(deleted.secret_ids, vec!["secret-abc", "secret-def"]);
        assert_eq!(deleted.activity_id, "activity-delete");
    }

    #[tokio::test]
    async fn run_refuses_non_interactive_without_yes() {
        // test_ctx is non-interactive; no server: the run must fail before
        // any config, credential, or network work.
        let dir = TempDir::new().unwrap();
        let Err(error) = run(
            &mut test_ctx(),
            args(&["secret-abc"], false),
            test_config(&dir, "http://127.0.0.1:1"),
        )
        .await
        else {
            panic!("expected non-interactive run without --yes to fail");
        };

        assert!(
            error
                .to_string()
                .contains("--yes is required in non-interactive mode"),
            "unexpected error: {error}"
        );
    }
}
