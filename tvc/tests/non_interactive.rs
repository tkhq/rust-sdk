//! Non-interactive regression fence.
//!
//! When `TVC_NON_INTERACTIVE=1` is set, every command that would otherwise
//! prompt must fail fast with a clear "flag X is required in non-interactive
//! mode" error instead of hanging.
//!
//! Commands join this fence as they gain prompting behavior. Commit 2 covers
//! `login` and `deploy approve`.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

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
