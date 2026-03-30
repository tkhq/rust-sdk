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

#[test]
fn login_api_env_dev() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .arg("--api-env")
        .arg("dev")
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    assert!(
        stderr.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed with --api-env dev, got: {stderr}"
    );
}

#[test]
fn login_api_env_preprod() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .arg("--api-env")
        .arg("preprod")
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    assert!(
        stderr.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed with --api-env preprod, got: {stderr}"
    );
}

#[test]
fn login_api_env_local() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .arg("--api-env")
        .arg("local")
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    assert!(
        stderr.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed with --api-env local, got: {stderr}"
    );
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

    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    assert!(
        stderr.contains("Verifying credentials") || stderr.contains("whoami request failed"),
        "Expected login to proceed with default api-env, got: {stderr}"
    );
}

#[test]
fn login_quiet_suppresses_status() {
    let temp = TempDir::new().unwrap();

    let result = cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--quiet")
        .arg("--no-input")
        .arg("login")
        .arg("--org-id")
        .arg("test-org-id")
        .output()
        .expect("failed to execute");

    let stderr = String::from_utf8(result.stderr).expect("not utf8");
    // --quiet suppresses routine status messages, but manual recovery
    // instructions for first-time API key setup should still be visible.
    assert!(
        !stderr.contains("Selected org")
            && !stderr.contains("Generating")
            && !stderr.contains("Verifying credentials")
            && stderr.contains("API Key Generated!")
            && stderr.contains("Public Key:")
            && stderr.contains("Add this API key to your Turnkey dashboard:"),
        "Expected --quiet to suppress status messages, got: {stderr}"
    );
}
