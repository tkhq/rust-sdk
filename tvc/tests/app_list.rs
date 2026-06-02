use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn app_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("app");
    (temp, cmd)
}

fn app_list_cmd() -> (TempDir, assert_cmd::Command) {
    let (temp, mut cmd) = app_cmd();
    cmd.arg("list");
    (temp, cmd)
}

#[test]
fn app_help_lists_list() {
    let (_temp, mut cmd) = app_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn app_list_help_lists_expected_flags() {
    let (_temp, mut cmd) = app_list_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--name <NAME>"));
}

#[test]
fn app_list_name_filter_accepted() {
    let (_temp, mut cmd) = app_list_cmd();

    // Fails on auth (no credentials), not on arg parsing.
    cmd.arg("--name")
        .arg("my-app")
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name").not());
}
