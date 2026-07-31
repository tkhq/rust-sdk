mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use indexmap::IndexMap;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, OperatorKind, OperatorRecord, OrgConfig};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const ORG_DUP: &str = "11111111-2222-4333-8444-555555555555";
const ORG_SOLO: &str = "22222222-2222-4222-8222-222222222222";
const ORG_TEST: &str = "33333333-3333-4333-8333-333333333333";

fn write_login_config(
    home: &TempDir,
    api_key_path: std::path::PathBuf,
    operator_key_path: std::path::PathBuf,
) {
    let turnkey_dir = home.path().join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: IndexMap::from([(
            "test".to_string(),
            OrgConfig {
                id: ORG_TEST.parse().unwrap(),
                api_key_path,
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: vec![OperatorRecord::local(operator_key_path)],
                default_alias: false,
                extra: toml::Table::new(),
            },
        )]),
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

fn saved_config(home: &TempDir) -> String {
    fs::read_to_string(home.path().join(".config/turnkey/tvc.config.toml")).unwrap()
}

/// `true` when the saved config marks exactly `alias` (and no sibling) with
/// `default_alias = true`.
fn is_sole_default(saved: &str, alias: &str) -> bool {
    let table: toml::Table = toml::from_str(saved).unwrap();
    let orgs = table["orgs"].as_table().unwrap();
    orgs.iter().all(|(other, org)| {
        let marked = org
            .as_table()
            .unwrap()
            .get("default_alias")
            .is_some_and(|value| value == &toml::Value::Boolean(true));
        marked == (other == alias)
    })
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

/// TVC-159: an org-ID query matching several unmarked profiles must not
/// resolve to an arbitrary one. Config loading marks the first profile in
/// file order as the default; login follows it, warns, and persists the
/// marker when it saves.
#[test]
fn login_non_interactive_unmarked_duplicates_use_first_profile() {
    let temp = TempDir::new().unwrap();
    // "alias-b" is written first deliberately: file order decides, not name
    // order.
    common::write_profiles_config(
        temp.path(),
        &[("alias-b", ORG_DUP), ("alias-a", ORG_DUP)],
        Some("alias-a"),
        &[],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    // The command still exits nonzero at the network step (dead-port base
    // URL), but selection happens and is reported before that.
    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg(ORG_DUP)
        .assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Selected org: alias-b ({ORG_DUP})"
        )))
        .stderr(predicate::str::contains("Using default profile 'alias-b'"));

    // Login saved the config, so the repaired marker is now on disk and every
    // later resolution sticks with alias-b.
    assert!(is_sole_default(&saved_config(&temp), "alias-b"));
}

/// With a `default_alias` marker on file, non-interactive login resolves an
/// org-ID query to the marked profile (file order does not matter) and warns
/// that duplicates remain.
#[test]
fn login_non_interactive_resolves_org_id_to_default_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &["alias-b"],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg(ORG_DUP)
        .assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Selected org: alias-b ({ORG_DUP})"
        )))
        .stderr(predicate::str::contains("Using default profile 'alias-b'"));
}

/// Several profiles marked `default_alias` for one organization (hand-edited
/// config) is repaired at load time: the first marked profile in file order
/// wins and the save normalizes the file.
#[test]
fn login_non_interactive_multiple_marked_defaults_keep_first_marked() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &["alias-a", "alias-b"],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg(ORG_DUP)
        .assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Selected org: alias-a ({ORG_DUP})"
        )))
        .stderr(predicate::str::contains("Using default profile 'alias-a'"));

    assert!(is_sole_default(&saved_config(&temp), "alias-a"));
}

/// Explicitly logging in with the marked default alias works, with the same
/// duplicate warning; a non-default (secondary) alias is refused with
/// instructions naming the default.
#[test]
fn login_non_interactive_refuses_secondary_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &["alias-b"],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-b")
        .assert()
        .failure()
        .stdout(predicate::str::contains(format!(
            "Selected org: alias-b ({ORG_DUP})"
        )))
        .stderr(predicate::str::contains("duplicate profiles"));

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "Profile 'alias-a' is a duplicate: the default profile for organization \
             '{ORG_DUP}' is 'alias-b'.",
        )))
        .stderr(predicate::str::contains(
            "tvc profile set-default-alias --org alias-a",
        ));
}

