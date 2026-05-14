use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn restore_cmd(home: &TempDir) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear()
        .env("HOME", home.path())
        .arg("deploy")
        .arg("restore");
    cmd
}

#[test]
fn restore_help_lists_deploy_id() {
    let temp = TempDir::new().unwrap();

    restore_cmd(&temp)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn restore_requires_deploy_id() {
    let temp = TempDir::new().unwrap();

    restore_cmd(&temp)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}
