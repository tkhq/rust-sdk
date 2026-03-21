use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn cli_help_lists_commands() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("git-sign"))
        .stdout(predicate::str::contains("public-key"));
}

#[test]
fn public_key_requires_turnkey_org_id() {
    let temp = tempdir().expect("temp dir should exist");
    let config_path = temp.path().join("auth.toml");

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("public-key")
        .env("TURNKEY_AUTH_CONFIG_PATH", &config_path)
        .env_remove("TURNKEY_ORGANIZATION_ID")
        .env_remove("TURNKEY_API_PUBLIC_KEY")
        .env_remove("TURNKEY_API_PRIVATE_KEY")
        .env_remove("TURNKEY_PRIVATE_KEY_ID")
        .env_remove("TURNKEY_API_BASE_URL");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("turnkey.organizationId"));
}

#[test]
fn introspect_includes_nested_subcommand_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_auth"))
        .arg("introspect")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let schema: Value =
        serde_json::from_slice(&output).expect("introspect output should be valid JSON");
    let commands = schema["commands"]
        .as_array()
        .expect("top-level commands should be an array");
    let config = commands
        .iter()
        .find(|command| command["name"] == "config")
        .expect("config command should exist");
    let subcommands = config["subcommands"]
        .as_array()
        .expect("config command should expose nested subcommands");
    let get = subcommands
        .iter()
        .find(|command| command["name"] == "get")
        .expect("config get subcommand should exist");
    let flags = get["flags"]
        .as_array()
        .expect("config get flags should be serialized");

    assert!(
        flags.iter().any(|flag| flag["name"] == "key"),
        "config get should expose its key argument"
    );
}
