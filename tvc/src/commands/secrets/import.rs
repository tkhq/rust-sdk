//! Secret import command - imports one secret value into Turnkey secret
//! storage.

use crate::client::build_client;
use crate::config::turnkey::Config;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use crate::signer_quorum::signer_quorum_key;
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::mem;
use std::path::{Path, PathBuf};
use tracing::instrument;
use turnkey_client::ActivityResult;
use turnkey_client::generated::ActivityStatus;
use zeroize::{Zeroize, Zeroizing};

pub const LONG_ABOUT: &str = r#"Import one secret value into Turnkey secret storage.

The value is read from a file (or stdin with `--value-file -`) so it never
travels through command-line arguments. One trailing newline is stripped, the
way shell heredocs and `echo` produce values. The value is encrypted to a
single-use Turnkey enclave key before it leaves this process."#;

/// Import one secret value into Turnkey secret storage.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = LONG_ABOUT)]
pub struct Args {
    /// Name for the secret, unique within the organization.
    #[arg(long, value_name = "NAME")]
    name: Option<String>,

    /// File to read the secret value from; pass `-` to read stdin instead.
    #[arg(long, value_name = "PATH")]
    value_file: PathBuf,

    /// Plaintext metadata attached to the secret, as KEY=VALUE. Repeatable.
    /// Policies can read these; the value itself stays encrypted end to end.
    #[arg(long = "property", value_name = "KEY=VALUE", value_parser = parse_static_property)]
    static_properties: Vec<(String, String)>,

    /// Hex signer quorum public key override. Defaults to the Turnkey key for
    /// the active org's environment (production or preprod).
    #[arg(long = "signer-quorum-key", value_name = "HEX")]
    signer_quorum_key_hex: Option<String>,
}

/// Parse one `--property KEY=VALUE` argument.
fn parse_static_property(raw: &str) -> Result<(String, String), String> {
    match raw.split_once('=') {
        Some((key, value)) if !key.is_empty() => Ok((key.to_string(), value.to_string())),
        _ => Err(format!(
            "expected KEY=VALUE with a non-empty key, got '{raw}'"
        )),
    }
}

/// Collect parsed static properties, rejecting duplicate keys.
fn unique_static_properties(properties: Vec<(String, String)>) -> Result<BTreeMap<String, String>> {
    let mut unique = BTreeMap::new();
    for (key, value) in properties {
        ensure!(
            !unique.contains_key(&key),
            "duplicate --property key '{key}'"
        );
        unique.insert(key, value);
    }
    Ok(unique)
}

/// Read the secret value from `path` (`-` = stdin) into a zeroized buffer.
///
/// The value must be non-empty UTF-8; one trailing newline is stripped.
async fn read_value(path: &Path) -> Result<Zeroizing<String>> {
    let mut bytes = if path == Path::new("-") {
        // Blocking is fine here: nothing else runs until the value is read.
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .context("failed to read secret value from stdin")?;
        Zeroizing::new(bytes)
    } else {
        Zeroizing::new(
            tokio::fs::read(path)
                .await
                .with_context(|| format!("failed to read secret value file: {}", path.display()))?,
        )
    };

    // Move the bytes into the string without copying. On the error path, wipe
    // the buffer and report the failure without echoing the contents.
    let mut value = match String::from_utf8(mem::take(&mut *bytes)) {
        Ok(value) => Zeroizing::new(value),
        Err(error) => {
            error.into_bytes().zeroize();
            bail!("secret value in {} is not valid UTF-8", path.display());
        }
    };

    if value.ends_with('\n') {
        value.pop();
        if value.ends_with('\r') {
            value.pop();
        }
    }
    ensure!(
        !value.is_empty(),
        "secret value in {} is empty",
        path.display()
    );
    Ok(value)
}

