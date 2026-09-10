//! Secret export command - exports one secret value from Turnkey secret
//! storage.

use super::list_all_secrets;
use crate::client::{AuthenticatedClient, build_client};
use crate::config::turnkey::Config;
use crate::errors::MissingResource;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::signer_quorum::signer_quorum_key;
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::instrument;
use turnkey_client::ActivityResult;
use zeroize::Zeroizing;

pub const LONG_ABOUT: &str = r#"Export one secret value from Turnkey secret storage.

The value is decrypted with a single-use transport key inside this process.
Pass --out to write it to a new file readable only by the current user, or
pipe stdout to receive the exact value bytes. Printing to an interactive
terminal requires --plain. JSON output carries metadata only, never the
value, so --message-format json requires --out."#;

/// Export one secret value from Turnkey secret storage.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
#[command(group(clap::ArgGroup::new("selector").required(true)))]
pub struct Args {
    /// ID of the secret to export.
    #[arg(long, value_name = "SECRET_ID", group = "selector")]
    id: Option<String>,

    /// Name of the secret to export. Fails if the name does not resolve to
    /// exactly one secret.
    #[arg(long, value_name = "NAME", group = "selector")]
    name: Option<String>,

    /// Write the value to this new file (created with mode 0600) instead of
    /// printing it. Fails if the file already exists.
    #[arg(long, value_name = "PATH")]
    out: Option<PathBuf>,

    /// Print the value to an interactive terminal. Without it, printing is
    /// only allowed when stdout is piped.
    #[arg(long)]
    plain: bool,

    /// Hex signer quorum public key override. Defaults to the Turnkey key for
    /// the active org's environment (production or preprod).
    #[arg(long = "signer-quorum-key", value_name = "HEX")]
    signer_quorum_key_hex: Option<String>,
}

/// Decide where the decrypted value goes - `Some(path)` for a file, `None`
/// for stdout - from parsed arguments and terminal state, before any config,
/// credential, or network work.
fn decide_delivery(ctx: &mut StdCtx, out: Option<PathBuf>, plain: bool) -> Result<Option<PathBuf>> {
    if out.is_none() {
        if ctx.shell().message_format().is_json() {
            bail!("JSON output carries metadata only, never the value; pass --out FILE");
        }
        if std::io::stdout().is_terminal() && !plain {
            bail!(
                "refusing to print the secret value to an interactive terminal; \
                 pass --plain to print it anyway, or --out FILE to write it to a file"
            );
        }
    }
    Ok(out)
}

/// Resolve `--name` to exactly one secret ID via the metadata listing.
async fn resolve_secret_id(auth: &AuthenticatedClient, name: &str) -> Result<String> {
    let mut matches: Vec<String> = list_all_secrets(auth)
        .await?
        .into_iter()
        .filter(|metadata| metadata.name.as_deref() == Some(name))
        .map(|metadata| metadata.secret_id)
        .collect();
    match matches.len() {
        0 => Err(MissingResource::new("secret", name).into()),
        1 => Ok(matches.swap_remove(0)),
        _ => bail!(
            "secret name '{name}' is ambiguous; it matches {} secrets ({}). Use --id instead",
            matches.len(),
            matches.join(", ")
        ),
    }
}

/// Write the exported value to a new file readable only by the current user.
async fn write_value_file(path: &Path, value: &str) -> Result<()> {
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(path)
        .await
        .with_context(|| format!("failed to create output file: {}", path.display()))?;
    file.write_all(value.as_bytes())
        .await
        .with_context(|| format!("failed to write output file: {}", path.display()))?;
    file.flush()
        .await
        .with_context(|| format!("failed to write output file: {}", path.display()))
}

