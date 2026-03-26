use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn login_empty_org_id_fails() {
    let temp = TempDir::new().unwrap();

    let input = "\n";

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("login")
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Organization ID is required"));
}

#[test]
fn login_no_input_without_org_fails() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No organization specified in non-interactive mode",
        ));
}

#[test]
fn login_no_input_with_org_id_creates_config() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .arg("--alias")
        .arg("test")
        .arg("--api-env")
        .arg("prod")
        .arg("--skip-api-key-wait")
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8(result.stderr).expect("not utf8");

    // Should proceed past org creation to credential verification
    // (which fails since there's no real API, but the point is it didn't hang)
    assert!(
        stderr.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed past org creation, got: {stderr}"
    );
}
