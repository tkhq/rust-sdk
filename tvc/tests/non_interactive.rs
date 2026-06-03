//! Regression tests for non-interactive mode for CI
//!
//! When `TVC_NON_INTERACTIVE=1` is set, every command that would otherwise
//! prompt must fail fast with a clear "flag X is required in non-interactive
//! mode" error instead of hanging.
//!
//! Commands join this fence as they gain prompting behavior.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

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
fn approve_without_skip_interactive_errors_when_non_interactive_forced() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
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
            "--interactive conflicts with TVC_NON_INTERACTIVE",
        ));
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
            "--interactive conflicts with TVC_NON_INTERACTIVE",
        ));
}

/// `deploy create` with no config file and no required flags can't prompt for
/// the missing values, so it bails naming every flag the user still has to set.
#[test]
fn deploy_create_without_required_flags_bails_naming_each_flag() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("deploy")
        .arg("create")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--app-id"))
        .stderr(predicate::str::contains("--qos-version"))
        .stderr(predicate::str::contains("--pivot-image-url"))
        .stderr(predicate::str::contains("--pivot-path"))
        .stderr(predicate::str::contains("--expected-pivot-digest"));
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
        .arg("--operator-seed")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success();
}
