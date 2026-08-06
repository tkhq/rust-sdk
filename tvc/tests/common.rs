//! Fixtures shared across the tvc integration-test binaries.

// Each test binary that declares `mod common;` compiles its own copy and
// uses a subset of these helpers, so every binary's build has genuinely
// unused items here; without this, each `cargo test` target warns.
#![allow(dead_code)]

use indexmap::IndexMap;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use tvc::config::turnkey::{
    Config, KeyCurve, OperatorKind, OperatorRecord, OrgConfig, QosOperatorPublicKey, StoredApiKey,
    StoredQosOperatorKey,
};

/// Dead port: connection attempts fail immediately, so commands stop at their
/// first network step without hanging.
pub const LOCAL_API_BASE_URL: &str = "http://127.0.0.1:1";

fn org_dir(home: &Path, alias: &str) -> PathBuf {
    home.join(".config/turnkey/orgs").join(alias)
}

/// Write a v1 `tvc.config.toml` under `home` with one profile per
/// `(alias, org_id)` pair, using the legacy alias-keyed key-file layout and a
/// dead-port API base URL. The legacy layout is deliberate: these fixtures
/// seed pre-existing user state (which interactive login migrates to the
/// id-keyed layout); the current layout is exercised through the real CLI's
/// new-profile flow. Profiles are written in slice order, which is
/// meaningful: config loading marks the first profile of a duplicated
/// organization as its default when none of them carries the marker. Aliases
/// listed in `default_aliases` are written with `default_alias = true`.
pub fn write_profiles_config(
    home: &Path,
    profiles: &[(&str, &str)],
    active_org: Option<&str>,
    default_aliases: &[&str],
) {
    let turnkey_dir = home.join(".config/turnkey");
    fs::create_dir_all(&turnkey_dir).unwrap();

    let orgs: IndexMap<_, _> = profiles
        .iter()
        .map(|(alias, org_id)| {
            let dir = org_dir(home, alias);
            (
                alias.to_string(),
                OrgConfig {
                    id: org_id.parse().expect("test org ids must be UUIDs"),
                    api_key_path: dir.join("api_key.json"),
                    api_base_url: LOCAL_API_BASE_URL.to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators: vec![OperatorRecord::local(dir.join("operator.json"))],
                    default_alias: default_aliases.contains(alias),
                    extra: toml::Table::new(),
                },
            )
        })
        .collect();

    let config = Config {
        active_org: active_org.map(String::from),
        orgs,
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
