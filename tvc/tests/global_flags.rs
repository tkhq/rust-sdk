use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_shows_global_flags() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--no-input"));
}

#[test]
fn no_input_flag_recognized() {
    cargo_bin_cmd!("tvc")
        .arg("--no-input")
        .arg("deploy")
        .arg("approve")
        .arg("--dry-run")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest source is required"));
}
