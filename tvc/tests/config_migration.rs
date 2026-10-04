//! End-to-end config-migration behavior through the real binary: any
//! config-loading command migrates eagerly, migration notes reach stderr,
//! the interrupted-migration fence blocks with instructions, and the
//! instructed manual fixes actually recover.
//!
//! The vehicle command is `keys backup-operator-key --output <path>`: it
//! loads the config right after its non-interactive input check and touches
//! no network, so every run deterministically exercises the load path.

mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";
const ORG_ID: &str = "00000000-0000-0000-0000-000000000001";

fn config_path(home: &Path) -> PathBuf {
    home.join(".config/turnkey/tvc.config.toml")
}

fn backup_path(home: &Path) -> PathBuf {
    home.join(".config/turnkey/tvc.config.toml.backup")
}

fn write_config(home: &Path, contents: &str) {
    let dir = home.join(".config/turnkey");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("tvc.config.toml"), contents).unwrap();
}

fn v0_config() -> String {
    format!(
        r#"
active_org = "default"

[orgs.default]
id = "{ORG_ID}"
api_key_path = "/keys/api.json"
operator_key_path = "/keys/operator.json"
"#
    )
}

fn run_config_loading_command(home: &Path) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("tvc")
        .env("HOME", home)
        .env(NON_INTERACTIVE_ENV, "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(home.join("backup-out.json"))
        .assert()
}

/// A v0 config is rewritten to the current schema by the first command that
/// loads it — even one that goes on to fail — and the fence backup is gone
/// afterwards.
#[test]
fn any_config_loading_command_migrates_v0_eagerly() {
    let temp = TempDir::new().unwrap();
    write_config(temp.path(), &v0_config());

    // The command itself fails later (the registered key file does not
    // exist); the migration has already been persisted by then.
    run_config_loading_command(temp.path()).failure();

    let saved = fs::read_to_string(config_path(temp.path())).unwrap();
    assert!(saved.contains("version = 2"), "{saved}");
    assert!(saved.contains(ORG_ID), "{saved}");
    assert!(!backup_path(temp.path()).exists());
}

/// A jump straight from v0 to the current schema is one load, not a chain of
/// upgrades the user must run in order: the rewritten file keeps the org,
/// its alias, and the active-org marker.
#[test]
fn v0_to_v2_is_a_single_load_with_nothing_left_behind() {
    let temp = TempDir::new().unwrap();
    write_config(temp.path(), &v0_config());

    run_config_loading_command(temp.path()).failure();

    let saved = fs::read_to_string(config_path(temp.path())).unwrap();
    let table: toml::Table = toml::from_str(&saved).unwrap();
    assert_eq!(table["version"].as_integer(), Some(2));
    assert_eq!(table["active_org"].as_str(), Some(ORG_ID));
    assert_eq!(table["aliases"]["default"].as_str(), Some(ORG_ID));
    assert!(table["orgs"][ORG_ID].is_table(), "{saved}");
}

/// Lossy migration decisions are reported on stderr, once, at migration
/// time.
#[test]
fn v1_duplicate_merge_notes_reach_stderr() {
    let temp = TempDir::new().unwrap();
    write_config(
        temp.path(),
        &format!(
            r#"
version = 1
active_org = "alias-a"

[orgs.alias-a]
id = "{ORG_ID}"
api_key_path = "/keys/a/api.json"

[orgs.alias-b]
id = "{ORG_ID}"
api_key_path = "/keys/b/api.json"
"#
        ),
    );

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains(
            "config migration: merged duplicate profile 'alias-b'",
        ));

    // The note is one-time: the migrated file loads silently afterwards.
    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("config migration").not());
}

