use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::turnkey::{Config, KeyCurve, OrgConfig, StoredApiKey};

const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";
const LOCAL_API_BASE_URL: &str = "http://127.0.0.1:1";

fn app_status_cmd() -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .arg("app")
        .arg("status")
        .arg("--app-id")
        .arg("app_test");
    cmd
}

fn generated_api_key() -> (String, String) {
    let stamper = TurnkeyP256ApiKey::generate();
    (
        hex::encode(stamper.compressed_public_key()),
        hex::encode(stamper.private_key()),
    )
}

#[test]
fn env_auth_accepts_all_three_required_vars() {
    let (public_key, private_key) = generated_api_key();

    app_status_cmd()
        .env(ENV_ORG_ID, "org-env")
        .env(ENV_API_KEY_PUBLIC, public_key)
        .env(ENV_API_KEY_PRIVATE, private_key)
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to fetch app status"))
        .stderr(predicate::str::contains("partial env var auth").not())
        .stderr(predicate::str::contains("failed to load API key").not());
}

#[test]
fn env_auth_rejects_two_required_vars() {
    let (public_key, _) = generated_api_key();

    app_status_cmd()
        .env(ENV_ORG_ID, "org-env")
        .env(ENV_API_KEY_PUBLIC, public_key)
        .assert()
        .failure()
        .stderr(predicate::str::contains("partial env var auth"))
        .stderr(predicate::str::contains(ENV_API_KEY_PRIVATE));
}

#[test]
fn env_auth_rejects_one_required_var() {
    app_status_cmd()
        .env(ENV_ORG_ID, "org-env")
        .assert()
        .failure()
        .stderr(predicate::str::contains("partial env var auth"))
        .stderr(predicate::str::contains(ENV_API_KEY_PUBLIC))
        .stderr(predicate::str::contains(ENV_API_KEY_PRIVATE));
}

#[test]
fn auth_falls_back_to_disk_config_when_required_env_vars_are_unset() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config").join("turnkey");
    let org_dir = turnkey_dir.join("orgs").join("test");
    let api_key_path = org_dir.join("api_key.json");
    let operator_key_path = org_dir.join("operator.json");
    fs::create_dir_all(&org_dir).unwrap();

    let (public_key, private_key) = generated_api_key();
    fs::write(
        &api_key_path,
        serde_json::to_string_pretty(&StoredApiKey {
            public_key,
            private_key,
            curve: KeyCurve::P256,
        })
        .unwrap(),
    )
    .unwrap();

    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-from-disk".to_string(),
                api_key_path,
                operator_key_path,
                api_base_url: LOCAL_API_BASE_URL.to_string(),
                passkey_session_path: None,
            },
        )]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
    };
    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        toml::to_string_pretty(&config).unwrap(),
    )
    .unwrap();

    app_status_cmd()
        .env("HOME", temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to fetch app status"))
        .stderr(predicate::str::contains("No active organization").not());
}
