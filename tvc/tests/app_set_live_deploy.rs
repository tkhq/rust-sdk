use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

fn set_live_deploy_cmd() -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().arg("app").arg("set-live-deploy");
    cmd
}

#[test]
fn app_help_lists_set_live_deploy() {
    let mut cmd = cargo_bin_cmd!("tvc");

    cmd.env_clear()
        .arg("app")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("set-live-deploy"));
}

#[test]
fn command_help_exposes_deploy_id() {
    set_live_deploy_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn missing_deploy_id_fails() {
    set_live_deploy_cmd()
        .assert()
        .failure()
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn set_live_deploy_reaches_auth_setup() {
    set_live_deploy_cmd()
        .arg("--deploy-id")
        .arg("deploy_test")
        .env("TVC_ORG_ID", "org_env")
        .assert()
        .failure()
        .stderr(predicate::str::contains("partial env var auth"));
}
