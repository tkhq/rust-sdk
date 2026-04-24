use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

// TODO(daniil): refactor these away from piped-stdin
/// Piped-stdin "new org" flow with an empty org ID bails with a clear error
/// before any API call. Also verifies that the ported `prompts::text`
/// primitive still handles piped stdin correctly.
#[test]
fn login_empty_org_id_fails() {
    let temp = TempDir::new().unwrap();

    // User enters empty org ID
    let input = "\n";

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("login")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Organization ID is required"));
}

/// When `--org <alias>` points to an alias that is not present in the local
/// config, we fail fast without entering any interactive flow. Exercises the
/// early-return branch in `select_or_create_org`.
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
