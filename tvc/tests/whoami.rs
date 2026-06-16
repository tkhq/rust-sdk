use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;

const ENV_ORG_ID: &str = "TVC_ORG_ID";
const ENV_API_BASE_URL: &str = "TVC_API_BASE_URL";
const ENV_API_KEY_PUBLIC: &str = "TVC_API_KEY_PUBLIC";
const ENV_API_KEY_PRIVATE: &str = "TVC_API_KEY_PRIVATE";

fn generated_api_key() -> (String, String) {
    let stamper = TurnkeyP256ApiKey::generate();
    (
        hex::encode(stamper.compressed_public_key()),
        hex::encode(stamper.private_key()),
    )
}

#[test]
fn top_level_help_includes_whoami() {
    cargo_bin_cmd!("tvc")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("whoami"));
}

#[test]
fn whoami_help_describes_command() {
    cargo_bin_cmd!("tvc")
        .arg("whoami")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Print the current authenticated Turnkey identity",
        ));
}

#[test]
fn whoami_without_active_org_shows_login_guidance() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env("HOME", temp.path())
        .arg("whoami")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "No active organization. Run `tvc login` first.",
        ));
}

#[test]
fn whoami_uses_complete_env_auth_before_network_request() {
    let (public_key, private_key) = generated_api_key();

    cargo_bin_cmd!("tvc")
        .env_clear()
        .env(ENV_ORG_ID, "org-env")
        .env(ENV_API_BASE_URL, "http://127.0.0.1:1")
        .env(ENV_API_KEY_PUBLIC, public_key)
        .env(ENV_API_KEY_PRIVATE, private_key)
        .arg("whoami")
        .assert()
        .failure()
        .stderr(predicate::str::contains("whoami request failed"))
        .stderr(predicate::str::contains("partial env var auth").not())
        .stderr(predicate::str::contains("No active organization").not());
}
