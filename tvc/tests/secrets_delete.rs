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
fn secrets_help_lists_delete() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn secrets_delete_help_explains_yes() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.args(["delete", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("cannot be undone"));
}

#[test]
fn secrets_delete_requires_an_id() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.args(["delete", "--yes"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--id"));
}

#[test]
fn secrets_delete_non_interactive_requires_yes() {
    let (_temp, mut cmd) = secrets_cmd();

    cmd.args(["delete", "--id", "secret-abc", "--non-interactive"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--yes is required in non-interactive mode",
        ));
}
