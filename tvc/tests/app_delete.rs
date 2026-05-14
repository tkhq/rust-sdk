use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn app_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("app");
    (temp, cmd)
}

fn app_delete_cmd() -> (TempDir, assert_cmd::Command) {
    let (temp, mut cmd) = app_cmd();
    cmd.arg("delete");
    (temp, cmd)
}

#[test]
fn app_help_lists_delete() {
    let (_temp, mut cmd) = app_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn app_delete_help_lists_expected_flags() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--app-id <APP_ID>"))
        .stdout(predicate::str::contains("--dangerous-skip-confirmation"));
}

#[test]
fn app_delete_requires_app_id() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--app-id <APP_ID>"));
}

#[test]
fn app_delete_rejection_cancels_before_auth() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.arg("--app-id")
        .arg("app_test")
        .write_stdin("n\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("operation cancelled by user"))
        .stderr(predicate::str::contains("No active organization").not());
}

#[test]
fn app_delete_typed_app_id_mismatch_cancels_before_auth() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.arg("--app-id")
        .arg("app_test")
        .write_stdin("y\nwrong_app\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("operation cancelled by user"))
        .stderr(predicate::str::contains("No active organization").not());
}

#[test]
fn app_delete_exact_typed_app_id_reaches_auth() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.arg("--app-id")
        .arg("app_test")
        .write_stdin("y\napp_test\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No active organization. Run `tvc login` first.",
        ))
        .stderr(predicate::str::contains("operation cancelled by user").not());
}

#[test]
fn app_delete_skip_confirmation_skips_both_prompts() {
    let (_temp, mut cmd) = app_delete_cmd();

    cmd.arg("--app-id")
        .arg("app_test")
        .arg("--dangerous-skip-confirmation")
        .write_stdin("n\nwrong_app\n")
        .assert()
        .failure()
        .stdout(predicate::str::contains("[y/N]").not())
        .stdout(predicate::str::contains("Type").not())
        .stderr(predicate::str::contains(
            "No active organization. Run `tvc login` first.",
        ))
        .stderr(predicate::str::contains("operation cancelled by user").not());
}
