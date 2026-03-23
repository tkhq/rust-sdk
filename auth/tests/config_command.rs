mod support;

use std::fs;

use predicates::prelude::*;
use serde_json::Value;

#[test]
fn config_command_help_lists_subcommands() {
    let env = support::AuthTestEnv::new();
    let mut cmd = env.command();
    cmd.args(["config", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn config_round_trip() {
    let env = support::AuthTestEnv::new();

    let mut set_cmd = env.command_without_auth_env();
    set_cmd.args(["config", "set", "turnkey.organizationId", "persisted-org"]);
    set_cmd.assert().success();

    let stored = fs::read_to_string(env.config_path()).expect("config file should exist");
    assert!(stored.contains("organizationId = \"persisted-org\""));

    let mut get_cmd = env.command_without_auth_env();
    get_cmd.args(["config", "get", "turnkey.organizationId"]);
    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("persisted-org"));

    let mut list_cmd = env.command_without_auth_env();
    list_cmd
        .args(["config", "list"])
        .env("TURNKEY_ORGANIZATION_ID", "env-org");
    let output = list_cmd.assert().success().get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&output).expect("config list should output json");
    assert_eq!(value["turnkey"]["organizationId"], "env-org");
}

#[test]
fn config_list_and_get_redact_private_key() {
    let env = support::AuthTestEnv::new();

    let mut set_cmd = env.command_without_auth_env();
    set_cmd.args([
        "config",
        "set",
        "turnkey.apiPrivateKey",
        "persisted-private-key",
    ]);
    set_cmd.assert().success();

    let mut list_cmd = env.command_without_auth_env();
    list_cmd.args(["config", "list"]);
    let output = list_cmd.assert().success().get_output().stdout.clone();
    let value: Value = serde_json::from_slice(&output).expect("config list should output json");
    assert_eq!(value["turnkey"]["apiPrivateKey"], "<redacted>");
    assert!(!String::from_utf8_lossy(&output).contains("persisted-private-key"));

    let mut get_cmd = env.command_without_auth_env();
    get_cmd.args(["config", "get", "turnkey.apiPrivateKey"]);
    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("persisted-private-key").not());
}
