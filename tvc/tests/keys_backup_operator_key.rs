mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const ORG_HOSTED_ONLY: &str = "88888888-8888-4888-8888-888888888888";

fn operator_key_path(home: &TempDir, alias: &str) -> PathBuf {
    home.path()
        .join(".config/turnkey/orgs")
        .join(alias)
        .join("operator.json")
}

/// A hosted-only org has nothing exportable: the failure explains why
/// instead of leaving a bare missing-operator error.
#[test]
fn hosted_only_org_explains_there_is_no_key_file_to_back_up() {
    let temp = TempDir::new().unwrap();
    common::write_hosted_only_config(temp.path(), "hosted-org", ORG_HOSTED_ONLY);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(temp.path().join("backup.json"))
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "org 'hosted-org' has no local operator key file to back up",
        ))
        .stderr(predicate::str::contains(
            "held by Turnkey and cannot be exported",
        ));
}

#[test]
fn backs_up_with_org_and_output() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    let operator_public_key = common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("backups/operator-backup.json");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--org")
        .arg("alias-a")
        .arg("--output")
        .arg(&destination)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operator key backed up!"))
        .stdout(predicate::str::contains(operator_public_key.to_string()));

    assert_eq!(
        fs::read(&destination).unwrap(),
        fs::read(operator_key_path(&temp, "alias-a")).unwrap()
    );
}

#[test]
fn defaults_to_active_org() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(&destination)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operator key backed up!"));

    assert!(destination.exists());
}

#[test]
fn existing_destination_requires_overwrite() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");
    fs::write(&destination, "previous backup").unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(&destination)
        .assert()
        .failure()
        .stderr(predicate::str::contains("pass --overwrite to replace it"));

    assert_eq!(fs::read_to_string(&destination).unwrap(), "previous backup");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(&destination)
        .arg("--overwrite")
        .assert()
        .success();

    assert_eq!(
        fs::read(&destination).unwrap(),
        fs::read(operator_key_path(&temp, "alias-a")).unwrap()
    );
}

// The harness runs the binary with piped stdin, so with TVC_NON_INTERACTIVE
// unset these exercise the no-TTY half of the prompt fence rather than the
// explicit non-interactive one (covered in tests/non_interactive.rs).
#[test]
fn piped_stdin_without_output_errors() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .arg("keys")
        .arg("backup-operator-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--output is required in non-interactive mode",
        ));
}

#[test]
fn piped_stdin_existing_destination_requires_overwrite_flag() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");
    fs::write(&destination, "previous backup").unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(&destination)
        .assert()
        .failure()
        .stderr(predicate::str::contains("pass --overwrite to replace it"));

    assert_eq!(fs::read_to_string(&destination).unwrap(), "previous backup");
}

#[test]
fn missing_operator_key_file_errors() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(temp.path().join("operator-backup.json"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("No operator key found at"))
        .stderr(predicate::str::contains("Run `tvc login` first."));
}

#[test]
fn json_message_format_emits_reason_tag() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--message-format")
        .arg("json")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(temp.path().join("operator-backup.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#""reason":"operator_key_backed_up""#,
        ))
        .stdout(predicate::str::contains(r#""alias":"alias-a""#));
}
