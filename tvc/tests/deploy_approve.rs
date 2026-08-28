mod common;

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};
use tempfile::TempDir;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::{
    GetTvcDeploymentResponse,
    external::data::v1::{TvcDeployment, TvcManifest, TvcOperator, TvcOperatorSet},
};
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, LocalOperatorRecord, OperatorKind, OperatorRecord,
    OperatorRecordKind, OrgConfig, QosOperatorPublicKey, StoredQosOperatorKey,
    YubiKeyOperatorRecord, YubiKeyRegistryEntry, YubiKeySerial,
};
use uuid::Uuid;

fn fixture_seed_hex() -> String {
    fs::read_to_string("fixtures/seed.hex")
        .unwrap()
        .trim()
        .to_string()
}

fn fixture_manifest_member_key(index: usize) -> QosOperatorPublicKey {
    let manifest: serde_json::Value =
        serde_json::from_slice(include_bytes!("../fixtures/manifest.json")).unwrap();
    manifest["manifestSet"]["members"][index]["pubKey"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

const HOSTED_OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
const LOCAL_OPERATOR_ID: &str = "33333333-3333-4333-8333-333333333333";
const DEPLOYMENT_ID: &str = "bb4c572f-0609-4b1d-b8b5-4dc83dbc89de";
const MANIFEST_ID: &str = "22222222-2222-4222-8222-222222222222";
const YUBIKEY_OPERATOR_ID: &str = "44444444-4444-4444-8444-444444444444";
const GET_DEPLOYMENT_PATH: &str = "/public/v1/query/get_tvc_deployment";

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

fn authenticated_command(home: &TempDir, api_base_url: &str) -> assert_cmd::Command {
    let stamper = TurnkeyP256ApiKey::generate();
    let mut command = cargo_bin_cmd!("tvc");
    command
        .env_clear()
        .env("HOME", home.path())
        .env("TVC_ORG_ID", "org-test")
        .env(
            "TVC_API_KEY_PUBLIC",
            hex::encode(stamper.compressed_public_key()),
        )
        .env("TVC_API_KEY_PRIVATE", hex::encode(stamper.private_key()))
        .env("TVC_API_BASE_URL", api_base_url);
    command
}

fn spawn_json_server(body: String) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        assert_eq!(
            request_line.split_whitespace().nth(1),
            Some(GET_DEPLOYMENT_PATH)
        );

        let mut content_length = 0;

        loop {
            let mut header = String::new();
            reader.read_line(&mut header).unwrap();

            if header == "\r\n" {
                break;
            }

            if let Some(value) = header
                .strip_prefix("content-length:")
                .or_else(|| header.strip_prefix("Content-Length:"))
            {
                content_length = value.trim().parse().unwrap();
            }
        }

        let mut request_body = vec![0; content_length];
        reader.read_exact(&mut request_body).unwrap();
        drop(reader);

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    });

    (format!("http://{address}"), handle)
}

fn deployment_response(
    hosted_key: QosOperatorPublicKey,
    yubikey_key: QosOperatorPublicKey,
) -> String {
    serde_json::to_string(&GetTvcDeploymentResponse {
        tvc_deployment: Some(TvcDeployment {
            id: DEPLOYMENT_ID.to_string(),
            organization_id: "org-test".to_string(),
            app_id: "app-test".to_string(),
            manifest_set: Some(TvcOperatorSet {
                id: "manifest-set-test".to_string(),
                name: "manifest-set".to_string(),
                organization_id: "org-test".to_string(),
                operators: vec![
                    TvcOperator {
                        id: HOSTED_OPERATOR_ID.to_string(),
                        name: "hosted".to_string(),
                        public_key: hosted_key.to_string(),
                        created_at: None,
                        updated_at: None,
                    },
                    TvcOperator {
                        id: YUBIKEY_OPERATOR_ID.to_string(),
                        name: "yubikey-op".to_string(),
                        public_key: yubikey_key.to_string(),
                        created_at: None,
                        updated_at: None,
                    },
                ],
                threshold: 2,
                created_at: None,
                updated_at: None,
            }),
            share_set: None,
            manifest: Some(TvcManifest {
                id: MANIFEST_ID.to_string(),
                manifest: fs::read("fixtures/manifest.json").unwrap(),
                created_at: None,
                updated_at: None,
            }),
            manifest_approvals: Vec::new(),
            qos_version: "1.0.0".to_string(),
            pivot_container: None,
            created_at: None,
            updated_at: None,
            delete: false,
            debug_mode: false,
        }),
    })
    .unwrap()
}

