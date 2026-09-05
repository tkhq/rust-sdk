use assert_cmd::{Command, cargo::cargo_bin_cmd};
use serde_json::{Value, json};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_string, method, path},
};

const ORG: &str = "00000000-0000-4000-8000-000000000001";
fn cli() -> Command {
    let mut cmd = cargo_bin_cmd!("tk");
    for name in [
        "HOME",
        "TK_CONFIG",
        "TK_PROFILE",
        "TURNKEY_ORGANIZATION_ID",
        "TURNKEY_API_PUBLIC_KEY",
        "TURNKEY_API_PRIVATE_KEY",
        "TURNKEY_API_BASE_URL",
        "TVC_ORG_ID",
        "TVC_API_KEY_PUBLIC",
        "TVC_API_KEY_PRIVATE",
        "TVC_API_BASE_URL",
    ] {
        cmd.env_remove(name);
    }
    cmd.arg("--message-format=json");
    cmd
}
fn authed(base: &str) -> Command {
    let mut cmd = cli();
    let key = TurnkeyP256ApiKey::generate();
    cmd.env("TURNKEY_ORGANIZATION_ID", ORG)
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(key.compressed_public_key()),
        )
        .env("TURNKEY_API_PRIVATE_KEY", hex::encode(key.private_key()))
        .arg("--api-base-url")
        .arg(base);
    cmd
}
fn record(cmd: &mut Command, code: i32) -> Value {
    let result = cmd.assert().code(code);
    assert!(result.get_output().stderr.is_empty());
    serde_json::from_slice(&result.get_output().stdout).unwrap()
}
#[test]
fn malformed_local_inputs_are_rejected_before_credentials() {
    for args in [
        vec![
            "request",
            "--path",
            "/public/v1/query/whoami",
            "--body",
            "{",
        ],
        vec!["user", "create", "--input-json", "{"],
        vec!["wallet", "create", "--input-json", "{"],
        vec!["sign", "payload", "--input-json", "{"],
    ] {
        let result = record(cli().args(args), 1);
        assert!(!result["message"].as_str().unwrap().contains("HOME"));
    }
}
#[test]
fn offline_generation_needs_no_identity_and_never_overwrites() {
    let temp = TempDir::new().unwrap();
    let key = temp.path().join("key.json");
    let result = record(cli().args(["api-key", "generate", "--output"]).arg(&key), 0);
    assert_eq!(result["command"], "api-key.generate");
    let bytes = std::fs::read(&key).unwrap();
    let stored: Value = serde_json::from_slice(&bytes).unwrap();
    assert!(
        !result
            .to_string()
            .contains(stored["private_key"].as_str().unwrap())
    );
    record(cli().args(["api-key", "generate", "--output"]).arg(&key), 1);
    assert_eq!(std::fs::read(key).unwrap(), bytes);
}
#[tokio::test]
async fn signed_request_preserves_body_and_pending_vs_rejected_exit_codes() {
    let server = MockServer::start().await;
    let body = format!("{{\n  \"organizationId\": \"{ORG}\"\n}}\n");
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/example"))
        .and(body_string(&body))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"activity":{"id":"pending-id","status":"ACTIVITY_STATUS_CONSENSUS_NEEDED"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let pending = record(
        authed(&server.uri()).args([
            "request",
            "--path",
            "/public/v1/submit/example",
            "--body",
            &body,
        ]),
        0,
    );
    assert_eq!(pending["status"], "pending");
    assert_eq!(pending["activity"]["id"], "pending-id");
    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_activity"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"activity":{"id":"rejected-id","status":"ACTIVITY_STATUS_REJECTED"}}),
        ))
        .expect(2)
        .mount(&server)
        .await;
    let inspected = record(
        authed(&server.uri()).args(["activity", "get", "rejected-id"]),
        0,
    );
    assert_eq!(inspected["activity"]["status"], "ACTIVITY_STATUS_REJECTED");
    let waited = record(
        authed(&server.uri()).args(["activity", "wait", "rejected-id"]),
        1,
    );
    assert_eq!(waited["status"], "rejected");
}
#[tokio::test]
async fn shared_resource_and_wallet_queries_use_selected_identity() {
    let server = MockServer::start().await;
    for (args, endpoint, response, command) in [
        (
            vec!["user", "list"],
            "list_users",
            json!({"users":[]}),
            "user.list",
        ),
        (
            vec!["policy", "list"],
            "list_policies",
            json!({"policies":[]}),
            "policy.list",
        ),
        (
            vec!["wallet", "list"],
            "list_wallets",
            json!({"wallets":[]}),
            "wallet.list",
        ),
    ] {
        Mock::given(method("POST"))
            .and(path(format!("/public/v1/query/{endpoint}")))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .expect(1)
            .mount(&server)
            .await;
        let result = record(authed(&server.uri()).args(args), 0);
        assert_eq!(result["command"], command);
    }
}

#[tokio::test]
async fn payload_signing_preserves_pending_activity_without_resubmission() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/public/v1/submit/sign_raw_payload"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"activity":{"id":"sign-id","status":"ACTIVITY_STATUS_CONSENSUS_NEEDED"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let result = record(authed(&server.uri()).args(["sign", "payload", "--input-json", r#"{"signWith":"0x1234","payload":"abcd","encoding":"PAYLOAD_ENCODING_HEXADECIMAL","hashFunction":"HASH_FUNCTION_NO_OP"}"#]), 0);
    assert_eq!(result["command"], "sign.payload");
    assert_eq!(result["status"], "pending");
    assert_eq!(result["activity"]["id"], "sign-id");
}
