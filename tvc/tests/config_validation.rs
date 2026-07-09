use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::json;
use std::fs;
use tempfile::NamedTempFile;
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
