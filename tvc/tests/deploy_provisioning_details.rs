use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn provisioning_details_requires_deploy_id() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("provisioning-details")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"));
}

#[test]
fn deploy_help_lists_provisioning_details_subcommand() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("provisioning-details"))
        .stdout(predicate::str::contains(
            "Get provisioning details for a deployment",
        ));
}

#[test]
fn provisioning_details_help_lists_expected_flags() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("provisioning-details")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"))
        .stdout(predicate::str::contains("--dangerous-skip-verification"))
        .stdout(predicate::str::contains("--provision-bundle-out <PATH>"));
}
