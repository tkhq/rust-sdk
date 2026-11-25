mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use qos_p256::P256Pair;
use serde_json::{Value, json};
use std::{fs, io::ErrorKind, net::TcpListener, path::Path, process::Output};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const ORGANIZATION_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

fn recipients() -> String {
    (0..2)
        .map(|_| hex::encode(&P256Pair::generate().unwrap().public_key().to_bytes()[..65]))
        .collect::<Vec<_>>()
        .join(",")
}

fn quorum_key_args(recipients: &str) -> [&str; 6] {
    [
        "keys",
        "create-quorum-key",
        "--threshold",
        "2",
        "--operator-encrypt-keys",
        recipients,
    ]
}

fn run(home: &Path, url: &str, format: &str, args: &[&str]) -> Output {
    let api_key = TurnkeyP256ApiKey::generate();
    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", home)
        .env("TVC_API_BASE_URL", url)
        .env("TVC_ORG_ID", ORGANIZATION_ID)
        .env(
            "TVC_API_KEY_PUBLIC",
            hex::encode(api_key.compressed_public_key()),
        )
        .env("TVC_API_KEY_PRIVATE", hex::encode(api_key.private_key()))
        .args(["--message-format", format])
        .args(args)
        .output()
        .unwrap()
}

fn assert_refused_before_post(home: &Path, args: &[&str]) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let output = run(home, &url, "json", args);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    let error: Value = serde_json::from_str(stdout).unwrap();
    assert_eq!(error["reason"], "command_error");
    assert_eq!(error["code"], "proof_verification_failed");
    assert!(error.get("quorumPublicKey").is_none());
    assert!(error.get("encryptPublicKey").is_none());
    assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
}

fn assert_refusal_creates_no_config(args: &[&str]) {
    let home = TempDir::new().unwrap();
    assert_refused_before_post(home.path(), args);
    assert!(!home.path().join(".config").exists());
}

fn assert_refusal_preserves_config(args: &[&str]) {
    let home = TempDir::new().unwrap();
    common::write_profiles_config(home.path(), &[("active", ORGANIZATION_ID)], Some("active"));
    let path = home.path().join(".config/turnkey/tvc.config.toml");
    let baseline = fs::read(&path).unwrap();

    assert_refused_before_post(home.path(), args);
    assert_eq!(fs::read(path).unwrap(), baseline);
}

#[test]
fn operator_creation_refuses_unknown_environment_without_creating_config() {
    assert_refusal_creates_no_config(&["operator", "create"]);
}

#[test]
fn operator_creation_refuses_unknown_environment_and_preserves_config() {
    assert_refusal_preserves_config(&["operator", "create"]);
}

#[test]
fn quorum_key_creation_refuses_unknown_environment_without_creating_config() {
    assert_refusal_creates_no_config(&quorum_key_args(&recipients()));
}

#[test]
fn quorum_key_creation_refuses_unknown_environment_and_preserves_config() {
    assert_refusal_preserves_config(&quorum_key_args(&recipients()));
}

const ACTIVITY_ID: &str = "synthetic-activity";
const GET_ACTIVITY: &str = "/public/v1/query/get_activity";
const SKIP: &str = "--dangerous-skip-verification";
const OPERATOR_LOOKUP: [&str; 6] = [
    "operator",
    "create",
    "verify-result",
    "--activity-id",
    ACTIVITY_ID,
    SKIP,
];
const QUORUM_LOOKUP: [&str; 6] = [
    "keys",
    "create-quorum-key",
    "verify-result",
    "--activity-id",
    ACTIVITY_ID,
    SKIP,
];

fn operator_activity() -> Value {
    let public = P256Pair::generate().unwrap().public_key().to_bytes();
    json!({
        "id": ACTIVITY_ID, "organizationId": ORGANIZATION_ID, "fingerprint": "fixture",
        "type": "ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status": "ACTIVITY_STATUS_COMPLETED",
        "result": {"createTvcOperatorResult": {
            "walletId":"11111111-1111-4111-8111-111111111111",
            "operatorId":"22222222-2222-4222-8222-222222222222",
            "encryptPublicKey":hex::encode(&public[..65]), "signPublicKey":hex::encode(&public[65..])
        }}
    })
}

fn quorum_activity() -> Value {
    let public = P256Pair::generate().unwrap().public_key().to_bytes();
    json!({
        "id": ACTIVITY_ID, "organizationId": ORGANIZATION_ID, "fingerprint": "fixture",
        "type": "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY", "status": "ACTIVITY_STATUS_COMPLETED",
        "result": {"createTvcQuorumKeyResult": {
            "quorumKeyId":"33333333-3333-4333-8333-333333333333", "quorumPublicKey":hex::encode(public),
            "shareIds":["44444444-4444-4444-8444-444444444444", "55555555-5555-4555-8555-555555555555"]
        }}
    })
}

async fn run_with_activity(
    endpoint: &str,
    activity: &Value,
    format: &str,
    args: &[&str],
) -> Output {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(endpoint))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": activity})))
        .mount(&server)
        .await;
    let home = TempDir::new().unwrap();
    common::write_profiles_config(home.path(), &[("active", ORGANIZATION_ID)], Some("active"));
    let config_path = home.path().join(".config/turnkey/tvc.config.toml");
    let profile = fs::read_to_string(&config_path)
        .unwrap()
        .replace(common::LOCAL_API_BASE_URL, &server.uri());
    fs::write(&config_path, &profile).unwrap();

    let output = run(home.path(), &server.uri(), format, args);
    assert_eq!(fs::read_to_string(config_path).unwrap(), profile);
    let requests = server.received_requests().await.unwrap();
    let [only] = requests.as_slice() else {
        panic!(
            "expected exactly one request to {endpoint}, got {}. CLI output: {output:?}",
            requests.len()
        );
    };
    assert_eq!(only.method.as_str(), "POST");
    assert_eq!(only.url.path(), endpoint);
    if endpoint == GET_ACTIVITY {
        assert_eq!(
            only.body_json::<Value>().unwrap(),
            json!({
                "organizationId": ORGANIZATION_ID, "activityId": ACTIVITY_ID
            })
        );
    }
    output
}

