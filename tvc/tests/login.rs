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
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8(result.stdout).expect("not utf8");
    let stderr = String::from_utf8(result.stderr).expect("not utf8");

    // Should proceed past org creation to credential verification
    // (which fails since there's no real API, but the point is it didn't hang)
    assert!(
        stdout.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed past org creation, got stdout: {stdout}, stderr: {stderr}"
    );
}

#[test]
fn login_org_and_org_id_conflict() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org")
        .arg("some-alias")
        .arg("--org-id")
        .arg("some-id")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--org <ORG>' cannot be used with '--org-id <ORG_ID>'",
        ));
}

#[test]
fn login_api_env_invalid_rejected() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .arg("--api-env")
        .arg("foobar")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'foobar'"));
}

#[test]
fn login_api_env_defaults_to_prod() {
    let temp = TempDir::new().unwrap();

    // Omit --api-env, should default to prod
    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8(result.stdout).expect("not utf8");
    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    assert!(
        stdout.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed with default api-env, got stdout: {stdout}, stderr: {stderr}"
    );
}

#[test]
fn login_no_input_suppresses_status() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8(result.stdout).expect("not utf8");
    // --no-input suppresses routine status messages, but manual setup
    // instructions for first-time API key setup should still be visible.
    assert!(
        !stdout.contains("Selected org")
            && !stdout.contains("Generating")
            && !stdout.contains("Verifying credentials")
            && stdout.contains("API Key Generated!")
            && stdout.contains("Public Key:")
            && stdout.contains("Add this API key to your Turnkey dashboard:"),
        "Expected --no-input to suppress status messages, got: {stdout}"
    );
}