/// Run the secret export command.
#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args, config: Config) -> Result<Outcome> {
    let Args {
        id,
        name,
        out,
        plain,
        signer_quorum_key_hex,
    } = args;
    let delivery = decide_delivery(ctx, out, plain)?;

    let auth = build_client(&config).await?;
    let signer = signer_quorum_key(&auth.api_base_url, signer_quorum_key_hex.as_deref())?;

    let secret_id = match (id, name) {
        (Some(id), None) => id,
        (None, Some(name)) => resolve_secret_id(&auth, &name).await?,
        // Clap's argument group guarantees exactly one is present.
        _ => bail!("pass exactly one of --id or --name"),
    };

    let ActivityResult {
        result: plaintexts,
        activity_id,
        status: _,
        app_proofs: _,
    } = auth
        .client
        .export_secret(auth.org_id, &[secret_id.as_str()], &signer)
        .await
        .with_context(|| format!("failed to export secret {secret_id}"))?;

    // Wrap every returned value before any other handling so each is wiped
    // even on error paths. The client already errors when the payload count
    // does not match the requested IDs, so exactly one value is expected.
    let mut values: Vec<Zeroizing<String>> = plaintexts.into_iter().map(Zeroizing::new).collect();
    ensure!(
        values.len() == 1,
        "expected exactly one exported value, got {}",
        values.len()
    );
    let value = values.swap_remove(0);

    let destination = match delivery {
        Some(path) => {
            write_value_file(&path, &value).await?;
            path.display().to_string()
        }
        None => {
            // Exact bytes when piped; a trailing newline for terminal reads.
            if std::io::stdout().is_terminal() {
                ctx.shell().human().line(value.as_str())?;
            } else {
                ctx.shell().human().print(value.as_str())?;
            }
            "stdout".to_string()
        }
    };

    Ok(Outcome::SecretExported(SecretExported {
        secret_id,
        value_length: value.len(),
        destination,
        activity_id,
    }))
}

/// Metadata only - the value itself is delivered via `destination` and never
/// rides the outcome.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SecretExported {
    secret_id: String,
    /// Decrypted value size in bytes.
    value_length: usize,
    /// Where the value went: a file path, or `stdout`.
    destination: String,
    activity_id: String,
}

