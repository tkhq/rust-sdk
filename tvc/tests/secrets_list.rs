use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

fn secrets_cmd() -> (TempDir, assert_cmd::Command) {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path()).arg("secrets");
    (temp, cmd)
}

#[test]
fn secrets_help_lists_list() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

#[test]
fn secrets_list_help_explains_metadata_only() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("metadata"));
}

#[test]
fn secrets_ls_alias_is_not_recognized() {
    let (temp, _) = secrets_cmd();
    let mut cmd = cargo_bin_cmd!("tvc");
    cmd.env_clear().env("HOME", temp.path());

    cmd.args(["secrets", "ls", "--help"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
