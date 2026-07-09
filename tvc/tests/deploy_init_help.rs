use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn deploy_init_help_documents_from_deployment_flag() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--from-deployment"))
        .stdout(predicate::str::contains("TVC_FROM_DEPLOYMENT"))
        .stdout(predicate::str::contains("expected pivot digest"));
}

#[test]
fn deploy_init_help_documents_interactive_and_output_flags() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--interactive"))
        .stdout(predicate::str::contains("--output"));
}
