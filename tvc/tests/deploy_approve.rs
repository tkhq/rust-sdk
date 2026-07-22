use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use qos_p256::P256Pair;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, LocalOperatorRecord, OperatorKind, OperatorRecord,
    OperatorRecordKind, OrgConfig, StoredQosOperatorKey,
};
use uuid::Uuid;

fn fixture_seed_hex() -> String {
    fs::read_to_string("fixtures/seed.hex")
        .unwrap()
        .trim()
        .to_string()
}

const HOSTED_OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
const LOCAL_OPERATOR_ID: &str = "33333333-3333-4333-8333-333333333333";

fn write_config(home: &TempDir, config: &Config) {
    let config_dir = home.path().join(".config/turnkey");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(
        config_dir.join("tvc.config.toml"),
        format!(
            r#"version = 1
{}"#,
            toml::to_string_pretty(config).unwrap()
        ),
    )
    .unwrap();
}

fn write_hosted_config(home: &TempDir) {
    let public = P256Pair::generate().unwrap().public_key().to_bytes();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: home.path().join("api-key.json"),
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![OperatorRecord {
                    name: "hosted".to_string(),
                    kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                        operator_id: Uuid::parse_str(HOSTED_OPERATOR_ID).unwrap(),
                        wallet_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
                        path: "m/5527107'/0'/0'".to_string(),
                        encrypt_public_key: hex::encode(&public[..65]),
                        sign_public_key: hex::encode(&public[65..]),
                        extra: toml::Table::new(),
                    }),
                }],
                extra: toml::Table::new(),
            },
        )]),
        ..Config::default()
    };
    write_config(home, &config);
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
fn hosted_dry_run_does_not_require_operator_id() {
    let temp = TempDir::new().unwrap();
    write_hosted_config(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--dry-run")
        .arg("--dangerous-skip-interactive")
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry run complete"));
}

#[test]
fn hosted_approval_requires_explicit_operator_id() {
    let temp = TempDir::new().unwrap();
    write_hosted_config(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--skip-post")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--operator-id is required to approve with a hosted operator",
        ));
}

#[test]
fn explicit_seed_rejects_hosted_operator_id() {
    let temp = TempDir::new().unwrap();
    write_hosted_config(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg(fixture_seed_hex())
        .arg("--operator-id")
        .arg(HOSTED_OPERATOR_ID)
        .arg("--skip-post")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "explicit local operator seed cannot be used with a hosted operator ID",
        ));
}

#[test]
fn hosted_operator_rejects_skip_post_before_api_activity() {
    let temp = TempDir::new().unwrap();
    write_hosted_config(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-id")
        .arg(HOSTED_OPERATOR_ID)
        .arg("--skip-post")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--skip-post is only supported for local operators",
        ));
}

#[test]
fn explicit_seed_does_not_load_malformed_config() {
    let temp = TempDir::new().unwrap();
    let config_dir = temp.path().join(".config/turnkey");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("tvc.config.toml"), "not valid toml").unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--operator-seed")
        .arg(fixture_seed_hex())
        .arg("--skip-post")
        .arg("--dangerous-skip-interactive")
        .assert()
        .success();
}

#[test]
fn malformed_saved_operator_id_is_reported() {
    let temp = TempDir::new().unwrap();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: temp.path().join("api-key.json"),
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: Vec::new(),
                extra: toml::Table::new(),
            },
        )]),
        last_operator_ids: HashMap::from([("test".to_string(), vec!["not-a-uuid".to_string()])]),
        ..Config::default()
    };
    write_config(&temp, &config);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--manifest-id")
        .arg("manifest-id")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "saved operator ID 'not-a-uuid' is not a UUID",
        ));
}

#[test]
fn malformed_registered_local_operator_id_is_reported() {
    let temp = TempDir::new().unwrap();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: temp.path().join("api-key.json"),
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: vec![OperatorRecord {
                    name: "local".to_string(),
                    kind: OperatorRecordKind::Local(LocalOperatorRecord {
                        key_path: temp.path().join("operator.json"),
                        operator_id: Some("not-a-uuid".to_string()),
                        extra: toml::Table::new(),
                    }),
                }],
                extra: toml::Table::new(),
            },
        )]),
        ..Config::default()
    };
    write_config(&temp, &config);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--skip-post")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "configured local operator ID must be a UUID",
        ));
}

#[test]
fn auto_selected_hosted_id_controls_signer_resolution_in_mixed_registry() {
    let temp = TempDir::new().unwrap();
    let public = P256Pair::generate().unwrap().public_key().to_bytes();
    let operator_key_path = temp.path().join("operator.json");
    fs::write(
        &operator_key_path,
        serde_json::to_string(&StoredQosOperatorKey {
            public_key: "unused".to_string(),
            private_key: fixture_seed_hex(),
        })
        .unwrap(),
    )
    .unwrap();
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: temp.path().join("api-key.json"),
                api_base_url: "https://api.turnkey.com".to_string(),
                default_operator_kind: OperatorKind::Local,
                operators: vec![
                    OperatorRecord {
                        name: "local".to_string(),
                        kind: OperatorRecordKind::Local(LocalOperatorRecord {
                            key_path: operator_key_path,
                            operator_id: Some(LOCAL_OPERATOR_ID.to_string()),
                            extra: toml::Table::new(),
                        }),
                    },
                    OperatorRecord {
                        name: "hosted".to_string(),
                        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                            operator_id: Uuid::parse_str(HOSTED_OPERATOR_ID).unwrap(),
                            wallet_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222")
                                .unwrap(),
                            path: "m/5527107'/0'/0'".to_string(),
                            encrypt_public_key: hex::encode(&public[..65]),
                            sign_public_key: hex::encode(&public[65..]),
                            extra: toml::Table::new(),
                        }),
                    },
                ],
                extra: toml::Table::new(),
            },
        )]),
        last_operator_ids: HashMap::from([(
            "test".to_string(),
            vec![HOSTED_OPERATOR_ID.to_string()],
        )]),
        ..Config::default()
    };
    write_config(&temp, &config);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--manifest-id")
        .arg("manifest-id")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stdout(predicate::str::contains(r#""signature""#).not())
        .stderr(predicate::str::contains("No API key found for org 'test'"));
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