fn write_hosted_config(home: &TempDir) {
    let public = fixture_manifest_member_key(0);
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
                        encrypt_public_key: hex::encode(&public.as_bytes()[..65]),
                        sign_public_key: hex::encode(&public.as_bytes()[65..]),
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

/// A selected hosted signer cannot satisfy `--skip-post`: the refusal fires
/// before credential loading (the fixture deliberately has no API key on
/// disk).
#[test]
fn selected_hosted_operator_rejects_skip_post() {
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
            "--skip-post is not supported for hosted operators",
        ));
}

/// A yubikey org in a non-interactive run refuses during resolution: the PIN
/// can only be typed at a prompt. No device is touched, so this needs no USB.
#[test]
fn selected_yubikey_without_a_prompt_reports_the_pin_requirement() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config_with_public_key(
        temp.path(),
        "test",
        "org-test",
        fixture_manifest_member_key(0),
    );

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
            "a YubiKey operator needs its PIN typed at an interactive prompt",
        ));
}

/// The selected serial narrows the locally available signers; its public key
/// then derives the server operator UUID from the fetched deployment.
#[test]
fn deploy_id_and_serial_resolve_one_operator_identity_by_public_key() {
    let temp = TempDir::new().unwrap();
    let hosted_key = fixture_manifest_member_key(0);
    let yubikey_key = fixture_manifest_member_key(2);
    let serial = YubiKeySerial::from(0x01c9_5c1f);
    let config = Config {
        active_org: Some("test".to_string()),
        orgs: HashMap::from([(
            "test".to_string(),
            OrgConfig {
                id: "org-test".to_string(),
                api_key_path: temp.path().join("api-key.json"),
                api_base_url: "http://127.0.0.1:1".to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![
                    OperatorRecord {
                        name: "unrelated-hosted".to_string(),
                        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                            operator_id: Uuid::parse_str(HOSTED_OPERATOR_ID).unwrap(),
                            wallet_id: Uuid::parse_str("55555555-5555-4555-8555-555555555555")
                                .unwrap(),
                            path: "m/5527107'/0'/0'".to_string(),
                            encrypt_public_key: hex::encode(&hosted_key.as_bytes()[..65]),
                            sign_public_key: hex::encode(&hosted_key.as_bytes()[65..]),
                            extra: toml::Table::new(),
                        }),
                    },
                    OperatorRecord {
                        name: "yubikey-op".to_string(),
                        kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                            serial,
                            extra: toml::Table::new(),
                        }),
                    },
                ],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: vec![YubiKeyRegistryEntry {
            serial,
            public_key: yubikey_key,
            extra: toml::Table::new(),
        }]
        .try_into()
        .unwrap(),
        last_operator_ids: HashMap::from([(
            "test".to_string(),
            vec![
                "66666666-6666-4666-8666-666666666666".to_string(),
                "77777777-7777-4777-8777-777777777777".to_string(),
            ],
        )]),
        ..Config::default()
    };
    write_config(&temp, &config);

    let body = deployment_response(hosted_key, yubikey_key);
    let (api_base_url, server) = spawn_json_server(body);

    authenticated_command(&temp, &api_base_url)
        .args([
            "deploy",
            "approve",
            "--deploy-id",
            DEPLOYMENT_ID,
            "--serial",
            "01c95c1f",
            "--dangerous-skip-interactive",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Fetching deployment"))
        .stdout(predicate::str::contains("Manifest loaded"))
        .stdout(predicate::str::contains("Select approving operator").not())
        .stderr(predicate::str::contains(
            "a YubiKey operator needs its PIN typed at an interactive prompt",
        ))
        .stderr(predicate::str::contains("multiple configured operators").not());

    server.join().unwrap();
}

#[test]
fn an_unknown_yubikey_serial_is_rejected_before_manifest_io() {
    let temp = TempDir::new().unwrap();
    common::write_yubikey_only_config(temp.path(), "test", "org-test");

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .args(["deploy", "approve"])
        .arg("--manifest")
        .arg(temp.path().join("does-not-exist.json"))
        .args([
            "--skip-post",
            "--dangerous-skip-interactive",
            "--serial",
            "deadbeef",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no YubiKey operator has serial deadbeef",
        ))
        .stderr(predicate::str::contains("does-not-exist.json").not());
}

/// The sole configured operator whose public key belongs to the manifest set
/// is selected without a default or remembered operator ID.
#[test]
fn registered_hosted_manifest_member_is_the_post_candidate() {
    let temp = TempDir::new().unwrap();
    write_hosted_config(&temp);

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .arg("deploy")
        .arg("approve")
        .arg("--manifest")
        .arg("fixtures/manifest.json")
        .arg("--manifest-id")
        .arg("11111111-1111-4111-8111-111111111111")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No API key found for org 'test'"));
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
            "the argument '--operator-seed <HEX_SEED>' cannot be used with '--operator-id <OPERATOR_ID>'",
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
            "--skip-post is not supported for hosted operators",
        ));
}