/// An interrupted migration blocks every command with the recovery
/// instructions, and does not touch either file while blocked.
#[test]
fn an_interrupted_migration_blocks_every_command_with_instructions() {
    let temp = TempDir::new().unwrap();
    write_config(temp.path(), &v0_config());
    fs::write(backup_path(temp.path()), "pre-migration bytes").unwrap();

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"))
        .stderr(predicate::str::contains("delete"));

    assert_eq!(
        fs::read_to_string(config_path(temp.path())).unwrap(),
        v0_config(),
        "a blocked load must not rewrite the config"
    );
    assert_eq!(
        fs::read_to_string(backup_path(temp.path())).unwrap(),
        "pre-migration bytes"
    );
}

/// The crash window between parking the original and writing the new file
/// leaves only the backup; starting fresh would shadow the user's data, so
/// the same fence applies.
#[test]
fn a_backup_alone_blocks_instead_of_starting_fresh() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".config/turnkey")).unwrap();
    fs::write(backup_path(temp.path()), v0_config()).unwrap();

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"));
}

/// Manual fix for the post-write window: the config is already migrated, so
/// deleting the backup is the instructed resolution — and it works.
#[test]
fn deleting_the_backup_recovers_the_post_write_crash_window() {
    let temp = TempDir::new().unwrap();
    write_config(temp.path(), &v0_config());
    run_config_loading_command(temp.path()).failure();

    // Recreate the window: migrated config on disk, stale backup beside it.
    fs::write(backup_path(temp.path()), v0_config()).unwrap();
    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"));

    fs::remove_file(backup_path(temp.path())).unwrap();

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration").not());
}

/// Manual fix for the pre-write window: the config file is missing, so
/// restoring the backup over the config path re-runs the migration cleanly.
#[test]
fn restoring_the_backup_recovers_the_pre_write_crash_window() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join(".config/turnkey")).unwrap();
    fs::write(backup_path(temp.path()), v0_config()).unwrap();
    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"));

    fs::rename(backup_path(temp.path()), config_path(temp.path())).unwrap();

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration").not());

    let saved = fs::read_to_string(config_path(temp.path())).unwrap();
    assert!(saved.contains("version = 2"), "{saved}");
    assert!(!backup_path(temp.path()).exists());
}

/// A config written by a newer tvc is refused by name and left untouched.
#[test]
fn a_newer_config_version_is_refused_and_left_untouched() {
    let temp = TempDir::new().unwrap();
    let contents = format!("version = 3\n\n[orgs.{ORG_ID}]\napi_key_path = \"/keys/api.json\"\n");
    write_config(temp.path(), &contents);

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains(
            "config written by a newer tvc (version 3)",
        ));

    assert_eq!(
        fs::read_to_string(config_path(temp.path())).unwrap(),
        contents
    );
}

/// The config-schema migration is eager, but the key-directory migration is
/// interactive-only: a non-interactive command migrates the file schema and
/// keeps reading key files from the legacy alias-keyed directory it still
/// points at.
#[test]
fn non_interactive_commands_use_legacy_directories_without_moving_them() {
    let temp = TempDir::new().unwrap();
    let legacy_dir = temp.path().join(".config/turnkey/orgs/alias-a");
    let id_dir = temp.path().join(".config/turnkey/orgs").join(ORG_ID);
    write_config(
        temp.path(),
        &format!(
            r#"
version = 1
active_org = "alias-a"

[orgs.alias-a]
id = "{ORG_ID}"
api_key_path = "{api_key}"

[[orgs.alias-a.operators]]
name = "default"
kind = "local"
key_path = "{operator_key}"
"#,
            api_key = legacy_dir.join("api_key.json").display(),
            operator_key = legacy_dir.join("operator.json").display(),
        ),
    );
    common::write_profile_key_files(temp.path(), "alias-a");

    run_config_loading_command(temp.path()).success();

    // Schema migrated, directories untouched, config still points at the
    // legacy layout.
    let saved = fs::read_to_string(config_path(temp.path())).unwrap();
    assert!(saved.contains("version = 2"), "{saved}");
    assert!(legacy_dir.join("operator.json").exists());
    assert!(!id_dir.exists());
    assert!(
        saved.contains(legacy_dir.join("api_key.json").to_str().unwrap()),
        "{saved}"
    );
}
