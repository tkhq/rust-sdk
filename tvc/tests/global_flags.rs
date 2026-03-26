use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_shows_global_flags() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"))
        .stdout(predicate::str::contains("--no-input"))
        .stdout(predicate::str::contains("--quiet"));
}

#[test]
fn json_flag_recognized_on_subcommand() {
    // --json should be accepted on any subcommand without error
    // We use app init with a non-existent file to trigger a known error
    // but the point is --json is parsed without "unknown flag" error
    cargo_bin_cmd!("tvc")
        .arg("--json")
        .arg("app")
        .arg("init")
        .arg("--output")
        .arg("/tmp/tvc-test-nonexistent-dir/test.json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown").not());
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

#[test]
fn deploy_status_help_shows_override_flags() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--api-key-file"))
        .stdout(predicate::str::contains("--api-url"))
        .stdout(predicate::str::contains("--org-id"));
}
