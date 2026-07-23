use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

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