#[test]
fn malformed_config_fails_before_dispatch_even_with_explicit_seed() {
    // Config is built once before dispatch, so a corrupt config file surfaces
    // immediately for every command — even an offline approve whose explicit
    // seed never reads it.
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
        .failure()
        .stderr(predicate::str::contains("failed to parse config file"));
}

#[test]
fn remembered_operator_ids_are_not_approval_candidates() {
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
        .arg("11111111-1111-4111-8111-111111111111")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "no configured operator public key belongs to this manifest set",
        ));
}

#[test]
fn malformed_registered_local_operator_id_is_reported() {
    let temp = TempDir::new().unwrap();
    let operator_key_path = temp.path().join("operator.json");
    fs::write(
        &operator_key_path,
        serde_json::to_string(&StoredQosOperatorKey {
            public_key: fixture_manifest_member_key(0),
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
                operators: vec![OperatorRecord {
                    name: "local".to_string(),
                    kind: OperatorRecordKind::Local(LocalOperatorRecord {
                        key_path: operator_key_path,
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
fn manifest_membership_controls_signer_resolution_in_mixed_registry() {
    let temp = TempDir::new().unwrap();
    let public = fixture_manifest_member_key(0);
    let operator_key_path = temp.path().join("operator.json");
    fs::write(
        &operator_key_path,
        serde_json::to_string(&StoredQosOperatorKey {
            // The signer path only reads the seed; a nil key stands in.
            public_key: QosOperatorPublicKey::default(),
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
                            encrypt_public_key: hex::encode(&public.as_bytes()[..65]),
                            sign_public_key: hex::encode(&public.as_bytes()[65..]),
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
        .arg("11111111-1111-4111-8111-111111111111")
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
            "the argument '--operator-seed <HEX_SEED>' cannot be used with '--operator-seed-path <PATH>'",
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
        .arg("5376f492-d014-4e01-a6bb-20fc97448e25")
        .arg("--dangerous-skip-interactive")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--manifest <PATH>' cannot be used with '--deploy-id <DEPLOY_ID>'",
        ));
}

#[test]
fn operator_id_and_serial_are_mutually_exclusive() {
    cargo_bin_cmd!("tvc")
        .args([
            "deploy",
            "approve",
            "--deploy-id",
            DEPLOYMENT_ID,
            "--operator-id",
            YUBIKEY_OPERATOR_ID,
            "--serial",
            "0171f8a4",
            "--dangerous-skip-interactive",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "the argument '--operator-id <OPERATOR_ID>' cannot be used with '--serial <SERIAL>'",
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
