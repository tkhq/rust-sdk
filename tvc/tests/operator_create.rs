use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
};
use uuid::Uuid;

const WALLET_ID: &str = "22222222-2222-4222-8222-222222222222";
const ACCOUNT_PATH: &str = "m/5527107'/0'/0'";

/// Write a config whose active org already has a hosted operator backed by
/// `WALLET_ID` at `ACCOUNT_PATH`.
fn write_config_with_hosted_operator(home: &TempDir) {
    let turnkey_dir = home.path().join(".config").join("turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: turnkey_dir.join("orgs/test/api_key.json"),
                api_base_url: "http://127.0.0.1:1".to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![OperatorRecord {
                    name: "tvc-operator".to_string(),
                    kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                        operator_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111")
                            .unwrap(),
                        wallet_id: Uuid::parse_str(WALLET_ID).unwrap(),
                        path: ACCOUNT_PATH.to_string(),
                        encrypt_public_key: format!("04{}", "11".repeat(64)),
                        sign_public_key: format!("04{}", "22".repeat(64)),
                        extra: toml::Table::new(),
                    }),
                }],
                extra: toml::Table::new(),
            },
        )]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// Sad path: the same wallet + account path as a saved hosted operator means
/// the server would derive identical keys; non-interactive mode must error
/// before any network call.
#[test]
fn operator_create_errors_on_key_collision_non_interactively() {
    let temp = TempDir::new().unwrap();
    write_config_with_hosted_operator(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args(["operator", "create", "--non-interactive"])
        .args(["--wallet-id", WALLET_ID, "--account-path", ACCOUNT_PATH])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "already backs hosted operator(s) 'tvc-operator' (11111111-1111-4111-8111-111111111111)",
        ))
        .stderr(predicate::str::contains("--account-path"));
}

/// Happy path: a different account path derives different keys, so creation
/// proceeds past the collision guard and fails later at credential loading —
/// proving the guard did not fire.
#[test]
fn operator_create_passes_collision_guard_for_different_path() {
    let temp = TempDir::new().unwrap();
    write_config_with_hosted_operator(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args(["operator", "create", "--non-interactive"])
        .args([
            "--wallet-id",
            WALLET_ID,
            "--account-path",
            "m/5527107'/0'/1'",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No API key found"))
        .stderr(predicate::str::contains("already backs").not());
}

#[test]
fn operator_create_help_documents_defaults_and_env_inputs() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--name"))
        .stdout(predicate::str::contains("tvc-operator"))
        .stdout(predicate::str::contains("TVC_OPERATOR_NAME"))
        .stdout(predicate::str::contains("--wallet-name"))
        .stdout(predicate::str::contains("tvc-wallet"))
        .stdout(predicate::str::contains("TVC_OPERATOR_WALLET_NAME"))
        .stdout(predicate::str::contains("--wallet-id"))
        .stdout(predicate::str::contains("TVC_OPERATOR_WALLET_ID"))
        .stdout(predicate::str::contains("--account-path"))
        .stdout(predicate::str::contains("m/5527107'/0'/0'"))
        .stdout(predicate::str::contains("TVC_OPERATOR_ACCOUNT_PATH"));
}

#[test]
fn operator_create_wallet_inputs_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--wallet-name")
        .arg("wallet")
        .arg("--wallet-id")
        .arg("11111111-1111-4111-8111-111111111111")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with '--wallet-id"));
}

#[test]
fn operator_create_rejects_malformed_wallet_uuid() {
    cargo_bin_cmd!("tvc")
        .arg("operator")
        .arg("create")
        .arg("--wallet-id")
        .arg("not-a-uuid")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value 'not-a-uuid' for '--wallet-id <WALLET_ID>'",
        ));
}

#[test]
fn operator_create_accepts_wallet_uuid_with_default_wallet_name() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("operator")
        .arg("create")
        .arg("--wallet-id")
        .arg("11111111-1111-4111-8111-111111111111")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"))
        .stderr(predicate::str::contains("cannot be used with '--wallet-id").not());
}

#[test]
fn operator_create_rejects_empty_text_inputs() {
    for flag in ["--name", "--wallet-name", "--account-path"] {
        cargo_bin_cmd!("tvc")
            .arg("operator")
            .arg("create")
            .arg(flag)
            .arg("")
            .assert()
            .failure()
            .stderr(predicate::str::contains("a value is required"));
    }
}