/// `profile set-default-alias` marks the named profile and clears the marker
/// from the organization's other profiles.
#[test]
fn profile_set_default_alias_marks_profile_and_clears_siblings() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &["alias-b"],
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("set-default-alias")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "Marked 'alias-a' as the default alias for organization '{ORG_DUP}' \
             (duplicates: alias-b).",
        )));

    let saved = saved_config(&temp);
    assert!(is_sole_default(&saved, "alias-a"));
    assert!(
        !saved.contains("default_alias = false"),
        "cleared marker must not be serialized: {saved}"
    );
}

/// A default alias is only meaningful while duplicates exist; a
/// non-duplicated organization is refused.
#[test]
fn profile_set_default_alias_errors_without_duplicates() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", ORG_SOLO)], Some("alias-a"), &[]);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("set-default-alias")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "a default alias is only needed while duplicate profiles exist",
        ));
}

/// The marker names one profile among several, so an org-ID query is
/// ambiguous; the error lists the aliases to choose from.
#[test]
fn profile_set_default_alias_requires_alias_not_org_id() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &[],
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("set-default-alias")
        .arg("--org")
        .arg(ORG_DUP)
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "'{ORG_DUP}' is an organization ID; pass the profile alias to mark as its \
             default (one of: alias-a, alias-b)",
        )));
}

/// TVC-159: same fence for `profile delete` — an ambiguous org-ID query must
/// not delete an arbitrary profile, and the `default_alias` marker is ignored
/// for destructive commands.
#[test]
fn profile_delete_non_interactive_with_duplicate_org_id_lists_profiles() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_DUP), ("alias-b", ORG_DUP)],
        Some("alias-a"),
        &["alias-a"],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("delete")
        .arg("--org")
        .arg(ORG_DUP)
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains(format!(
            "Organization '{ORG_DUP}' is configured under multiple profiles: alias-a, alias-b",
        )))
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
    let saved = saved_config(&temp);
    assert!(saved.contains("version = 1"));
    assert!(!saved.contains(ORG_TEST));
    assert!(!saved.contains("app-1"));
    assert!(!saved.contains("operator-1"));
}

/// A hand-edited config can point several profiles into one id-keyed
/// directory; deleting one profile must not take the survivor's keys with it.
#[test]
fn profile_delete_keeps_directory_shared_with_another_profile() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config/turnkey");
    let org_dir = turnkey_dir.join("orgs").join(ORG_DUP);
    fs::create_dir_all(&org_dir).unwrap();
    fs::write(org_dir.join("api_key.json"), "shared api key").unwrap();
    fs::write(org_dir.join("operator.json"), "shared operator key").unwrap();

    let shared_profile = || OrgConfig {
        id: ORG_DUP.parse().unwrap(),
        api_key_path: org_dir.join("api_key.json"),
        api_base_url: "https://api.turnkey.com".to_string(),
        default_operator_kind: OperatorKind::Local,
        operators: vec![OperatorRecord::local(org_dir.join("operator.json"))],
        default_alias: false,
        extra: toml::Table::new(),
    };
    let config = Config {
        active_org: Some("alias-b".to_string()),
        orgs: IndexMap::from([
            ("alias-a".to_string(), shared_profile()),
            ("alias-b".to_string(), shared_profile()),
        ]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
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
            "is still used by another profile and was NOT deleted",
        ));

    assert!(org_dir.join("api_key.json").exists());
    assert!(org_dir.join("operator.json").exists());

    let saved = saved_config(&temp);
    assert!(saved.contains("alias-b"));
    assert!(!saved.contains("alias-a"));
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
    assert!(!saved_config(&temp).contains(ORG_TEST));
}
