use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::json;
use std::fs;
use tempfile::{NamedTempFile, TempDir};
use tvc::config::deploy::DeployConfig;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

fn write_json(value: &serde_json::Value) -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), serde_json::to_string_pretty(value).unwrap()).unwrap();
    file
}

#[test]
fn deploy_create_non_interactive_reports_all_placeholder_errors() {
    let config = NamedTempFile::new().unwrap();
    fs::write(
        config.path(),
        serde_json::to_string_pretty(&DeployConfig::template(None)).unwrap(),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("create")
        .arg("--config-file")
        .arg(config.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("app_id"))
        .stderr(predicate::str::contains("pivot_container_image_url"))
        .stderr(predicate::str::contains("pivot_path"))
        .stderr(predicate::str::contains("expected_pivot_digest"))
        .stderr(predicate::str::contains(
            "pivotContainerEncryptedPullSecret",
        ));
}

#[test]
fn app_create_non_interactive_reports_placeholder_and_structural_errors() {
    let config = write_json(&json!({
        "name": "<FILL_IN_APP_NAME>",
        "quorumPublicKey": "test-quorum-public-key",
        "manifestSetId": "manifest-set-id",
        "manifestSetParams": {
            "name": "<FILL_IN_MANIFEST_SET_NAME>",
            "threshold": 1,
            "newOperators": [{
                "name": "operator-1",
                "publicKey": "<FILL_IN_OPERATOR_PUBLIC_KEY>"
            }]
        },
        "shareSetId": "share-set-id",
        "shareSetParams": {
            "name": "share-set",
            "threshold": 1,
            "newOperators": []
        }
    }));

    cargo_bin_cmd!("tvc")
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("app")
        .arg("create")
        .arg("--config-file")
        .arg(config.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("name"))
        .stderr(predicate::str::contains("manifestSetParams.name"))
        .stderr(predicate::str::contains(
            "manifestSetParams.newOperators[0].publicKey",
        ))
        .stderr(predicate::str::contains("manifestSetId"))
        .stderr(predicate::str::contains("shareSetId"))
        .stderr(predicate::str::contains("shareSetParams"))
        .stderr(predicate::str::contains("shareSetParams.threshold"));
}

#[test]
fn deploy_create_rejects_image_placeholder_before_authentication_or_submission() {
    let mut config = DeployConfig::template(Some("651b573c-861b-4f10-a478-cbcfe0c226af"));
    config.pivot_path = "/bin/pivot".into();
    config.expected_pivot_digest = "a".repeat(64);
    config.pivot_container_encrypted_pull_secret = None;
    let file = write_json(&serde_json::to_value(config).unwrap());
    let home = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", home.path())
        .args([
            "deploy",
            "create",
            "--message-format",
            "json",
            "--config-file",
        ])
        .arg(file.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("command_error"))
        .stdout(predicate::str::contains(
            "pivot_container_image_url contains placeholder",
        ))
        .stdout(predicate::str::contains("unauthorized").not());
}
