use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn secrets_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("secrets");
    (temp, cmd)
}

fn secrets_export_cmd() -> (TempDir, assert_cmd::Command) {
    let (temp, mut cmd) = secrets_cmd();
    cmd.arg("export");
    (temp, cmd)
}

#[test]
fn secrets_help_lists_export() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("export"));
}

#[test]
fn secrets_export_help_lists_expected_flags() {
    let (_temp, mut cmd) = secrets_export_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--id <SECRET_ID>"))
        .stdout(predicate::str::contains("--name <NAME>"))
        .stdout(predicate::str::contains("--out <PATH>"))
        .stdout(predicate::str::contains("--plain"))
        .stdout(predicate::str::contains("--signer-quorum-key <HEX>"));
}

#[test]
fn secrets_get_is_an_export_alias() {
    let (temp, _) = secrets_cmd();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path());

    cmd.args(["secrets", "get", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--out <PATH>"));
}

#[test]
fn secrets_export_requires_id_or_name() {
    let (_temp, mut cmd) = secrets_export_cmd();

    cmd.assert().failure().stderr(predicate::str::contains(
        "the following required arguments were not provided",
    ));
}

#[test]
fn secrets_export_rejects_both_id_and_name() {
    let (_temp, mut cmd) = secrets_export_cmd();

    cmd.args(["--id", "secret-abc", "--name", "db-password"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}
