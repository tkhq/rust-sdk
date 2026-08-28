//! Command-boundary tests for local YubiKey registration management.
//!
//! Everything here finishes before any hardware access: argument parsing,
//! non-interactive guards, and the registry and organization-reference rules.
//! Device reads and private-key operations are covered by the unit tests in
//! the yubikey module and its ignored hardware cycle.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{
    Config, OperatorRecord, OperatorRecordKind, OrgConfig, YubiKeyOperatorRecord, YubiKeyRegistry,
    YubiKeyRegistryEntry,
};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const SERIAL: &str = "01c95c1f";

fn write_config(home: &TempDir, config: &Config) {
    let turnkey_dir = home.path().join(".config").join("turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(config).unwrap()),
    )
    .unwrap();
}

fn config_with_registered_device() -> Config {
    let entry = YubiKeyRegistryEntry {
        serial: SERIAL.parse().unwrap(),
        public_key: "07".repeat(130).parse().unwrap(),
        extra: toml::Table::new(),
    };

    Config {
        yubikeys: YubiKeyRegistry::try_from(vec![entry]).unwrap(),
        ..Config::default()
    }
}

fn org_with_yubikey_operator() -> OrgConfig {
    OrgConfig {
        id: "org-test".to_string(),
        api_key_path: "/keys/api.json".into(),
        api_base_url: "http://127.0.0.1:1".to_string(),
        default_operator_kind: Default::default(),
        operators: vec![OperatorRecord {
            name: "yubikey".to_string(),
            kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                serial: SERIAL.parse().unwrap(),
                extra: toml::Table::new(),
            }),
        }],
        extra: toml::Table::new(),
    }
}

#[test]
fn help_lists_unregister_and_omits_device_mutation_commands() {
    cargo_bin_cmd!("tvc")
        .args(["keys", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("provision-yubikey").not())
        .stdout(predicate::str::contains("delete-yubikey").not());

    cargo_bin_cmd!("tvc")
        .args(["yubikey", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create-certs"))
        .stdout(predicate::str::contains("unregister"));
}

#[test]
fn serial_arguments_are_typed_hex() {
    cargo_bin_cmd!("tvc")
        .args(["yubikey", "unregister", "--serial", "zzzz"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be bare hex encoded"));
}

#[test]
fn unregister_requires_serial_and_yes_non_interactively() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["yubikey", "unregister"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--serial is required in non-interactive mode",
        ));

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["yubikey", "unregister", "--serial", SERIAL])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--yes is required in non-interactive mode",
        ));
}

#[test]
fn unregister_refuses_an_unregistered_serial() {
    let temp = TempDir::new().unwrap();
    write_config(&temp, &Config::default());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["yubikey", "unregister", "--serial", SERIAL, "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "YubiKey {SERIAL} is not in the registry"
        )));
}

#[test]
fn unregister_refuses_a_device_an_organization_references() {
    let temp = TempDir::new().unwrap();
    let config = Config {
        orgs: HashMap::from([("test".to_string(), org_with_yubikey_operator())]),
        ..config_with_registered_device()
    };
    write_config(&temp, &config);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["yubikey", "unregister", "--serial", SERIAL, "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "YubiKey {SERIAL} is an operator for organization(s) test"
        )))
        .stderr(predicate::str::contains(
            "remove those operator records first",
        ));
}

#[test]
fn json_mode_wraps_the_registry_refusal_in_the_error_envelope() {
    let temp = TempDir::new().unwrap();
    write_config(&temp, &Config::default());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args([
            "--message-format",
            "json",
            "yubikey",
            "unregister",
            "--serial",
            SERIAL,
            "--yes",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""reason":"command_error""#))
        .stdout(predicate::str::contains(format!(
            "YubiKey {SERIAL} is not in the registry"
        )));
}

#[test]
fn unregister_removes_only_the_local_registry_entry() {
    let temp = TempDir::new().unwrap();
    write_config(&temp, &config_with_registered_device());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["yubikey", "unregister", "--serial", SERIAL, "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "YubiKey {SERIAL} was removed from the local TVC configuration"
        )))
        .stdout(predicate::str::contains("The device was not modified"))
        .stdout(predicate::str::contains(
            "no organization operator was revoked",
        ));

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    let config = Config::from_toml(&saved).unwrap();
    assert!(!config.yubikeys.contains(SERIAL.parse().unwrap()));
}
