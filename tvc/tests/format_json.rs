//! Integration coverage for the global `--format json` flag.
//!
//! Step 1 verifies the cross-cutting machinery: the flag is accepted globally,
//! and any runtime failure is rendered as a JSON error envelope on stdout
//! (rather than a bare anyhow string on stderr). Per-command success-path JSON
//! lands in later steps as each command adopts the `Emitter`.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use serde_json::Value;
use tempfile::TempDir;

/// A runtime failure (here: `deploy status` with no auth configured) emits a
/// parseable `{"error": ...}` envelope on stdout and exits non-zero.
#[test]
fn json_runtime_error_is_an_error_envelope_on_stdout() {
    let temp = TempDir::new().unwrap();

    let output = cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .arg("--format")
        .arg("json")
        .arg("deploy")
        .arg("status")
        .arg("--deploy-id")
        .arg("deploy-does-not-matter")
        .output()
        .unwrap();

    assert!(!output.status.success(), "command should fail without auth");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout must be a JSON envelope, got {stdout:?}: {e}"));
    assert!(
        value.get("error").and_then(Value::as_str).is_some(),
        "envelope must carry a string `error` field: {value}"
    );
}

/// The flag also works after the subcommand (`global = true`).
#[test]
fn format_flag_is_accepted_after_subcommand() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", temp.path())
        .env("TVC_NON_INTERACTIVE", "1")
        .arg("deploy")
        .arg("status")
        .arg("--deploy-id")
        .arg("anything")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"error\""));
}

/// An invalid `--format` value is rejected by clap before dispatch.
#[test]
fn invalid_format_value_is_rejected() {
    cargo_bin_cmd!("tvc")
        .arg("--format")
        .arg("yaml")
        .arg("deploy")
        .arg("status")
        .arg("--deploy-id")
        .arg("anything")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'yaml'"));
}