impl Display for SecretExported {
    /// Renders empty (machine-only) when the value went to stdout: the value
    /// itself is the command's human output there, and trailing metadata
    /// would pollute pipes.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.destination == "stdout" {
            return Ok(());
        }
        write!(
            f,
            r#"Secret exported.

Secret ID: {}
Value ({} bytes) written to: {}
Activity ID: {}"#,
            self.secret_id, self.value_length, self.destination, self.activity_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::secrets::test_support::{
        quorum_public_key_hex, test_config, test_ctx, test_json_ctx, test_signing_key,
    };
    use p256::ecdsa::SigningKey;
    use tempfile::TempDir;
    use turnkey_client::generated::ExportSecretsRequest;
    use turnkey_enclave_encrypt::P256Public;
    use turnkey_enclave_encrypt::server::EnclaveEncryptServer;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Respond to `export_secrets` by encrypting `plaintext` to the request's
    /// single-use target key, the way the signer enclave would.
    async fn mount_export_mock(server: &MockServer, signing: SigningKey, plaintext: &'static str) {
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/export_secrets"))
            .respond_with(move |request: &wiremock::Request| {
                let export_request: ExportSecretsRequest =
                    serde_json::from_slice(&request.body).unwrap();
                let secrets = export_request.parameters.unwrap().secrets;
                assert_eq!(secrets.len(), 1);
                assert_eq!(secrets[0].secret_id, "secret-abc");

                let target: P256Public = hex::decode(&secrets[0].target_public_key)
                    .unwrap()
                    .try_into()
                    .unwrap();
                let enclave = EnclaveEncryptServer::from_enclave_auth_key(
                    signing.clone(),
                    "org-1".to_string(),
                    None,
                );
                let payload =
                    serde_json::to_string(&enclave.encrypt(&target, plaintext.as_bytes()).unwrap())
                        .unwrap();

                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "activity": {
                        "type": "ACTIVITY_TYPE_EXPORT_SECRETS",
                        "status": "ACTIVITY_STATUS_COMPLETED",
                        "id": "activity-export",
                        "organizationId": "org-1",
                        "fingerprint": "fingerprint",
                        "result": {
                            "exportSecretsResult": {"secretPayloads": [payload]},
                        }
                    }
                }))
            })
            .expect(1)
            .mount(server)
            .await;
    }

    /// Respond to `list_secrets` with one fixed page of metadata.
    async fn mount_list_mock(server: &MockServer, secrets: serde_json::Value) {
        Mock::given(method("POST"))
            .and(path("/public/v1/query/list_secrets"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"secrets": secrets})),
            )
            .expect(1)
            .mount(server)
            .await;
    }

    fn args(
        id: Option<&str>,
        name: Option<&str>,
        out: Option<PathBuf>,
        signing: &SigningKey,
    ) -> Args {
        Args {
            id: id.map(String::from),
            name: name.map(String::from),
            out,
            plain: false,
            signer_quorum_key_hex: Some(quorum_public_key_hex(signing)),
        }
    }

    #[tokio::test]
    async fn run_exports_by_id_to_piped_stdout_with_metadata_only_outcome() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let outcome = run(
            &mut test_ctx(),
            args(Some("secret-abc"), None, None, &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretExported(exported) = outcome else {
            panic!("expected SecretExported");
        };
        assert_eq!(exported.secret_id, "secret-abc");
        assert_eq!(exported.value_length, "hunter2".len());
        assert_eq!(exported.destination, "stdout");
        assert_eq!(exported.activity_id, "activity-export");
        // Machine-only rendering: the value itself was the stdout payload.
        assert_eq!(exported.to_string(), "");
    }

    #[tokio::test]
    async fn run_writes_the_value_to_a_new_owner_only_file() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let out = dir.path().join("exported");
        let outcome = run(
            &mut test_ctx(),
            args(Some("secret-abc"), None, Some(out.clone()), &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretExported(exported) = outcome else {
            panic!("expected SecretExported");
        };
        assert_eq!(exported.destination, out.display().to_string());
        assert_eq!(exported.value_length, "hunter2".len());

        assert_eq!(tokio::fs::read_to_string(&out).await.unwrap(), "hunter2");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&out).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[tokio::test]
    async fn run_resolves_a_unique_name_to_its_id() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_list_mock(
            &server,
            serde_json::json!([
                {"secretId": "secret-other", "name": "other", "createdAtUnixMs": "1"},
                {"secretId": "secret-abc", "name": "db-password", "createdAtUnixMs": "2"},
            ]),
        )
        .await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let outcome = run(
            &mut test_ctx(),
            args(None, Some("db-password"), None, &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretExported(exported) = outcome else {
            panic!("expected SecretExported");
        };
        assert_eq!(exported.secret_id, "secret-abc");
    }

    /// Run the command with `--name db-password` against a listing and return
    /// the error it must produce.
    async fn name_resolution_error(listing: serde_json::Value) -> anyhow::Error {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_list_mock(&server, listing).await;

        let dir = TempDir::new().unwrap();
        match run(
            &mut test_ctx(),
            args(None, Some("db-password"), None, &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        {
            Err(error) => error,
            Ok(_) => panic!("expected name resolution to fail"),
        }
    }

    #[tokio::test]
    async fn run_rejects_unknown_and_ambiguous_names() {
        let unknown = name_resolution_error(serde_json::json!([])).await;
        assert!(
            unknown.to_string().contains("db-password"),
            "unexpected error: {unknown}"
        );

        let ambiguous = name_resolution_error(serde_json::json!([
            {"secretId": "secret-abc", "name": "db-password", "createdAtUnixMs": "1"},
            {"secretId": "secret-def", "name": "db-password", "createdAtUnixMs": "2"},
        ]))
        .await;
        let message = ambiguous.to_string();
        assert!(message.contains("ambiguous"), "unexpected error: {message}");
        assert!(
            message.contains("secret-abc") && message.contains("secret-def"),
            "unexpected error: {message}"
        );
    }

    #[tokio::test]
    async fn run_requires_out_in_json_mode_before_any_network_work() {
        let signing = test_signing_key();
        let dir = TempDir::new().unwrap();

        // A dead-port config proves the guard fires before network access.
        let error = match run(
            &mut test_json_ctx(),
            args(Some("secret-abc"), None, None, &signing),
            test_config(&dir, "http://127.0.0.1:1"),
        )
        .await
        {
            Err(error) => error,
            Ok(_) => panic!("expected JSON mode without --out to fail"),
        };
        assert!(
            error.to_string().contains("--out"),
            "unexpected error: {error}"
        );
    }
}
