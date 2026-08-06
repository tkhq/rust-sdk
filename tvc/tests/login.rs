mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, OperatorKind, OperatorRecord, OrgConfig};
use uuid::Uuid;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const ORG_A: &str = "11111111-2222-4333-8444-555555555555";
const ORG_B: &str = "22222222-2222-4222-8222-222222222222";
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
        .insert(org_id, "app-1".to_string());
    config
        .last_operator_ids
        .insert(org_id, vec!["operator-1".to_string()]);

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 2\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

fn saved_config(home: &TempDir) -> String {
    fs::read_to_string(home.path().join(".config/turnkey/tvc.config.toml")).unwrap()
}

/// When `--org <alias>` points to an alias that is not present in the local
/// config, we fail fast without entering any interactive flow.
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
id = "{ORG_A}"
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
            "Selected org: alias-a ({ORG_A})"
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
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_A)], Some("alias-a"));
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
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_A)], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg(ORG_A)
        .assert()
        .failure() // dead-port whoami
        .stdout(predicate::str::contains(format!("Selected org: {ORG_A}")))
        .stdout(predicate::str::contains("Selected org: alias-a").not());
}

/// Non-interactive login never touches the filesystem layout: legacy
/// alias-keyed directories and the paths pointing at them stay put.
#[test]
fn login_non_interactive_leaves_legacy_layout_alone() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_B)], Some("alias-a"));
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
            .join(ORG_B)
            .exists()
    );
    assert!(saved_config(&temp).contains("orgs/alias-a/api_key.json"));
}

/// A hand-edited config can point several organizations into one directory
/// (here: org B's keys living inside org A's id-keyed directory); deleting
/// one must not take the survivor's keys with it.
#[test]
fn profile_delete_keeps_directory_shared_with_another_org() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    let shared_dir = turnkey_dir.join("orgs").join(ORG_A);
    fs::create_dir_all(&shared_dir).unwrap();
    fs::write(shared_dir.join("api_key.json"), "shared api key").unwrap();
    fs::write(shared_dir.join("operator.json"), "shared operator key").unwrap();

    let org_a: Uuid = ORG_A.parse().unwrap();
    let org_b: Uuid = ORG_B.parse().unwrap();
    let shared_profile = || OrgConfig {
        api_key_path: shared_dir.join("api_key.json"),
        api_base_url: "https://api.turnkey.com".to_string(),
        default_operator_kind: OperatorKind::Local,
        operators: vec![OperatorRecord::local(shared_dir.join("operator.json"))],
        extra: toml::Table::new(),
    };
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

    assert!(shared_dir.join("api_key.json").exists());
    assert!(shared_dir.join("operator.json").exists());

    let saved = saved_config(&temp);
    assert!(saved.contains("alias-b"));
    assert!(!saved.contains("alias-a"));
}

/// Deleting an organization whose keys use the legacy alias-keyed directory
/// layout still removes that directory, along with every alias bound to it.
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
        .stdout(predicate::str::contains("Removed key directory"))
        .stdout(predicate::str::contains("('test')"));

    assert!(!org_dir.exists());
    let saved = saved_config(&temp);
    assert!(saved.contains("version = 2"));
    assert!(!saved.contains(ORG_TEST));
    assert!(!saved.contains("app-1"));
    assert!(!saved.contains("operator-1"));
}

/// Deleting an organization with the id-keyed directory layout (what login
/// creates) removes that directory.
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
    assert!(!saved_config(&temp).contains(ORG_TEST));
}

/// Custom (non-default-layout) key paths are never deleted: the config entry
/// is removed but the files stay on disk with a warning.
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

    assert!(!saved_config(&temp).contains(ORG_TEST));
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
