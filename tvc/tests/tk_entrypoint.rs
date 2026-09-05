use assert_cmd::cargo::cargo_bin_cmd;
use clap::Parser;
use serde_json::Value;
use tempfile::TempDir;
use tvc::cli::TkCli;

#[test]
fn unified_command_tree_accepts_shared_flags_and_scopes_cloud_commands() {
    for args in [
        vec!["tk", "--message-format=json", "version"],
        vec!["tk", "version", "--message-format=json"],
        vec!["tk", "tvc", "app", "list", "--non-interactive"],
    ] {
        TkCli::try_parse_from(args).expect("unified invocation should parse");
    }
    for command in ["login", "profile", "version"] {
        assert!(TkCli::try_parse_from(["tk", "tvc", command]).is_err());
    }
    assert!(TkCli::try_parse_from(["tk", "deploy"]).is_err());
}

#[test]
fn legacy_and_nested_commands_produce_identical_offline_artifacts_and_json() {
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("deploy.json");
    let legacy = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .current_dir(temp.path())
        .args(["deploy", "init", "--message-format=json", "--output"])
        .arg(&output_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let artifact = std::fs::read(&output_path).unwrap();
    std::fs::remove_file(&output_path).unwrap();
    let nested = cargo_bin_cmd!("tk")
        .env("HOME", temp.path())
        .current_dir(temp.path())
        .args(["--message-format=json", "tvc", "deploy", "init", "--output"])
        .arg(&output_path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        serde_json::from_slice::<Value>(&legacy).unwrap(),
        serde_json::from_slice::<Value>(&nested).unwrap()
    );
    assert_eq!(artifact, std::fs::read(&output_path).unwrap());
}

#[test]
fn unified_usage_errors_follow_the_existing_json_protocol() {
    let result = cargo_bin_cmd!("tk")
        .args(["--message-format=json", "tvc", "unknown-command"])
        .assert()
        .code(2);
    let message: Value = serde_json::from_slice(&result.get_output().stdout).unwrap();
    assert_eq!(message["reason"], "command_error");
    assert_eq!(message["code"], "usage_error");
    assert!(result.get_output().stderr.is_empty());
}
