use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn deploy_help_lists_post_share_subcommand() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("post-share"))
        .stdout(predicate::str::contains(
            "Post a re-encrypted quorum key share for a deployment",
        ));
}

#[test]
fn post_share_help_lists_expected_flags() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("post-share")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--re-encrypted-share <PATH>"))
        .stdout(predicate::str::contains(
            "--share-operator-id <SHARE_OPERATOR_ID>",
        ))
        .stdout(predicate::str::contains("--deploy-id").not());
}

#[test]
fn post_share_requires_re_encrypted_share_and_share_operator_id() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("post-share")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--re-encrypted-share <PATH>"))
        .stderr(predicate::str::contains(
            "--share-operator-id <SHARE_OPERATOR_ID>",
        ));
}
