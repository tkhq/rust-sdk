use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

const DEPLOYMENT_ID: &str = "33333333-3333-4333-8333-333333333333";
const OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";

#[test]
fn deploy_help_lists_provision_subcommand() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("provision"))
        .stdout(predicate::str::contains(
            "Provision one hosted quorum-key share for a deployment",
        ));
}

#[test]
fn provision_help_lists_only_singular_live_inputs() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("provision")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--deploy-id <DEPLOY_ID>"))
        .stdout(predicate::str::contains("--operator-id <OPERATOR_ID>"))
        .stdout(predicate::str::contains("--dangerous-skip-verification"))
        .stdout(predicate::str::contains("TVC_DEPLOY_ID"))
        .stdout(predicate::str::contains("TVC_OPERATOR_ID"))
        .stdout(predicate::str::contains("--provision-bundle").not())
        .stdout(predicate::str::contains("--operators").not());
}

#[test]
fn provision_requires_deployment_and_operator_ids() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .arg("deploy")
        .arg("provision")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--deploy-id <DEPLOY_ID>"))
        .stderr(predicate::str::contains("--operator-id <OPERATOR_ID>"));
}

#[test]
fn provision_reads_ids_from_environment() {
    let temp = tempfile::TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", temp.path())
        .env("TVC_DEPLOY_ID", DEPLOYMENT_ID)
        .env("TVC_OPERATOR_ID", OPERATOR_ID)
        .arg("deploy")
        .arg("provision")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"))
        .stderr(predicate::str::contains("required arguments").not());
}

#[test]
fn provision_rejects_malformed_uuids_during_parsing() {
    for (deploy_id, operator_id) in [("not-a-uuid", OPERATOR_ID), (DEPLOYMENT_ID, "not-a-uuid")] {
        cargo_bin_cmd!("tvc")
            .env_clear()
            .arg("deploy")
            .arg("provision")
            .arg("--deploy-id")
            .arg(deploy_id)
            .arg("--operator-id")
            .arg(operator_id)
            .assert()
            .failure()
            .stderr(predicate::str::contains("invalid value 'not-a-uuid'"));
    }
}
