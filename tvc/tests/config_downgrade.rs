//! `config downgrade` through the real binary: it runs without loading the
//! config (so a release that cannot read the newer schema can still run it),
//! rewrites a version-2 file to version 1 with a copy of the original beside
//! it, and refuses the states where rewriting would lose or shadow data.
//!
//! The config-loading vehicle is `keys backup-operator-key --output <path>`:
//! it loads the config right after its non-interactive input check and
//! touches no network.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const ORG_ID: &str = "11111111-1111-4111-8111-111111111111";
const NEWER_VERSION_ERROR: &str = "config written by a newer tvc";

fn turnkey_dir(home: &Path) -> PathBuf {
    home.join(".config/turnkey")
}

fn config_path(home: &Path) -> PathBuf {
    turnkey_dir(home).join("tvc.config.toml")
}

fn write_config(home: &Path, contents: &str) {
    fs::create_dir_all(turnkey_dir(home)).unwrap();
    fs::write(config_path(home), contents).unwrap();
}

/// One organization under two names, with key files in the id-keyed layout
/// the version-2 release writes.
fn v2_config(home: &Path) -> String {
    let org_dir = turnkey_dir(home).join("orgs").join(ORG_ID);

    format!(
        r#"
version = 2
active_org = "{ORG_ID}"

[aliases]
work = "{ORG_ID}"
turnkey = "{ORG_ID}"

[orgs.{ORG_ID}]
api_key_path = "{api_key}"
api_base_url = "http://127.0.0.1:1"
default_operator_kind = "local"

[[orgs.{ORG_ID}.operators]]
name = "default"
kind = "local"
key_path = "{operator_key}"
"#,
        api_key = org_dir.join("api_key.json").display(),
        operator_key = org_dir.join("operator.json").display(),
    )
}

fn downgrade(home: &Path) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("tvc")
        .env("HOME", home)
        .env("TVC_NON_INTERACTIVE", "1")
        .arg("config")
        .arg("downgrade")
        .assert()
}

fn run_config_loading_command(home: &Path) -> assert_cmd::assert::Assert {
    cargo_bin_cmd!("tvc")
        .env("HOME", home)
        .env("TVC_NON_INTERACTIVE", "1")
        .arg("keys")
        .arg("backup-operator-key")
        .arg("--output")
        .arg(home.join("backup-out.json"))
        .assert()
}

#[test]
fn downgrades_a_v2_config_in_place_and_keeps_the_v2_copy() {
    let temp = TempDir::new().unwrap();
    let original = v2_config(temp.path());
    write_config(temp.path(), &original);

    downgrade(temp.path())
        .success()
        .stdout(predicate::str::contains("Rewrote"))
        .stdout(predicate::str::contains("tvc.config.toml.v2"));

    let saved = fs::read_to_string(config_path(temp.path())).unwrap();
    let table: toml::Table = toml::from_str(&saved).unwrap();
    assert_eq!(table["version"].as_integer(), Some(1), "{saved}");
    assert_eq!(table["active_org"].as_str(), Some("turnkey"), "{saved}");
    assert_eq!(
        table["orgs"]["work"]["id"].as_str(),
        Some(ORG_ID),
        "{saved}"
    );
    assert_eq!(
        table["orgs"]["turnkey"]["id"].as_str(),
        Some(ORG_ID),
        "{saved}"
    );
    assert!(!table.contains_key("aliases"), "{saved}");
    assert_eq!(
        fs::read_to_string(config_path(temp.path()).with_extension("toml.v2")).unwrap(),
        original
    );

    // The rewritten file is readable by this release, whatever schema it
    // speaks: the command fails later, on the missing key file, not on the
    // config version.
    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains(NEWER_VERSION_ERROR).not());
}

#[test]
fn json_output_reports_config_downgraded() {
    let temp = TempDir::new().unwrap();
    write_config(temp.path(), &v2_config(temp.path()));

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("--message-format")
        .arg("json")
        .arg("config")
        .arg("downgrade")
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""reason":"config_downgraded""#))
        .stdout(predicate::str::contains(r#""configPath":"#))
        .stdout(predicate::str::contains(r#""backupPath":"#));
}

#[test]
fn an_interrupted_migration_fence_blocks_the_downgrade() {
    let temp = TempDir::new().unwrap();
    let original = v2_config(temp.path());
    write_config(temp.path(), &original);
    let fence = config_path(temp.path()).with_extension("toml.backup");
    fs::write(&fence, "pre-migration bytes").unwrap();

    downgrade(temp.path())
        .failure()
        .stderr(predicate::str::contains("interrupted config migration"));

    assert_eq!(
        fs::read_to_string(config_path(temp.path())).unwrap(),
        original
    );
    assert_eq!(fs::read_to_string(&fence).unwrap(), "pre-migration bytes");
    assert!(!config_path(temp.path()).with_extension("toml.v2").exists());
}

#[test]
fn an_existing_v2_copy_blocks_the_downgrade() {
    let temp = TempDir::new().unwrap();
    let original = v2_config(temp.path());
    write_config(temp.path(), &original);
    let copy = config_path(temp.path()).with_extension("toml.v2");
    fs::write(&copy, "an earlier copy").unwrap();

    downgrade(temp.path())
        .failure()
        .stderr(predicate::str::contains("already exists"));

    assert_eq!(
        fs::read_to_string(config_path(temp.path())).unwrap(),
        original
    );
    assert_eq!(fs::read_to_string(&copy).unwrap(), "an earlier copy");
}

#[test]
fn a_v1_config_is_refused_and_left_untouched() {
    let temp = TempDir::new().unwrap();
    let original = format!(
        r#"
version = 1
active_org = "work"

[orgs.work]
id = "{ORG_ID}"
api_key_path = "/keys/api.json"
"#
    );
    write_config(temp.path(), &original);

    downgrade(temp.path())
        .failure()
        .stderr(predicate::str::contains("already at version 1"));

    assert_eq!(
        fs::read_to_string(config_path(temp.path())).unwrap(),
        original
    );
    assert!(!config_path(temp.path()).with_extension("toml.v2").exists());
}

/// A stranded user's first contact with the problem is the version refusal;
/// it has to point at the way out.
#[test]
fn the_newer_version_refusal_names_the_downgrade_command() {
    let temp = TempDir::new().unwrap();
    write_config(
        temp.path(),
        &format!("version = 3\n\n[orgs.{ORG_ID}]\napi_key_path = \"/keys/api.json\"\n"),
    );

    run_config_loading_command(temp.path())
        .failure()
        .stderr(predicate::str::contains(NEWER_VERSION_ERROR))
        .stderr(predicate::str::contains("tvc config downgrade"));
}
