use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{Config, StoredApiKey};

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

#[test]
fn login_help_shows_api_base_url_override() {
    cargo_bin_cmd!("tvc")
        .arg("login")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--api-base-url"))
        .stdout(predicate::str::contains("TVC_API_BASE_URL"))
        .stdout(predicate::str::contains("--create-org"))
        .stdout(predicate::str::contains("--dashboard-url"))
        .stdout(predicate::str::contains("TVC_DASHBOARD_URL"));
}

#[test]
fn login_create_org_non_interactive_prints_dashboard_handoff_and_stores_api_key() {
    let temp = TempDir::new().unwrap();

    let output = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--non-interactive")
        .arg("login")
        .arg("--create-org")
        .arg("--org")
        .arg("test")
        .assert()
        .success()
        .stdout(predicate::str::contains("https://app.turnkey.com"))
        .stdout(predicate::str::contains("magic-link"))
        .stdout(predicate::str::contains("passkey"))
        .stdout(predicate::str::contains("Public Key:"))
        .stdout(predicate::str::contains("tvc login"))
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    let config_path = temp.path().join(".config/turnkey/tvc.config.toml");
    let config: Config = toml::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    let org = config.orgs.get("test").unwrap();
    let stored_key: StoredApiKey =
        serde_json::from_str(&fs::read_to_string(&org.api_key_path).unwrap()).unwrap();

    assert_eq!(config.active_org, None);
    assert_eq!(org.id, "pending-dashboard-signup");
    assert!(stdout.contains(&stored_key.public_key));
}
