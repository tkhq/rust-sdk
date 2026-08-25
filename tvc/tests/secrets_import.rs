use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn secrets_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("secrets");
    (temp, cmd)
}

fn secrets_import_cmd() -> (TempDir, assert_cmd::Command) {
    let (temp, mut cmd) = secrets_cmd();
    cmd.arg("import");
    (temp, cmd)
}

#[test]
fn secrets_help_lists_import() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("import"));
}

#[test]
fn secrets_import_help_lists_expected_flags() {
    let (_temp, mut cmd) = secrets_import_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--name <NAME>"))
        .stdout(predicate::str::contains("--value-file <PATH>"))
        .stdout(predicate::str::contains("--property <KEY=VALUE>"))
        .stdout(predicate::str::contains("--signer-quorum-key <HEX>"));
}

#[test]
fn secrets_import_requires_value_file() {
    let (_temp, mut cmd) = secrets_import_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--value-file <PATH>"));
}

#[test]
fn secrets_import_rejects_malformed_static_properties() {
    let (temp, mut cmd) = secrets_import_cmd();
    let value_file = temp.path().join("value");
    std::fs::write(&value_file, "hunter2\n").unwrap();

    cmd.arg("--value-file")
        .arg(&value_file)
        .args(["--property", "environment"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("KEY=VALUE"));
}
