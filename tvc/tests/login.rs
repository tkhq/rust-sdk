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
/// arbitrary one; without a `default_alias` marker, non-interactive login
/// fails fast naming the profiles and every exit.
#[test]
fn login_non_interactive_without_default_alias_names_every_exit() {
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
            "Multiple profiles are configured for organization 'org-dup-test': alias-a, alias-b",
        ))
        .stderr(predicate::str::contains(
            "tvc profile set-default-alias --org <alias>",
        ))
        .stderr(predicate::str::contains(
            "tvc profile delete --org <alias> --yes",
        ))
        .stderr(predicate::str::contains(
            "run `tvc login` interactively to consolidate",
        ));
}

/// With a `default_alias` marker, non-interactive login resolves an org-ID
/// query to the marked profile and warns that duplicates remain.
#[test]
fn login_non_interactive_resolves_org_id_to_default_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config_with_defaults(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
        &["alias-b"],
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
        .arg("org-dup-test")
        .assert()
        .failure()
        .stdout(predicate::str::contains(
            "Selected org: alias-b (org-dup-test)",
        ))
        .stderr(predicate::str::contains("Using default profile 'alias-b'"));
}

/// Explicitly logging in with the marked default alias works, with the same
/// duplicate warning; a non-default (secondary) alias is refused with
/// instructions naming the default.
#[test]
fn login_non_interactive_refuses_secondary_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config_with_defaults(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
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
        .stdout(predicate::str::contains(
            "Selected org: alias-b (org-dup-test)",
        ))
        .stderr(predicate::str::contains("duplicate profiles"));

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("alias-a")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Profile 'alias-a' is a duplicate: the default profile for organization \
             'org-dup-test' is 'alias-b'.",
        ))
        .stderr(predicate::str::contains(
            "tvc profile set-default-alias --org alias-a",
        ));
}

/// Several profiles marked `default_alias` for one organization (hand-edited
/// config) is refused with the command that repairs it.
#[test]
fn login_non_interactive_refuses_multiple_marked_defaults() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config_with_defaults(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
        &["alias-a", "alias-b"],
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
            "Multiple profiles are marked default_alias for organization 'org-dup-test': \
             alias-a, alias-b",
        ));
}

/// `profile set-default-alias` marks the named profile and clears the marker
/// from the organization's other profiles.
#[test]
fn profile_set_default_alias_marks_profile_and_clears_siblings() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config_with_defaults(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
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
        .stdout(predicate::str::contains(
            "Marked 'alias-a' as the default alias for organization 'org-dup-test' \
             (duplicates: alias-b).",
        ));

    let saved = fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    let table: toml::Table = toml::from_str(&saved).unwrap();
    let orgs = table["orgs"].as_table().unwrap();
    assert_eq!(orgs["alias-a"]["default_alias"], toml::Value::Boolean(true));
    assert!(
        !orgs["alias-b"]
            .as_table()
            .unwrap()
            .contains_key("default_alias"),
        "cleared marker must not be serialized: {saved}"
    );
}

/// A default alias is only meaningful while duplicates exist; a
/// non-duplicated organization is refused.
#[test]
fn profile_set_default_alias_errors_without_duplicates() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-solo")], Some("alias-a"));

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
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
    );

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("profile")
        .arg("set-default-alias")
        .arg("--org")
        .arg("org-dup-test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "'org-dup-test' is an organization ID; pass the profile alias to mark as its \
             default (one of: alias-a, alias-b)",
        ));
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