/// Run the secret import command.
#[instrument(skip_all)]
pub async fn run(_ctx: &mut StdCtx, args: Args, config: Config) -> Result<Outcome> {
    let Args {
        name,
        value_file,
        static_properties,
        signer_quorum_key_hex,
    } = args;
    let static_properties = unique_static_properties(static_properties)?;
    let mut value = read_value(&value_file).await?;

    let auth = build_client(&config).await?;
    let signer = signer_quorum_key(&auth.api_base_url, signer_quorum_key_hex.as_deref())?;

    // Hand the client the buffer itself; it wipes the plaintext once the
    // encrypted payload has been produced.
    let plaintext = mem::take(&mut *value);
    let ActivityResult {
        result: secret_id,
        activity_id,
        status,
        app_proofs: _,
    } = auth
        .client
        .import_secret(
            auth.org_id,
            name.clone(),
            plaintext,
            static_properties,
            &signer,
        )
        .await
        .context("failed to import secret")?;

    Ok(Outcome::SecretImported(SecretImported {
        secret_id,
        name,
        activity_id,
        activity_status: status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretImported {
    secret_id: String,
    name: Option<String>,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for SecretImported {
    fn default() -> Self {
        Self {
            secret_id: String::default(),
            name: None,
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Display for SecretImported {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Secret imported.

Secret ID: {}
Name: {}
Activity ID: {}
Activity Status: {}"#,
            self.secret_id,
            self.name.as_deref().unwrap_or("(none)"),
            self.activity_id,
            self.activity_status.as_str_name()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::secrets::test_support::{
        quorum_public_key_hex, test_config, test_ctx, test_signing_key,
    };
    use tempfile::TempDir;
    use turnkey_enclave_encrypt::server::EnclaveEncryptServer;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn static_property_parses_key_and_value() {
        assert_eq!(
            parse_static_property("environment=demo").unwrap(),
            ("environment".to_string(), "demo".to_string())
        );
    }

    #[test]
    fn static_property_value_may_contain_equals() {
        assert_eq!(
            parse_static_property("query=a=b").unwrap(),
            ("query".to_string(), "a=b".to_string())
        );
    }

    #[test]
    fn static_property_without_equals_is_rejected() {
        let error = parse_static_property("environment").unwrap_err();
        assert!(error.contains("KEY=VALUE"), "unexpected error: {error}");
    }

    #[test]
    fn static_property_with_empty_key_is_rejected() {
        let error = parse_static_property("=demo").unwrap_err();
        assert!(error.contains("KEY=VALUE"), "unexpected error: {error}");
    }

    #[test]
    fn duplicate_static_property_keys_are_rejected() {
        let error = unique_static_properties(vec![
            ("environment".to_string(), "demo".to_string()),
            ("environment".to_string(), "prod".to_string()),
        ])
        .unwrap_err();
        assert!(
            error.to_string().contains("environment"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn read_value_strips_one_trailing_newline() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("value");
        tokio::fs::write(&file, "hunter2\n").await.unwrap();
        assert_eq!(read_value(&file).await.unwrap().as_str(), "hunter2");

        tokio::fs::write(&file, "hunter2\r\n").await.unwrap();
        assert_eq!(read_value(&file).await.unwrap().as_str(), "hunter2");

        // Only one newline is stripped; inner whitespace is preserved.
        tokio::fs::write(&file, " hunter2 \n\n").await.unwrap();
        assert_eq!(read_value(&file).await.unwrap().as_str(), " hunter2 \n");
    }

    #[tokio::test]
    async fn read_value_rejects_empty_values() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("value");
        tokio::fs::write(&file, "\n").await.unwrap();
        let error = read_value(&file).await.unwrap_err();
        assert!(error.to_string().contains("empty"), "unexpected: {error}");
    }

    #[tokio::test]
    async fn read_value_rejects_invalid_utf8_without_echoing_it() {
        let dir = TempDir::new().unwrap();
        let file = dir.path().join("value");
        tokio::fs::write(&file, [0xff, 0xfe, 0x00]).await.unwrap();
        let error = read_value(&file).await.unwrap_err();
        assert!(error.to_string().contains("UTF-8"), "unexpected: {error}");
    }

    #[tokio::test]
    async fn read_value_reports_missing_files() {
        let dir = TempDir::new().unwrap();
        let error = read_value(&dir.path().join("nope")).await.unwrap_err();
        assert!(
            error.to_string().contains("nope"),
            "unexpected error: {error}"
        );
    }

    #[tokio::test]
    async fn run_imports_the_file_value_end_to_end() {
        let signing = test_signing_key();
        let server = MockServer::start().await;

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            signing.clone(),
            "org-1".to_string(),
            Some(String::new()),
        );
        let target = serde_json::to_string(&enclave.publish_target().unwrap()).unwrap();

        Mock::given(method("POST"))
            .and(path("/public/v1/submit/init_import_secrets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activity": {
                    "type": "ACTIVITY_TYPE_INIT_IMPORT_SECRETS",
                    "status": "ACTIVITY_STATUS_COMPLETED",
                    "id": "activity-init",
                    "organizationId": "org-1",
                    "fingerprint": "fingerprint",
                    "result": {
                        "initImportSecretsResult": {
                            "enclaveTargetMessages": [target],
                        }
                    }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/public/v1/submit/import_secrets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activity": {
                    "type": "ACTIVITY_TYPE_IMPORT_SECRETS",
                    "status": "ACTIVITY_STATUS_COMPLETED",
                    "id": "activity-import",
                    "organizationId": "org-1",
                    "fingerprint": "fingerprint",
                    "result": {
                        "importSecretsResult": {"secretIds": ["secret-abc"]}
                    }
                }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let dir = TempDir::new().unwrap();
        let value_file = dir.path().join("value");
        tokio::fs::write(&value_file, "hunter2\n").await.unwrap();

        let args = Args {
            name: Some("db-password".to_string()),
            value_file,
            static_properties: vec![("environment".to_string(), "demo".to_string())],
            signer_quorum_key_hex: Some(quorum_public_key_hex(&signing)),
        };

        let outcome = run(&mut test_ctx(), args, test_config(&dir, &server.uri()))
            .await
            .unwrap();
        let Outcome::SecretImported(imported) = outcome else {
            panic!("expected SecretImported");
        };
        assert_eq!(imported.secret_id, "secret-abc");
        assert_eq!(imported.name.as_deref(), Some("db-password"));
        assert_eq!(imported.activity_id, "activity-import");
        assert_eq!(imported.activity_status, ActivityStatus::Completed);

        // The import request carried the name and static properties, and the
        // enclave can decrypt exactly the newline-stripped file value.
        let requests = server.received_requests().await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&requests[1].body).unwrap();
        let secret = &body["parameters"]["secrets"][0];
        assert_eq!(secret["name"], "db-password");
        assert_eq!(
            secret["staticProperties"],
            serde_json::json!([{"key": "environment", "value": "demo"}])
        );
        let payload = secret["secretPayload"].as_str().unwrap();
        assert_eq!(
            enclave
                .into_recv()
                .decrypt(&serde_json::from_str(payload).unwrap())
                .unwrap(),
            b"hunter2",
        );
    }
}
