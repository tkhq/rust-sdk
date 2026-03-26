use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_shows_global_flags() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-input"))
        .stdout(predicate::str::contains("--quiet"));
}

#[test]
fn quiet_flag_recognized() {
    cargo_bin_cmd!("tvc")
        .arg("--quiet")
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn no_input_flag_recognized() {
    cargo_bin_cmd!("tvc")
        .arg("--no-input")
        .arg("deploy")
        .arg("approve")
        .arg("--dry-run")
        .arg("--yes")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest source is required"));
}
