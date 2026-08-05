mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const ORG_BACKUP: &str = "66666666-6666-4666-8666-666666666666";

fn operator_key_path(home: &TempDir, alias: &str) -> PathBuf {
    home.path()
        .join(".config/turnkey/orgs")
        .join(alias)
        .join("operator.json")
}

#[test]
fn backs_up_with_org_and_output() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );
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

/// Backup is read-only, so it shares login's duplicate rules: an org-ID query
/// over several profiles resolves to the `default_alias` one with a warning.
#[test]
fn resolves_duplicated_org_id_to_default_alias() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP), ("alias-b", ORG_BACKUP)],
        Some("alias-a"),
        &["alias-b"],
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    let default_public_key = common::write_profile_key_files(temp.path(), "alias-b");
    let destination = temp.path().join("operator-backup.json");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--org")
        .arg(ORG_BACKUP)
        .arg("--output")
        .arg(&destination)
        .assert()
        .success()
        .stdout(predicate::str::contains(default_public_key.to_string()))
        .stderr(predicate::str::contains("Using default profile 'alias-b'"));

    assert_eq!(
        fs::read(&destination).unwrap(),
        fs::read(operator_key_path(&temp, "alias-b")).unwrap()
    );
}

#[test]
fn defaults_to_active_org() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );
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
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );
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

#[test]
fn missing_operator_key_file_errors() {
    let temp = TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );

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
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", ORG_BACKUP)],
        Some("alias-a"),
        &[],
    );
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
