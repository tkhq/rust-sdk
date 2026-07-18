use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::turnkey::{Config, KeyCurve, OrgConfig, StoredApiKey};

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
        .stdout(predicate::str::contains("TVC_API_BASE_URL"));
}

/// When the org ID cannot be verified against the API (bad ID, wrong creds, or
/// an unreachable base URL), the login command must:
///   1. Surface a clear, user-actionable error message that names the org ID
///      and points at the Turnkey dashboard as the canonical source of truth.
///   2. Refuse to persist any config changes to disk — leaving a partially
///      valid entry pointing at an unverified org is exactly the bug this
///      guards against.
///
/// This exercises the reorder that moved `config.save()` to *after*
/// `verify_credentials` in `execute_login`. We simulate whoami failure by
/// pointing the org at an unroutable local port (`http://127.0.0.1:1`).
#[test]
fn login_rejects_unverifiable_org_before_persisting_config() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config").join("turnkey");
    let org_dir = turnkey_dir.join("orgs").join("test");
    let api_key_path = org_dir.join("api_key.json");
    let operator_key_path = org_dir.join("operator.json");
    fs::create_dir_all(&org_dir).unwrap();

    // Seed a real API key on disk so we get past the "generate + wait for
    // dashboard registration" step and reach the whoami call. The key material
    // itself is arbitrary; whoami is what we're exercising.
    let stamper = TurnkeyP256ApiKey::generate();
    let public_key = hex::encode(stamper.compressed_public_key());
    let private_key = hex::encode(stamper.private_key());
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

    // Point the org at an unreachable local port so the whoami round-trip is
    // guaranteed to fail. Leave `active_org = None` so that a successful save
    // would visibly change the file (making the "did not persist" assertion
    // meaningful).
    let config = Config {
        active_org: None,
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-bogus".to_string(),
                api_key_path,
                operator_key_path,
                api_base_url: "http://127.0.0.1:1".to_string(),
            },
        )]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
    };
    let config_path = turnkey_dir.join("tvc.config.toml");
    let original_config_bytes = toml::to_string_pretty(&config).unwrap();
    fs::write(&config_path, &original_config_bytes).unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("login")
        .arg("--non-interactive")
        .arg("--org")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "organization ID org-bogus could not be verified",
        ))
        .stderr(predicate::str::contains(
            "copy the org ID from the Turnkey dashboard home page",
        ));

    // Verification failed, so the config file must be byte-for-byte unchanged
    // (in particular, `active_org` was never flipped to "test").
    let post_bytes = fs::read_to_string(&config_path).unwrap();
    assert_eq!(
        post_bytes, original_config_bytes,
        "config file must not be modified when org verification fails"
    );
}
