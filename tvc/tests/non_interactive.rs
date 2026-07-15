//! Regression tests for non-interactive mode for CI
//!
//! When `TVC_NON_INTERACTIVE=1` is set, every command that would otherwise
//! prompt must fail fast with a clear "flag X is required in non-interactive
//! mode" error instead of hanging.
//!
//! Commands join this fence as they gain prompting behavior.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::turnkey::{Config, KeyCurve, OrgConfig, StoredApiKey};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const LOCAL_API_BASE_URL: &str = "http://127.0.0.1:1";

fn generated_api_key() -> (String, String) {
    let stamper = TurnkeyP256ApiKey::generate();
    (
        hex::encode(stamper.compressed_public_key()),
        hex::encode(stamper.private_key()),
    )
}

fn write_config(
    home: &TempDir,
    api_key_path: std::path::PathBuf,
    operator_key_path: std::path::PathBuf,
    last_operator_ids: Vec<String>,
) {
    let turnkey_dir = home.path().join(".config").join("turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path,
                operator_key_path,
                api_base_url: LOCAL_API_BASE_URL.to_string(),
            },
        )]),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::from([("test".to_string(), last_operator_ids)]),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        toml::to_string_pretty(&config).unwrap(),
    )
    .unwrap();
}

fn write_api_key(path: &std::path::Path) {
    let (public_key, private_key) = generated_api_key();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        path,
        serde_json::to_string_pretty(&StoredApiKey {
            public_key,
            private_key,
            curve: KeyCurve::P256,
        })
        .unwrap(),
    )
    .unwrap();
}

#[test]
fn login_without_org_errors_when_non_interactive_forced() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--org is required in non-interactive mode",
        ))
        .stderr(predicate::str::contains(NON_INTERACTIVE_ENV));
}

#[test]
fn login_without_org_errors_with_non_interactive_flag() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--non-interactive")
        .arg("login")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--org is required in non-interactive mode",
        ))
        .stderr(predicate::str::contains("--non-interactive"));
}

#[test]
fn login_non_interactive_requires_existing_api_key_before_generating_one() {
    let temp = TempDir::new().unwrap();
    let org_dir = temp
        .path()
        .join(".config")
        .join("turnkey")
        .join("orgs")
        .join("test");
    let api_key_path = org_dir.join("api_key.json");
    let operator_key_path = org_dir.join("operator.json");
    write_config(&temp, api_key_path.clone(), operator_key_path, vec![]);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("login")
        .arg("--org")
        .arg("test")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API key"))
        .stderr(predicate::str::contains("non-interactive"));

    assert!(
        !api_key_path.exists(),
        "non-interactive login must not generate an API key"
    );
}

#[test]
fn approve_without_skip_interactive_errors_when_non_interactive_forced() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--skip-post")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--dangerous-skip-interactive is required in non-interactive mode",
        ))
        .stderr(predicate::str::contains(NON_INTERACTIVE_ENV));
}

#[test]
fn deploy_init_interactive_conflicts_with_non_interactive_env() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .current_dir(temp.path())
        .arg("deploy")
        .arg("init")
        .arg("--interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE",
        ));
}

#[test]
fn deploy_init_interactive_conflicts_with_non_interactive_flag() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .current_dir(temp.path())
        .arg("deploy")
        .arg("init")
        .arg("--interactive")
        .arg("--non-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE",
        ));
}

#[test]
fn non_interactive_env_zero_does_not_force_non_interactive_mode() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "0")
        .current_dir(temp.path())
        .arg("deploy")
        .arg("init")
        .arg("--interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--interactive requires a TTY"))
        .stderr(predicate::str::contains("conflicts").not());
}

#[test]
fn invalid_non_interactive_env_value_errors_during_cli_parse() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "definitely")
        .arg("login")
        .arg("--org")
        .arg("default")
        .assert()
        .failure()
        .stderr(predicate::str::contains("value was not a boolean"));
}

#[test]
fn app_init_interactive_conflicts_with_non_interactive_env() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .current_dir(temp.path())
        .arg("app")
        .arg("init")
        .arg("--interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--interactive conflicts with --non-interactive or TVC_NON_INTERACTIVE",
        ));
}

/// `deploy create` with no config file and no required fields can't prompt for
/// the missing values, so it bails naming every field the user still has to set.
#[test]
fn deploy_create_without_required_fields_bails_naming_each_field() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("create")
        .assert()
        .failure()
        .stderr(predicate::str::contains("app_id"))
        .stderr(predicate::str::contains("pivot_container_image_url"))
        .stderr(predicate::str::contains("pivot_path"))
        .stderr(predicate::str::contains("expected_pivot_digest"));
}

/// All required fields are filled from the config file, but the pull-secret
/// sentinel is still present. Non-interactive mode can't prompt the user about
/// it, so the resolve bails rather than silently shipping the sentinel to the
/// API or mutating the config.
#[test]
fn deploy_create_pull_secret_placeholder_bails_when_non_interactive() {
    let temp = TempDir::new().unwrap();

    // Every required field filled; pivotContainerEncryptedPullSecret left as the
    // init-time sentinel that the user must resolve to null (public) or a real
    // encrypted secret (private).
    let config = r#"{
        "appId": "file-app-id",
        "qosVersion": "file-qos",
        "pivotContainerImageUrl": "file-image",
        "pivotPath": "file-path",
        "pivotArgs": ["a", "b"],
        "expectedPivotDigest": "file-digest",
        "pivotContainerEncryptedPullSecret": "<REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC>",
        "healthCheckType": "TVC_HEALTH_CHECK_TYPE_HTTP",
        "healthCheckPort": 4000,
        "publicIngressPort": 5000
    }"#;
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(config.as_bytes()).unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("create")
        .arg("--config-file")
        .arg(file.path())
        .assert()
        .failure()
        // names the offending field ...
        .stderr(predicate::str::contains(
            "pivotContainerEncryptedPullSecret",
        ))
        // ... and points the user at the resolution flag.
        .stderr(predicate::str::contains("--pivot-pull-secret"));
}

/// Guardrail: `--dangerous-skip-interactive` continues to bypass prompts
/// cleanly even with `TVC_NON_INTERACTIVE=1` set.
#[test]
fn approve_dangerous_skip_interactive_bypasses_prompts_when_non_interactive_forced() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn approve_non_interactive_requires_operator_id_when_saved_ids_are_ambiguous() {
    let temp = TempDir::new().unwrap();
    let org_dir = temp
        .path()
        .join(".config")
        .join("turnkey")
        .join("orgs")
        .join("test");
    let api_key_path = org_dir.join("api_key.json");
    let operator_key_path = org_dir.join("operator.json");
    write_config(
        &temp,
        api_key_path.clone(),
        operator_key_path,
        vec!["operator-1".to_string(), "operator-2".to_string()],
    );
    write_api_key(&api_key_path);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--manifest-id")
        .arg("manifest-id")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--operator-id"))
        .stderr(predicate::str::contains("multiple"));
}

#[test]
fn deploy_init_template_does_not_require_readable_existing_config() {
    let temp = TempDir::new().unwrap();
    let turnkey_dir = temp.path().join(".config").join("turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();
    fs::write(turnkey_dir.join("tvc.config.toml"), "not valid toml").unwrap();

    let output = temp.path().join("deploy.json");
    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("init")
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Port guidance:"))
        .stdout(predicate::str::contains(
            "Use the same port for both unless your binary exposes health checks",
        ));

    assert!(output.exists(), "deploy init should write the template");
}
