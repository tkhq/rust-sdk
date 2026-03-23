use std::fs;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn config_command_help_lists_subcommands() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.args(["config", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn config_round_trip() {
    let temp = tempdir().expect("temp dir should exist");
    let config_path = temp.path().join("auth.toml");

    let mut set_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    set_cmd
        .args(["config", "set", "turnkey.organizationId", "persisted-org"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    set_cmd.assert().success();

    let stored = fs::read_to_string(&config_path).expect("config file should exist");
    assert!(stored.contains("organizationId = \"persisted-org\""));

    let mut get_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    get_cmd
        .args(["config", "get", "turnkey.organizationId"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("persisted-org"));

    let mut list_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    list_cmd
        .args(["config", "list"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env("TURNKEY_ORGANIZATION_ID", "env-org")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    list_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("organizationId = \"env-org\""));
}

#[test]
fn config_list_and_get_redact_private_key() {
    let temp = tempdir().expect("temp dir should exist");
    let config_path = temp.path().join("auth.toml");

    let mut set_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    set_cmd
        .args(["config", "set", "turnkey.apiPrivateKey", "persisted-private-key"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    set_cmd.assert().success();

    let mut list_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    list_cmd
        .args(["config", "list"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    list_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("apiPrivateKey = \"<redacted>\""))
        .stdout(predicate::str::contains("persisted-private-key").not());

    let mut get_cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    get_cmd
        .args(["config", "get", "turnkey.apiPrivateKey"])
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");
    get_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("<redacted>"))
        .stdout(predicate::str::contains("persisted-private-key").not());
}
