use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

#[test]
fn deploy_init_defaults_to_human_output() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("deploy.json");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .current_dir(temp.path())
        .arg("deploy")
        .arg("init")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Created deployment config template:",
        ))
        .stdout(predicate::str::contains("\"reason\"").not());
}

#[test]
fn deploy_init_json_output_emits_structured_message() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("deploy.json");

    let assert = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .current_dir(temp.path())
        .arg("--message-format=json")
        .arg("deploy")
        .arg("init")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Created deployment config template:").not());

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected one JSON message, got {stdout:?}");

    let message: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(message["reason"], "deployment-config-created");
    assert_eq!(message["command"], "deploy init");
    assert_eq!(message["path"], output.display().to_string());
    assert_eq!(message["template"], true);
    assert_eq!(message["interactive"], false);
}

#[test]
fn login_missing_org_in_json_mode_outputs_structured_error_to_stdout() {
    let temp = TempDir::new().unwrap();

    let assert = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--message-format=json")
        .arg("login")
        .assert()
        .failure()
        .stderr(predicate::str::is_empty());

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(lines.len(), 1, "expected one JSON error, got {stdout:?}");

    let message: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(message["reason"], "missing-required-input");
    assert_eq!(message["code"], "missing_required_input");
    assert!(message["message"].as_str().unwrap().contains("--org"));
}