fn success_json(output: Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_operator_output(output: Output, activity: &Value) {
    let value = success_json(output);
    let result = &activity["result"]["createTvcOperatorResult"];
    assert_eq!(value["reason"], "unverified_operator_result");
    assert_eq!(value["verificationStatus"], "unverified");
    assert_eq!(value["activityId"], ACTIVITY_ID);
    assert_eq!(value["walletId"], result["walletId"]);
    assert_eq!(value["operatorId"], result["operatorId"]);
    assert_eq!(value["encryptPublicKey"], result["encryptPublicKey"]);
    assert_eq!(value["signPublicKey"], result["signPublicKey"]);
    assert_eq!(
        value["compositePublicKey"],
        format!(
            "{}{}",
            result["encryptPublicKey"].as_str().unwrap(),
            result["signPublicKey"].as_str().unwrap()
        )
    );
}

fn assert_quorum_output(output: Output, activity: &Value) {
    let value = success_json(output);
    let result = &activity["result"]["createTvcQuorumKeyResult"];
    assert_eq!(value["reason"], "unverified_quorum_key_result");
    assert_eq!(value["verificationStatus"], "unverified");
    assert_eq!(value["activityId"], ACTIVITY_ID);
    assert_eq!(value["quorumKeyId"], result["quorumKeyId"]);
    assert_eq!(value["quorumPublicKey"], result["quorumPublicKey"]);
    assert_eq!(value["shareIds"], result["shareIds"]);
}

#[tokio::test]
async fn skipped_operator_creation_reports_unsigned_keys_without_registering_them() {
    let activity = operator_activity();
    let output = run_with_activity(
        "/public/v1/submit/create_tvc_operator",
        &activity,
        "json",
        &["operator", "create", SKIP],
    )
    .await;
    assert_operator_output(output, &activity);
}

#[tokio::test]
async fn skipped_quorum_creation_reports_unsigned_keys() {
    let activity = quorum_activity();
    let recipients = recipients();
    let create = quorum_key_args(&recipients);
    let args = [create.as_slice(), &[SKIP]].concat();
    let output = run_with_activity(
        "/public/v1/submit/create_tvc_quorum_key",
        &activity,
        "json",
        &args,
    )
    .await;
    assert_quorum_output(output, &activity);
}

#[tokio::test]
async fn skipped_operator_lookup_reports_unsigned_keys_without_a_checkpoint() {
    let activity = operator_activity();
    let output = run_with_activity(GET_ACTIVITY, &activity, "json", &OPERATOR_LOOKUP).await;
    assert_operator_output(output, &activity);
}

#[tokio::test]
async fn skipped_quorum_lookup_reports_unsigned_keys_without_a_checkpoint() {
    let activity = quorum_activity();
    let output = run_with_activity(GET_ACTIVITY, &activity, "json", &QUORUM_LOOKUP).await;
    assert_quorum_output(output, &activity);
}

fn assert_human_key_is_unverified(output: Output, key: &Value) {
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.to_lowercase().contains("unverified"), "{stdout}");
    assert!(stdout.contains(key.as_str().unwrap()), "{stdout}");
}

#[tokio::test]
async fn skipped_quorum_lookup_labels_the_key_unverified_in_human_output() {
    let activity = quorum_activity();
    let output = run_with_activity(GET_ACTIVITY, &activity, "human", &QUORUM_LOOKUP).await;
    assert_human_key_is_unverified(
        output,
        &activity["result"]["createTvcQuorumKeyResult"]["quorumPublicKey"],
    );
}

#[tokio::test]
async fn skipped_operator_lookup_labels_the_keys_unverified_in_human_output() {
    let activity = operator_activity();
    let output = run_with_activity(GET_ACTIVITY, &activity, "human", &OPERATOR_LOOKUP).await;
    assert_human_key_is_unverified(
        output,
        &activity["result"]["createTvcOperatorResult"]["encryptPublicKey"],
    );
}

async fn assert_operator_lookup_refused(field: &str, replacement: &str) {
    let mut activity = operator_activity();
    activity[field] = json!(replacement);
    let output = run_with_activity(GET_ACTIVITY, &activity, "json", &OPERATOR_LOOKUP).await;
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let error: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(error["reason"], "command_error");
    assert!(error.get("encryptPublicKey").is_none());
    assert!(error.get("quorumPublicKey").is_none());
}

#[tokio::test]
async fn skipped_lookup_refuses_another_organizations_activity() {
    assert_operator_lookup_refused("organizationId", "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb").await;
}

#[tokio::test]
async fn skipped_lookup_refuses_a_different_activity_id() {
    assert_operator_lookup_refused("id", "some-other-activity").await;
}

#[tokio::test]
async fn skipped_lookup_refuses_a_different_activity_type() {
    assert_operator_lookup_refused("type", "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY").await;
}

#[tokio::test]
async fn skipped_lookup_refuses_keys_from_a_failed_activity() {
    assert_operator_lookup_refused("status", "ACTIVITY_STATUS_FAILED").await;
}
