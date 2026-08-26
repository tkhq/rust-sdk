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
        .stdout(predicate::str::contains("--from-file <PATH>"))
        .stdout(predicate::str::contains("--property <KEY=VALUE>"))
        .stdout(predicate::str::contains("--signer-quorum-key <HEX>"));
}

#[test]
fn secrets_set_alias_is_not_recognized() {
    let (temp, _) = secrets_cmd();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path());

    cmd.args(["secrets", "set", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn secrets_import_requires_the_name_flag() {
    let (_temp, mut cmd) = secrets_import_cmd();

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains(
            "the following required arguments were not provided",
        ))
        .stderr(predicate::str::contains("--name <NAME>"));
}

#[test]
fn secrets_import_rejects_a_positional_name() {
    let (_temp, mut cmd) = secrets_import_cmd();

    cmd.arg("db-password")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn secrets_import_rejects_malformed_properties() {
    let (temp, mut cmd) = secrets_import_cmd();
    let value_file = temp.path().join("value");
    std::fs::write(&value_file, "hunter2\n").unwrap();

    cmd.args(["--name", "db-password"])
        .arg("--from-file")
        .arg(&value_file)
        .args(["--property", "environment"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("KEY=VALUE"));
}

#[test]
fn secrets_import_reads_piped_stdin_before_any_network_work() {
    let (_temp, mut cmd) = secrets_import_cmd();

    // An empty piped value proves the stdin path runs (and fails) before the
    // command ever needs config or network access.
    cmd.args(["--name", "db-password"])
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("secret value from stdin is empty"));
}
