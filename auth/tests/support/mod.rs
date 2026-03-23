//! Shared integration-test harness for the `auth` CLI.
//!
//! Use these helpers when a test needs to spawn the binary, configure a
//! temporary auth environment, stand up a mocked Turnkey API, or generate SSH
//! fixtures for end-to-end signing verification.
//!
//! Prefer lower-level unit tests for pure command or encoding behavior. Use this
//! harness when the process boundary, environment resolution, or on-disk
//! artifacts are part of what the test is validating.

#![allow(dead_code)]

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;
use tempfile::{tempdir, TempDir};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use auth::ssh;

pub struct AuthTestEnv {
    temp: TempDir,
    config_path: PathBuf,
}

impl AuthTestEnv {
    pub fn new() -> Self {
        let temp = tempdir().expect("temp dir should exist");
        let config_path = temp.path().join("auth.toml");
        Self { temp, config_path }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn path(&self) -> &Path {
        self.temp.path()
    }

    pub fn command(&self) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
        cmd.env("TURNKEY_AUTH_CONFIG_PATH", &self.config_path);
        cmd
    }

    pub fn command_without_auth_env(&self) -> Command {
        let mut cmd = self.command();
        cmd.env_remove("TURNKEY_ORGANIZATION_ID")
            .env_remove("TURNKEY_API_PUBLIC_KEY")
            .env_remove("TURNKEY_API_PRIVATE_KEY")
            .env_remove("TURNKEY_PRIVATE_KEY_ID")
            .env_remove("TURNKEY_API_BASE_URL");
        cmd
    }

    pub fn turnkey_command(&self, server: &MockServer) -> Command {
        let api_key = TurnkeyP256ApiKey::generate();
        let mut cmd = self.command_without_auth_env();
        cmd.env("TURNKEY_ORGANIZATION_ID", "org-id")
            .env(
                "TURNKEY_API_PUBLIC_KEY",
                hex::encode(api_key.compressed_public_key()),
            )
            .env(
                "TURNKEY_API_PRIVATE_KEY",
                hex::encode(api_key.private_key()),
            )
            .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
            .env("TURNKEY_API_BASE_URL", server.uri());
        cmd
    }
}

pub struct SshFixture {
    pub key_path: PathBuf,
    pub public_key_path: PathBuf,
    pub payload_path: PathBuf,
    pub allowed_signers_path: PathBuf,
    pub public_key_line: String,
    pub parsed_public_key: ssh::ParsedPublicKey,
}

pub async fn start_mock_turnkey_server() -> MockServer {
    MockServer::start().await
}

pub async fn mount_get_private_key_mock(server: &MockServer, public_key: &str) {
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_private_key"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "privateKey": {
                "privateKeyId": "pk-id",
                "publicKey": public_key,
                "privateKeyName": "git signer",
                "curve": "CURVE_ED25519",
                "addresses": [],
                "privateKeyTags": [],
                "createdAt": null,
                "updatedAt": null,
                "exported": false,
                "imported": false
            }
        })))
        .mount(server)
        .await;
}

pub async fn mount_sign_raw_payload_mock(server: &MockServer, raw_signature: &[u8]) {
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/sign_raw_payload"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "id": "activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
                "status": "ACTIVITY_STATUS_COMPLETED",
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "result": {
                    "signRawPayloadResult": {
                        "r": hex::encode(&raw_signature[..32]),
                        "s": hex::encode(&raw_signature[32..]),
                        "v": ""
                    }
                }
            }
        })))
        .mount(server)
        .await;
}

pub fn create_ssh_fixture(root: &Path, payload: &[u8]) -> SshFixture {
    let key_path = root.join("id_ed25519");
    let public_key_path = root.join("id_ed25519.pub");
    let payload_path = root.join("payload.txt");
    let allowed_signers_path = root.join("allowed_signers");

    let status = StdCommand::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&key_path)
        .status()
        .expect("ssh-keygen should run");
    assert!(status.success());

    fs::write(&payload_path, payload).expect("payload should be written");

    let public_key_line = fs::read_to_string(&public_key_path).expect("public key should exist");
    let parsed_public_key =
        ssh::parse_public_key_line(&public_key_line).expect("public key should parse");

    SshFixture {
        key_path,
        public_key_path,
        payload_path,
        allowed_signers_path,
        public_key_line,
        parsed_public_key,
    }
}

pub fn extract_raw_signature(key_path: &Path, payload_path: &Path) -> Vec<u8> {
    let status = StdCommand::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "git", "-f"])
        .arg(key_path)
        .arg(payload_path)
        .status()
        .expect("ssh-keygen sign should run");
    assert!(status.success());

    let signature_path = payload_path.with_extension("txt.sig");
    let armored = fs::read_to_string(signature_path).expect("signature should exist");
    parse_raw_signature_from_armored(&armored)
}

pub fn verify_signature(
    allowed_signers_path: &Path,
    public_key_line: &str,
    payload_path: &Path,
    signature_path: &Path,
) {
    fs::write(
        allowed_signers_path,
        format!("git {}", public_key_line.trim()),
    )
    .expect("allowed signers should be written");

    let status = StdCommand::new("ssh-keygen")
        .args(["-Y", "verify", "-n", "git", "-I", "git", "-f"])
        .arg(allowed_signers_path)
        .arg("-s")
        .arg(signature_path)
        .stdin(File::open(payload_path).expect("payload should open"))
        .status()
        .expect("ssh-keygen verify should run");

    assert!(status.success(), "ssh-keygen should verify auth output");
}

fn parse_raw_signature_from_armored(armored: &str) -> Vec<u8> {
    use base64::Engine;

    let base64 = armored
        .lines()
        .filter(|line| !line.starts_with("-----"))
        .collect::<String>();
    let blob = base64::engine::general_purpose::STANDARD
        .decode(base64)
        .expect("signature body should decode");

    let mut cursor = blob.as_slice();
    assert_eq!(&cursor[..6], b"SSHSIG");
    cursor = &cursor[6 + 4..];

    let _public_key = read_ssh_bytes(&mut cursor);
    let _namespace = read_ssh_bytes(&mut cursor);
    let _reserved = read_ssh_bytes(&mut cursor);
    let _hash_algorithm = read_ssh_bytes(&mut cursor);
    let signature_blob = read_ssh_bytes(&mut cursor);

    let mut signature_cursor = signature_blob.as_slice();
    let algorithm = read_ssh_bytes(&mut signature_cursor);
    assert_eq!(std::str::from_utf8(&algorithm).unwrap(), "ssh-ed25519");
    read_ssh_bytes(&mut signature_cursor)
}

fn read_ssh_bytes(cursor: &mut &[u8]) -> Vec<u8> {
    let length = u32::from_be_bytes(cursor[..4].try_into().expect("ssh length should exist"));
    *cursor = &cursor[4..];
    let length = length as usize;
    let value = cursor[..length].to_vec();
    *cursor = &cursor[length..];
    value
}
