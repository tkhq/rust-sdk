mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, OperatorKind, OperatorRecord, OrgConfig};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

fn write_login_config(
    home: &TempDir,
    api_key_path: std::path::PathBuf,
    operator_key_path: std::path::PathBuf,
) {
    let turnkey_dir = home.path().join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path,
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: vec![OperatorRecord::local(operator_key_path)],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: Default::default(),
        last_created_app_id: HashMap::from([("test".to_string(), "app-1".to_string())]),
        last_operator_ids: HashMap::from([("test".to_string(), vec!["operator-1".to_string()])]),
        extra: toml::Table::new(),
    };
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// When `--org <alias>` points to an alias that is not present in the local
/// config, we fail fast without entering any interactive flow. Exercises the
/// `OrgPlan::Existing` branch in `execute_login`.
#[test]
fn login_errors_when_provided_org_not_found() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("login")
        .arg("--org")
        .arg("does-not-exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Organization 'does-not-exist' not found",
        ));
}

/// TVC-159: an org-ID query matching several profiles must not resolve to an
/// arbitrary one; non-interactive login fails fast and names them all.
#[test]
fn login_non_interactive_with_duplicate_org_id_lists_profiles() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("org-dup-test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Organization 'org-dup-test' is configured under multiple profiles: alias-a, alias-b",
        ))
        .stderr(predicate::str::contains("--org <alias>"));
}

/// TVC-159: same fence for `profile delete` — an ambiguous org-ID query must
/// not delete an arbitrary profile.
#[test]
fn profile_delete_non_interactive_with_duplicate_org_id_lists_profiles() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("delete")
        .arg("--org")
        .arg("org-dup-test")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Organization 'org-dup-test' is configured under multiple profiles: alias-a, alias-b",
        ))
        .stderr(predicate::str::contains(
            "to select which profile to delete",
        ));

    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-b").exists());
}

/// Custom (non-default-layout) key paths are never deleted: the profile entry
/// is removed from the config but the files stay on disk with a warning.
#[test]
fn profile_delete_warns_and_keeps_custom_key_paths() {
    let temp = TempDir::new().unwrap();
    let custom_dir = temp.path().join("custom-keys");
    let api_key_path = custom_dir.join("api_key.json");
    let operator_key_path = custom_dir.join("operator.json");
    fs::create_dir_all(&custom_dir).unwrap();
    fs::write(&api_key_path, "custom api key").unwrap();
    fs::write(&operator_key_path, "custom operator key").unwrap();
    write_login_config(&temp, api_key_path.clone(), operator_key_path.clone());

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("delete")
        .arg("--org")
        .arg("test")
        .arg("--yes")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "custom key paths are configured and were NOT deleted",
        ));

    assert!(api_key_path.exists());
    assert!(operator_key_path.exists());

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("org-test"));
}

#[test]
fn login_help_shows_api_base_url_override() {
    cargo_bin_cmd!("tvc")
        .arg("login")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--api-base-url"))
        .stdout(predicate::str::contains("TVC_API_BASE_URL"));
}

#[test]
fn login_delete_removes_default_registry_key_layout() {
    let temp = TempDir::new().unwrap();
    let org_dir = temp.path().join(".config/turnkey/orgs/test");
    let api_key_path = org_dir.join("api_key.json");
    let operator_key_path = org_dir.join("operator.json");
    fs::create_dir_all(&org_dir).unwrap();
    fs::write(&api_key_path, "not needed for deletion").unwrap();
    fs::write(&operator_key_path, "not needed for deletion").unwrap();
    write_login_config(&temp, api_key_path, operator_key_path);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("delete")
        .arg("--org")
        .arg("test")
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed key directory"));

    assert!(!org_dir.exists());
    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("version = 1"));
    assert!(!saved.contains("org-test"));
    assert!(!saved.contains("app-1"));
    assert!(!saved.contains("operator-1"));
}

/// Deleting a yubikey-default profile removes the org but never the device:
/// the shared [[yubikeys]] registry entry survives and the output points at
/// the explicit local unregistration command.
#[test]
fn login_delete_keeps_the_yubikey_registry_entry() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config(temp.path(), "test", "org-test");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["profile", "delete", "--org", "test", "--yes"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Kept the YubiKey registry entries (serials 01c95c1f)",
        ))
        .stdout(predicate::str::contains("tvc yubikey unregister"));

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("[[yubikeys]]"), "{saved}");
    assert!(saved.contains("serial = \"01c95c1f\""), "{saved}");
    assert!(!saved.contains("org-test"), "{saved}");
}

/// With two profiles sharing one registered serial, deleting one leaves the
/// other profile and the shared registry entry fully intact.
#[test]
fn login_delete_preserves_a_shared_yubikey_registry_entry() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_shared_config(temp.path(), &[("one", "org-1"), ("two", "org-2")]);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["profile", "delete", "--org", "one", "--yes"])
        .assert()
        .success();

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("org-1"), "{saved}");
    assert!(saved.contains("org-2"), "{saved}");
    assert!(saved.contains("[[yubikeys]]"), "{saved}");
    assert!(saved.contains("serial = \"01c95c1f\""), "{saved}");
}
