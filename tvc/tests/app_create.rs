//! Integration tests for `tvc app create`'s operator-reuse decision.

use assert_cmd::cargo::cargo_bin_cmd;
use indexmap::IndexMap;
use predicates::prelude::*;
use qos_p256::P256Pair;
use std::collections::HashMap;
use std::path::Path;
use tempfile::TempDir;
use tvc::config::app::KNOWN_QUORUM_KEY;
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
};

const NON_INTERACTIVE_ENV: &str = "TVC_NON_INTERACTIVE";

/// Write a config whose active org has two hosted identities proven to use
/// the same key plus one saved ID with no persisted key association. Returns
/// the shared composite public key.
fn write_two_candidate_config(home: &Path) -> String {
    let turnkey_dir = home.join(".config/turnkey");
    std::fs::create_dir_all(&turnkey_dir).unwrap();

    let composite = hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());
    let (encrypt_public_key, sign_public_key) = composite.split_at(composite.len() / 2);

    let config = Config {
        active_org: Some("hosted-org".to_string()),
        orgs: IndexMap::from([(
            "hosted-org".to_string(),
            OrgConfig {
                id: "44444444-4444-4444-8444-444444444444".parse().unwrap(),
                api_key_path: turnkey_dir.join("orgs/hosted-org/api_key.json"),
                api_base_url: "http://127.0.0.1:1".to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![
                    OperatorRecord {
                        name: "hosted-op".to_string(),
                        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                            operator_id: "11111111-1111-4111-8111-111111111111".parse().unwrap(),
                            wallet_id: "22222222-2222-4222-8222-222222222222".parse().unwrap(),
                            path: "m/5527107'/0'/0'".to_string(),
                            encrypt_public_key: encrypt_public_key.to_string(),
                            sign_public_key: sign_public_key.to_string(),
                            extra: toml::Table::new(),
                        }),
                    },
                    OperatorRecord {
                        name: "hosted-op-2".to_string(),
                        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                            operator_id: "33333333-3333-4333-8333-333333333333".parse().unwrap(),
                            wallet_id: "55555555-5555-4555-8555-555555555555".parse().unwrap(),
                            path: "m/5527107'/0'/0'".to_string(),
                            encrypt_public_key: encrypt_public_key.to_string(),
                            sign_public_key: sign_public_key.to_string(),
                            extra: toml::Table::new(),
                        }),
                    },
                ],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: Default::default(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::from([(
            "hosted-org".to_string(),
            vec!["66666666-6666-4666-8666-666666666666".to_string()],
        )]),
        extra: toml::Table::new(),
    };
    std::fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();

    composite
}

/// Picking among multiple reuse candidates needs a prompt; with piped
/// (non-TTY) stdin and no --non-interactive flag, the fence must bail with
/// the remediation message instead of reaching the selection prompt.
#[test]
fn piped_stdin_with_multiple_reuse_candidates_errors() {
    let temp = TempDir::new().unwrap();
    let composite = write_two_candidate_config(temp.path());

    // A complete app config: nothing to fill, so the run goes straight to
    // the reuse decision.
    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "operator-1",
            "publicKey": "{composite}"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .args([
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "multiple operator IDs use the requested manifest operator key",
        ));
}

/// The escape hatch the fence's error names: an explicit
/// `existingOperatorIds` picks the operator in the config, so the same
/// piped-stdin run sails past the reuse decision without a prompt. The
/// command still fails later — creating the app needs credentials this
/// fixture doesn't have — but only after the decision.
#[test]
fn piped_stdin_multiple_candidates_with_explicit_ids_proceeds() {
    let temp = TempDir::new().unwrap();
    write_two_candidate_config(temp.path());

    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "existingOperatorIds": ["33333333-3333-4333-8333-333333333333"]
    }}
}}"#
        ),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .args([
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Creating app 'test-app'"))
        .stderr(
            predicate::str::contains(
                "multiple operator IDs use the requested manifest operator key",
            )
            .not(),
        );
}

/// The other escape hatch: `--no-operator-reuse` opts out of the decision
/// entirely, so the piped-stdin run mints new operators from the config and
/// never needs a prompt. Fails later on missing credentials, as above.
#[test]
fn piped_stdin_multiple_candidates_with_no_reuse_flag_proceeds() {
    let temp = TempDir::new().unwrap();
    let composite = write_two_candidate_config(temp.path());

    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "operator-1",
            "publicKey": "{composite}"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .args([
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
            "--no-operator-reuse",
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Creating app 'test-app'"))
        .stderr(
            predicate::str::contains(
                "multiple operator IDs use the requested manifest operator key",
            )
            .not(),
        );
}

/// Unrelated registered/saved identities must not override an explicit
/// newOperators key. This is the local/YubiKey regression: those backends
/// supply a public key in app.json but do not yet have a reusable server ID.
#[test]
fn piped_stdin_preserves_an_unmatched_new_operator_key() {
    let temp = TempDir::new().unwrap();
    write_two_candidate_config(temp.path());
    let requested_key = "07".repeat(130);

    let app_config_path = temp.path().join("app.json");
    std::fs::write(
        &app_config_path,
        format!(
            r#"{{
    "name": "test-app",
    "quorumPublicKey": "{KNOWN_QUORUM_KEY}",
    "manifestSetParams": {{
        "name": "manifest-set",
        "threshold": 1,
        "newOperators": [{{
            "name": "self-custody-operator",
            "publicKey": "{requested_key}"
        }}]
    }}
}}"#
        ),
    )
    .unwrap();

    cargo_bin_cmd!("tvc")
        .env("HOME", temp.path())
        .env_remove(NON_INTERACTIVE_ENV)
        .args([
            "app",
            "create",
            "--config-file",
            app_config_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("Creating app 'test-app'"))
        .stdout(predicate::str::contains("Reusing operator").not())
        .stderr(
            predicate::str::contains(
                "multiple operator IDs use the requested manifest operator key",
            )
            .not(),
        );
}
