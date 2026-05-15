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
        .stdout(predicate::str::contains("--app-id <APP_ID>"));
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
