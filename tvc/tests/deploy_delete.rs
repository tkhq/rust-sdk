use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn deploy_delete_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("delete");
    (temp, cmd)
}

#[test]
fn deploy_delete_help_lists_expected_flags() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"))
        .stdout(predicate::str::contains("--dangerous-skip-confirmation"));
}

#[test]
fn deploy_delete_requires_deploy_id() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn deploy_delete_rejection_cancels_before_auth() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.arg("--deploy-id")
        .arg("deploy_test")
        .write_stdin("n\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("operation cancelled by user"))
        .stderr(predicate::str::contains("No active organization").not());
}

#[test]
fn deploy_delete_skip_confirmation_reaches_auth() {
    let (_temp, mut cmd) = deploy_delete_cmd();

    cmd.arg("--deploy-id")
        .arg("deploy_test")
        .arg("--dangerous-skip-confirmation")
        .assert()
        .failure()
        .stdout(predicate::str::contains("[y/N]").not())
        .stderr(predicate::str::contains(
            "No active organization. Run `tvc login` first.",
        ));
}
