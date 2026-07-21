use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn fixture_seed_hex() -> String {
    fs::read_to_string("fixtures/seed.hex")
        .unwrap()
        .trim()
        .to_string()
}

#[test]
fn approve_requires_source() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--dry-run")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains("manifest source is required"));
}

#[test]
fn approve_without_explicit_seed_requires_an_active_org() {
    let temp = TempDir::new().unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No active organization"));
}

#[test]
fn dangerous_approve_with_seed_path() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manifest approval quorum reached").not());
}

#[test]
fn dangerous_approve_with_seed_value() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg(fixture_seed_hex())
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn dangerous_approve_with_0x_prefixed_seed_value() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg(format!("0x{}", fixture_seed_hex()))
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .success();
}

#[test]
fn dangerous_approve_with_seed_env() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .env("TVC_OPERATOR_SEED", fixture_seed_hex())
        .assert()
        .success();
}

#[test]
fn operator_seed_flags_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg(fixture_seed_hex())
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--operator-seed and --operator-seed-path are mutually exclusive",
        ));
}

#[test]
fn operator_seed_rejects_a_non_hex_value() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg("not-a-hex-seed")
        .arg("--dangerous-skip-interactive")
        .arg("--skip-post")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid value 'not-a-hex-seed' for '--operator-seed <HEX_SEED>'",
        ));
}

#[test]
fn manifest_and_deploy_id_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--deploy-id")
        .arg("some-deploy-id")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--manifest <PATH>' cannot be used with '--deploy-id <DEPLOY_ID>'",
        ));
}

/// Test that --skip-post is required when --manifest-id is not provided
#[test]
fn approve_requires_manifest_id_or_skip_post() {
    cargo_bin_cmd!("tvc")
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed-path")
        .arg("fixtures/seed.hex")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--manifest-id is required to post approval to API",
        ));
}
