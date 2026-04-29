use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn root_help_lists_re_encrypt_share_command() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("re-encrypt-share"))
        .stdout(predicate::str::contains(
            "Re-encrypt a share for enclave provisioning",
        ));
}

#[test]
fn re_encrypt_share_help_lists_expected_flags() {
    cargo_bin_cmd!("tvc")
        .arg("re-encrypt-share")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--share-path <SHARE_PATH>"))
        .stdout(predicate::str::contains(
            "--provision-bundle <PROVISION_BUNDLE>",
        ))
        .stdout(predicate::str::contains("--operator-seed <OPERATOR_PATH>"))
        .stdout(predicate::str::contains("--dangerous-skip-verification"))
        .stdout(predicate::str::contains(
            "--re-encrypted-out <RE_ENCRYPTED_OUT>",
        ));
}

#[test]
fn re_encrypt_share_requires_share_and_provision_bundle() {
    cargo_bin_cmd!("tvc")
        .arg("re-encrypt-share")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--share-path"))
        .stderr(predicate::str::contains("--provision-bundle"));
}
