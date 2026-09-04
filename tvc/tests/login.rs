mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, OperatorKind, OperatorRecord, OrgConfig};
use uuid::Uuid;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

const ORG_DUP: &str = "11111111-2222-4333-8444-555555555555";
const ORG_SOLO: &str = "55555555-5555-4555-8555-555555555555";
const ORG_TEST: &str = "33333333-3333-4333-8333-333333333333";

fn write_login_config(
    home: &TempDir,
    api_key_path: std::path::PathBuf,
    operator_key_path: std::path::PathBuf,
) {
    let turnkey_dir = home.path().join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    let org_id: Uuid = ORG_TEST.parse().unwrap();
    let mut config = Config::default();
    config.orgs.insert(
        org_id,
        OrgConfig {
            api_key_path,
            api_base_url: "https://api.turnkey.com".to_string(),
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(operator_key_path)],
            extra: toml::Table::new(),
        },
    );
    config.aliases.bind("test".to_string(), org_id);
    config.set_active_org(org_id).unwrap();
    config
        .last_created_app_id
        .insert(org_id, Uuid::from_u128(0xAA));
    config
        .last_operator_ids
        .insert(org_id, vec![Uuid::from_u128(0xBB)]);
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 2\n{}", toml::to_string_pretty(&config).unwrap()),
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

fn saved_config(home: &TempDir) -> String {
    fs::read_to_string(home.path().join(".config/turnkey/tvc.config.toml")).unwrap()
}

/// A v1 (alias-keyed) config is migrated eagerly on the first load: the file
/// is rewritten as v2 behind the backup fence, and the command proceeds
/// against the migrated model.
#[test]
fn any_command_migrates_a_v1_config_eagerly() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!(
            r#"version = 1
active_org = "alias-a"

[orgs.alias-a]
id = "{ORG_DUP}"
api_key_path = "/keys/api.json"
api_base_url = "{}"
"#,
            common::LOCAL_API_BASE_URL
        ),
    )
    .unwrap();

    // The command itself fails (no API key at the fixture path), but the
    // resolution ran against the migrated config and the file was rewritten.
    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Selected org: alias-a ({ORG_DUP})"
        )));

    let saved = saved_config(&temp);
    assert!(saved.contains("version = 2"), "{saved}");
    assert!(saved.contains("[aliases]"), "{saved}");
    assert!(
        !temp
            .path()
            .join(".config/turnkey/tvc.config.toml.backup")
            .exists()
    );
}

/// A leftover migration backup means a previous run crashed mid-migration;
/// every command refuses to run until it is resolved manually.
#[test]
fn a_leftover_migration_backup_blocks_commands() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_DUP)], Some("alias-a"));
    fs::write(turnkey_dir.join("tvc.config.toml.backup"), "old contents").unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"));
}

/// Logging in by organization ID echoes the ID; the alias only appears in
/// output when the user typed one (inputs match outputs).
#[test]
fn login_by_id_echoes_the_id_not_the_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_DUP)], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg(ORG_DUP)
        .assert()
        .failure() // dead-port whoami
        .stdout(predicate::str::contains(format!("Selected org: {ORG_DUP}")))
        .stdout(predicate::str::contains("Selected org: alias-a").not());
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
    assert!(!saved.contains(ORG_TEST));
}

/// Non-interactive login never touches the filesystem layout: legacy
/// alias-keyed directories and the paths pointing at them stay put.
#[test]
fn login_non_interactive_leaves_legacy_layout_alone() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_SOLO)], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let legacy_dir = temp.path().join(".config/turnkey/orgs/alias-a");

    // Exits nonzero at the dead-port whoami; the layout must be untouched.
    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stdout(predicate::str::contains("Moved key directory").not());

    assert!(legacy_dir.join("api_key.json").exists());
    assert!(
        !temp
            .path()
            .join(".config/turnkey/orgs")
            .join(ORG_SOLO)
            .exists()
    );

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("orgs/alias-a/api_key.json"));
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

/// Deleting a profile with the legacy alias-keyed directory layout still
/// removes that directory (TVC-55 changed the default to id-keyed, but the
/// legacy layout remains recognized).
#[test]
fn login_delete_removes_legacy_alias_keyed_layout() {
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
    assert!(saved.contains("version = 2"));
    assert!(!saved.contains(ORG_TEST));
    assert!(!saved.contains(&Uuid::from_u128(0xAA).to_string()));
    assert!(!saved.contains(&Uuid::from_u128(0xBB).to_string()));
}

/// Deleting a profile with the id-keyed directory layout (what login creates
/// since TVC-55) removes that directory.
#[test]
fn login_delete_removes_id_keyed_layout() {
    let temp = TempDir::new().unwrap();
    let org_dir = temp.path().join(".config/turnkey/orgs").join(ORG_TEST);
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
    assert!(!saved.contains(ORG_TEST));
}

/// A hand-edited config can point several organizations into one directory;
/// deleting one must not take the survivor's keys with it.
#[test]
fn profile_delete_keeps_directory_shared_with_another_org() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    let org_dir = turnkey_dir.join("orgs").join(ORG_DUP);
    fs::create_dir_all(&org_dir).unwrap();
    fs::write(org_dir.join("api_key.json"), "shared api key").unwrap();
    fs::write(org_dir.join("operator.json"), "shared operator key").unwrap();

    let shared_profile = || OrgConfig {
        api_key_path: org_dir.join("api_key.json"),
        api_base_url: "https://api.turnkey.com".to_string(),
        default_operator_kind: OperatorKind::Local,
        operators: vec![OperatorRecord::local(org_dir.join("operator.json"))],
        extra: toml::Table::new(),
    };
    let org_a: Uuid = ORG_DUP.parse().unwrap();
    let org_b: Uuid = ORG_TEST.parse().unwrap();
    let mut config = Config::default();
    config.orgs.insert(org_a, shared_profile());
    config.orgs.insert(org_b, shared_profile());
    config.aliases.bind("alias-a".to_string(), org_a);
    config.aliases.bind("alias-b".to_string(), org_b);
    config.set_active_org(org_b).unwrap();
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 2\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("delete")
        .arg("--org")
        .arg("alias-a")
        .arg("--yes")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "is still used by another organization and was NOT deleted",
        ));

    assert!(org_dir.join("api_key.json").exists());
    assert!(org_dir.join("operator.json").exists());

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("alias-b"));
    assert!(!saved.contains("alias-a"));
}

/// Deleting a yubikey-default profile removes the org but never the device:
/// the shared [[yubikeys]] registry entry survives and the output points at
/// the explicit local unregistration command.
#[test]
fn login_delete_keeps_the_yubikey_registry_entry() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config(temp.path(), "test", ORG_TEST);

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
    assert!(!saved.contains(ORG_TEST), "{saved}");
}

/// With two profiles sharing one registered serial, deleting one leaves the
/// other profile and the shared registry entry fully intact.
#[test]
fn login_delete_preserves_a_shared_yubikey_registry_entry() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_shared_config(
        temp.path(),
        &[
            ("one", "10000000-0000-4000-8000-000000000011"),
            ("two", "10000000-0000-4000-8000-000000000022"),
        ],
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .args(["profile", "delete", "--org", "one", "--yes"])
        .assert()
        .success();

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(
        !saved.contains("10000000-0000-4000-8000-000000000011"),
        "{saved}"
    );
    assert!(
        saved.contains("10000000-0000-4000-8000-000000000022"),
        "{saved}"
    );
    assert!(saved.contains("[[yubikeys]]"), "{saved}");
    assert!(saved.contains("serial = \"01c95c1f\""), "{saved}");
}
