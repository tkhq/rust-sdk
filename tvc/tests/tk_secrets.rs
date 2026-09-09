use assert_cmd::{Command, cargo::cargo_bin_cmd};
use serde_json::{Value, json};
use std::fs;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn cli() -> Command {
    let mut cmd = cargo_bin_cmd!("tk");
    cmd.env_clear().arg("--message-format=json");
    cmd
}

#[test]
fn malformed_recovery_state_does_not_disclose_contents_or_require_auth() {
    let temp = TempDir::new().unwrap();
    let state = temp.path().join("state.json");
    let marker = "synthetic-sensitive-state-marker";
    fs::write(
        &state,
        format!(r#"{{"key_material":"{marker}","version":[]}}"#),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&state, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let out = cli()
        .args(["secret", "resume", "--state-file"])
        .arg(&state)
        .assert()
        .failure();
    for bytes in [&out.get_output().stdout, &out.get_output().stderr] {
        let text = String::from_utf8_lossy(bytes);
        assert!(!text.contains(marker));
        assert!(!text.contains("HOME"));
    }
    let record: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(record["reason"], "command_error");
    assert!(fs::read_to_string(state).unwrap().contains(marker));
}

#[test]
fn export_rejects_stdout_and_existing_files_before_authentication() {
    let temp = TempDir::new().unwrap();
    let state = temp.path().join("state.json");
    let output = temp.path().join("output.bin");
    fs::write(&output, b"existing bytes").unwrap();
    for destination in [std::path::Path::new("-"), output.as_path()] {
        let out = cli()
            .args([
                "secret",
                "export",
                "00000000-0000-4000-8000-000000000001",
                "--output",
            ])
            .arg(destination)
            .arg("--state-file")
            .arg(&state)
            .assert()
            .failure();
        let record: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
        assert_eq!(record["reason"], "command_error");
        assert!(!record["message"].as_str().unwrap().contains("HOME"));
        assert!(!state.exists());
        assert_eq!(fs::read(&output).unwrap(), b"existing bytes");
    }
}

#[tokio::test]
async fn metadata_list_discards_unexpected_secret_payload_fields() {
    let server = MockServer::start().await;
    let marker = "synthetic-unexpected-plaintext";
    Mock::given(method("POST"))
        .and(path("/public/v1/query/list_secrets"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"secrets":[],"plaintext":marker,"secretPayload":marker})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let key = TurnkeyP256ApiKey::generate();
    let out = cli()
        .env(
            "TURNKEY_ORGANIZATION_ID",
            "00000000-0000-4000-8000-000000000001",
        )
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(key.compressed_public_key()),
        )
        .env("TURNKEY_API_PRIVATE_KEY", hex::encode(key.private_key()))
        .arg("--api-base-url")
        .arg(server.uri())
        .args(["secret", "list"])
        .assert()
        .success();
    let record: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    assert_eq!(record["command"], "secret.list");
    assert_eq!(record["data"], json!({"items":[],"nextCursor":null}));
    assert!(out.get_output().stderr.is_empty());
    assert!(!String::from_utf8_lossy(&out.get_output().stdout).contains(marker));
}
