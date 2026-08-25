use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn secret_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("secret");
    (temp, cmd)
}

fn secret_export_cmd() -> (TempDir, assert_cmd::Command) {
    let (temp, mut cmd) = secret_cmd();
    cmd.arg("export");
    (temp, cmd)
}

#[test]
fn secret_help_lists_export() {
    let (_temp, mut cmd) = secret_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("export"));
}

#[test]
fn secret_export_help_lists_expected_flags() {
    let (_temp, mut cmd) = secret_export_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--secret-id <SECRET_ID>"))
        .stdout(predicate::str::contains("--output-file <PATH>"))
        .stdout(predicate::str::contains("--signer-public-key <HEX>"));
}

#[test]
fn secret_export_requires_secret_id() {
    let (_temp, mut cmd) = secret_export_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--secret-id <SECRET_ID>"));
}
