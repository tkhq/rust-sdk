//! Fixtures shared across the tvc integration-test binaries. Each binary
//! compiles its own copy and uses a subset, so unused items are expected.
#![allow(dead_code)]

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::turnkey::{
    Config, HostedOperatorRecord, KeyCurve, OperatorKind, OperatorRecord, OperatorRecordKind,
    OrgConfig, QosOperatorPublicKey, StoredApiKey, StoredQosOperatorKey, YubiKeyOperatorRecord,
    YubiKeyRegistryEntry, YubiKeySerial,
};

/// Dead port: connection attempts fail immediately, so commands stop at their
/// first network step without hanging.
pub const LOCAL_API_BASE_URL: &str = "http://127.0.0.1:1";

fn org_dir(home: &Path, alias: &str) -> PathBuf {
    home.join(".config/turnkey/orgs").join(alias)
}

/// Write a v1 `tvc.config.toml` under `home` with one profile per
/// `(alias, org_id)` pair, using the default alias-keyed key-file layout and
/// a dead-port API base URL.
pub fn write_profiles_config(home: &Path, profiles: &[(&str, &str)], active_org: Option<&str>) {
    let turnkey_dir = home.join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let orgs: HashMap<_, _> = profiles
        .iter()
        .map(|(alias, org_id)| {
            let dir = org_dir(home, alias);
            (
                alias.to_string(),
                OrgConfig {
                    id: org_id.to_string(),
                    api_key_path: dir.join("api_key.json"),
                    api_base_url: LOCAL_API_BASE_URL.to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators: vec![OperatorRecord::local(dir.join("operator.json"))],
                    extra: toml::Table::new(),
                },
            )
        })
        .collect();

    let config = Config {
        active_org: active_org.map(String::from),
        orgs,
        yubikeys: Default::default(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// Write a v1 `tvc.config.toml` under `home` whose sole, active organization
/// defaults to the hosted backend and registers exactly one operator, hosted
/// — the fixture for commands that need local key material a hosted-only org
/// does not have. The record carries a real generated composite key split
/// into its two points, the way `operator create` stores it.
pub fn write_hosted_only_config(home: &Path, alias: &str, org_id: &str) {
    let turnkey_dir = home.join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let composite = hex::encode(
        qos_p256::P256Pair::generate()
            .unwrap()
            .public_key()
            .to_bytes(),
    );
    let (encrypt_public_key, sign_public_key) = composite.split_at(composite.len() / 2);

    let config = Config {
        active_org: Some(alias.to_string()),
        orgs: HashMap::from([(
            alias.to_string(),
            OrgConfig {
                id: org_id.to_string(),
                api_key_path: turnkey_dir.join(format!("orgs/{alias}/api_key.json")),
                api_base_url: LOCAL_API_BASE_URL.to_string(),
                default_operator_kind: OperatorKind::Hosted,
                operators: vec![OperatorRecord {
                    name: "hosted-op".to_string(),
                    kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                        operator_id: "11111111-1111-4111-8111-111111111111".parse().unwrap(),
                        wallet_id: "22222222-2222-4222-8222-222222222222".parse().unwrap(),
                        path: "m/5527107'/0'/0'".to_string(),
                        encrypt_public_key: encrypt_public_key.to_string(),
                        sign_public_key: sign_public_key.to_string(),
                        extra: toml::Table::new(),
                    }),
                }],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: Default::default(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// Write a v1 `tvc.config.toml` under `home` whose sole, active organization
/// defaults to the yubikey backend: one serial-only operator record plus the
/// top-level registry entry caching a real generated composite key. Flows
/// that read the cache need no device; device-touching flows fail at their
/// first PC/SC step.
pub fn write_yubikey_only_config(home: &Path, alias: &str, org_id: &str) {
    let turnkey_dir = home.join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let serial = YubiKeySerial::from(0x01c9_5c1f);
    let public_key = QosOperatorPublicKey::try_from(
        qos_p256::P256Pair::generate()
            .unwrap()
            .public_key()
            .to_bytes()
            .as_slice(),
    )
    .unwrap();

    let config = Config {
        active_org: Some(alias.to_string()),
        orgs: HashMap::from([(
            alias.to_string(),
            OrgConfig {
                id: org_id.to_string(),
                api_key_path: turnkey_dir.join(format!("orgs/{alias}/api_key.json")),
                api_base_url: LOCAL_API_BASE_URL.to_string(),
                default_operator_kind: OperatorKind::Yubikey,
                operators: vec![OperatorRecord {
                    name: "yubikey-op".to_string(),
                    kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                        serial,
                        extra: toml::Table::new(),
                    }),
                }],
                extra: toml::Table::new(),
            },
        )]),
        yubikeys: vec![YubiKeyRegistryEntry {
            serial,
            public_key,
            extra: toml::Table::new(),
        }]
        .try_into()
        .unwrap(),
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// Write a v1 `tvc.config.toml` under `home` with one yubikey-default
/// organization per `(alias, org_id)` pair, every org referencing the SAME
/// registered serial — the fixture for shared-registry-entry behavior. The
/// first alias is active.
pub fn write_yubikey_shared_config(home: &Path, profiles: &[(&str, &str)]) {
    let turnkey_dir = home.join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let serial = YubiKeySerial::from(0x01c9_5c1f);
    let public_key = QosOperatorPublicKey::try_from(
        qos_p256::P256Pair::generate()
            .unwrap()
            .public_key()
            .to_bytes()
            .as_slice(),
    )
    .unwrap();

    let orgs: HashMap<_, _> = profiles
        .iter()
        .map(|(alias, org_id)| {
            (
                alias.to_string(),
                OrgConfig {
                    id: org_id.to_string(),
                    api_key_path: turnkey_dir.join(format!("orgs/{alias}/api_key.json")),
                    api_base_url: LOCAL_API_BASE_URL.to_string(),
                    default_operator_kind: OperatorKind::Yubikey,
                    operators: vec![OperatorRecord {
                        name: "yubikey-op".to_string(),
                        kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                            serial,
                            extra: toml::Table::new(),
                        }),
                    }],
                    extra: toml::Table::new(),
                },
            )
        })
        .collect();

    let config = Config {
        active_org: profiles.first().map(|(alias, _)| alias.to_string()),
        orgs,
        yubikeys: vec![YubiKeyRegistryEntry {
            serial,
            public_key,
            extra: toml::Table::new(),
        }],
        last_created_app_id: HashMap::new(),
        last_operator_ids: HashMap::new(),
        extra: toml::Table::new(),
    };

    fs::write(
        turnkey_dir.join("tvc.config.toml"),
        format!("version = 1\n{}", toml::to_string_pretty(&config).unwrap()),
    )
    .unwrap();
}

/// Create the default-layout key files for `alias`: a valid generated
/// `StoredApiKey` (login loads it before its first network step) and a real
/// generated operator key. Returns the operator public key so tests can
/// assert on rendered output.
pub fn write_profile_key_files(home: &Path, alias: &str) -> QosOperatorPublicKey {
    let dir = org_dir(home, alias);
    fs::create_dir_all(&dir).unwrap();

    let stamper = TurnkeyP256ApiKey::generate();
    let api_key = StoredApiKey {
        public_key: hex::encode(stamper.compressed_public_key()),
        private_key: hex::encode(stamper.private_key()),
        curve: KeyCurve::P256,
    };

    fs::write(
        dir.join("api_key.json"),
        serde_json::to_string_pretty(&api_key).unwrap(),
    )
    .unwrap();

    let pair = qos_p256::P256Pair::generate().unwrap();
    let operator_key = StoredQosOperatorKey {
        public_key: QosOperatorPublicKey::try_from(pair.public_key().to_bytes().as_slice())
            .unwrap(),
        private_key: hex::encode(pair.to_master_seed()),
    };
    fs::write(
        dir.join("operator.json"),
        serde_json::to_string_pretty(&operator_key).unwrap(),
    )
    .unwrap();

    operator_key.public_key
}
