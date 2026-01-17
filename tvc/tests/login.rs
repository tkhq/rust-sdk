use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn login_empty_org_id_fails() {
    let temp = TempDir::new().unwrap();

    // User enters empty org ID
    let input = "\n";

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("login")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Organization ID is required"));
}
