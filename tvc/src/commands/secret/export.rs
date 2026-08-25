//! Secret export command - exports one secret value from Turnkey secret
//! storage.

use super::signer_quorum_public_key;
use crate::client::build_client;
use crate::config::turnkey::Config;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use anyhow::{Context, Result, ensure};
use clap::Args as ClapArgs;
use serde::{Serialize, Serializer};
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;
use tracing::instrument;
use turnkey_client::ActivityResult;
use turnkey_client::generated::ActivityStatus;
use zeroize::Zeroizing;

pub const LONG_ABOUT: &str = r#"Export one secret value from Turnkey secret storage.

The value is decrypted with a single-use transport key inside this process.
By default it is included in the command output; pass --output-file to write
it to a new file readable only by the current user instead."#;

/// Export one secret value from Turnkey secret storage.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// ID of the secret to export.
    #[arg(long, value_name = "SECRET_ID")]
    secret_id: String,

    /// Write the value to this new file (created with mode 0600) instead of
    /// including it in the command output. Fails if the file already exists.
    #[arg(long, value_name = "PATH")]
    output_file: Option<PathBuf>,

    /// Hex signer quorum public key override. Defaults to the Turnkey key for
    /// the active org's environment (production or preprod).
    #[arg(long, value_name = "HEX")]
    signer_public_key: Option<String>,
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
pub async fn run(_ctx: &mut StdCtx, args: Args, config: Config) -> Result<Outcome> {
    let Args {
        secret_id,
        output_file,
        signer_public_key,
    } = args;

    let auth = build_client(&config).await?;
    let signer = signer_quorum_public_key(&auth.api_base_url, signer_public_key.as_deref())?;

    let ActivityResult {
        result: plaintexts,
        activity_id,
        status,
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

    let value = match &output_file {
        Some(path) => {
            write_value_file(path, &value).await?;
            None
        }
        None => Some(value),
    };

    Ok(Outcome::SecretExported(SecretExported {
        secret_id,
        value,
        output_file,
        activity_id,
        activity_status: status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretExported {
    secret_id: String,
    /// The decrypted value; absent when it was written to `output_file`.
    #[serde(serialize_with = "serialize_plaintext")]
    value: Option<Zeroizing<String>>,
    output_file: Option<PathBuf>,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// `Zeroizing` intentionally implements no serde traits; the exported value
/// serializes through its borrowed contents.
fn serialize_plaintext<S: Serializer>(
    value: &Option<Zeroizing<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(value) => serializer.serialize_some(value.as_str()),
        None => serializer.serialize_none(),
    }
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for SecretExported {
    fn default() -> Self {
        Self {
            secret_id: String::default(),
            value: None,
            output_file: None,
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Display for SecretExported {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Secret exported.

Secret ID: {}
Activity ID: {}
Activity Status: {}"#,
            self.secret_id,
            self.activity_id,
            self.activity_status.as_str_name()
        )?;
        match (&self.output_file, &self.value) {
            (Some(path), _) => write!(f, "\nValue written to: {}", path.display()),
            (None, Some(value)) => write!(f, "\n\n{}", value.as_str()),
            (None, None) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::secret::test_support::{
        quorum_public_key_hex, test_config, test_ctx, test_signing_key,
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

    fn args(secret_id: &str, output_file: Option<PathBuf>, signing: &SigningKey) -> Args {
        Args {
            secret_id: secret_id.to_string(),
            output_file,
            signer_public_key: Some(quorum_public_key_hex(signing)),
        }
    }

    #[tokio::test]
    async fn run_exports_and_returns_the_decrypted_value() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let outcome = run(
            &mut test_ctx(),
            args("secret-abc", None, &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretExported(exported) = outcome else {
            panic!("expected SecretExported");
        };
        assert_eq!(exported.secret_id, "secret-abc");
        assert_eq!(exported.value.as_ref().unwrap().as_str(), "hunter2");
        assert_eq!(exported.output_file, None);
        assert_eq!(exported.activity_id, "activity-export");
        assert_eq!(exported.activity_status, ActivityStatus::Completed);
    }

    #[tokio::test]
    async fn run_writes_the_value_to_a_new_owner_only_file() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let output_file = dir.path().join("exported");
        let outcome = run(
            &mut test_ctx(),
            args("secret-abc", Some(output_file.clone()), &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        .unwrap();

        let Outcome::SecretExported(exported) = outcome else {
            panic!("expected SecretExported");
        };
        assert!(exported.value.is_none());
        assert_eq!(exported.output_file, Some(output_file.clone()));

        assert_eq!(
            tokio::fs::read_to_string(&output_file).await.unwrap(),
            "hunter2"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&output_file)
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[tokio::test]
    async fn run_refuses_to_overwrite_an_existing_output_file() {
        let signing = test_signing_key();
        let server = MockServer::start().await;
        mount_export_mock(&server, signing.clone(), "hunter2").await;

        let dir = TempDir::new().unwrap();
        let output_file = dir.path().join("exported");
        tokio::fs::write(&output_file, "already here")
            .await
            .unwrap();

        let error = match run(
            &mut test_ctx(),
            args("secret-abc", Some(output_file.clone()), &signing),
            test_config(&dir, &server.uri()),
        )
        .await
        {
            Err(error) => error,
            Ok(_) => panic!("expected the existing file to be refused"),
        };

        assert!(
            error.to_string().contains("exported"),
            "unexpected error: {error}"
        );
        assert_eq!(
            tokio::fs::read_to_string(&output_file).await.unwrap(),
            "already here"
        );
    }

    #[test]
    fn exported_value_serializes_as_a_plain_string() {
        let exported = SecretExported {
            secret_id: "secret-abc".to_string(),
            value: Some(Zeroizing::new("hunter2".to_string())),
            output_file: None,
            activity_id: "activity-export".to_string(),
            activity_status: ActivityStatus::Completed,
        };
        assert_eq!(
            serde_json::to_value(&exported).unwrap(),
            serde_json::json!({
                "secretId": "secret-abc",
                "value": "hunter2",
                "outputFile": null,
                "activityId": "activity-export",
                "activityStatus": "ACTIVITY_STATUS_COMPLETED",
            })
        );
    }
}
