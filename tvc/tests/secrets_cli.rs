//! CLI surface tests for the `tvc secrets` command group: canonical flag
//! names, ratified UX guards, and pre-network validation.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn secrets_cmd(args: &[&str]) -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("secrets");
    cmd.args(args);
    (temp, cmd)
}

#[test]
fn import_help_lists_the_canonical_flags() {
    let (_temp, mut cmd) = secrets_cmd(&["import", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--name <NAME>"))
        .stdout(predicate::str::contains("--from-file <PATH>"))
        .stdout(predicate::str::contains("--property <KEY=VALUE>"))
        .stdout(predicate::str::contains("--signer-quorum-key <HEX>"));
}

#[test]
fn export_help_lists_the_canonical_flags() {
    let (_temp, mut cmd) = secrets_cmd(&["export", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--id <SECRET_ID>"))
        .stdout(predicate::str::contains("--name <NAME>"))
        .stdout(predicate::str::contains("--out <PATH>"))
        .stdout(predicate::str::contains("--plain"))
        .stdout(predicate::str::contains("--signer-quorum-key <HEX>"));
}

#[test]
fn list_help_explains_metadata_only() {
    let (_temp, mut cmd) = secrets_cmd(&["list", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("metadata"));
}

#[test]
fn delete_help_explains_yes_and_irreversibility() {
    let (_temp, mut cmd) = secrets_cmd(&["delete", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("cannot be undone"));
}

/// Only canonical command names exist: the set/get/ls aliases from earlier
/// designs must stay unrecognized.
#[test]
fn aliases_are_not_recognized() {
    for alias in ["set", "get", "ls"] {
        let (_temp, mut cmd) = secrets_cmd(&[alias, "--help"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
    }
}

#[test]
fn export_rejects_both_id_and_name() {
    let (_temp, mut cmd) = secrets_cmd(&["export", "--id", "secret-abc", "--name", "db-password"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn import_rejects_malformed_properties() {
    let (temp, mut cmd) = secrets_cmd(&["import", "--name", "db-password"]);
    let value_file = temp.path().join("value");
    std::fs::write(&value_file, "hunter2\n").unwrap();

    cmd.arg("--from-file")
        .arg(&value_file)
        .args(["--property", "environment"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("KEY=VALUE"));
}

#[test]
fn import_reads_piped_stdin_before_any_network_work() {
    let (_temp, mut cmd) = secrets_cmd(&["import", "--name", "db-password"]);

    // An empty piped value proves the stdin path runs (and fails) before the
    // command ever needs config or network access.
    cmd.write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("secret value from stdin is empty"));
}
