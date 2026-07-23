//! Create a hosted quorum key from operator encryption public keys.

use crate::{
    client::build_client,
    config::turnkey::Config,
    operator::{OperatorPublicKey, ensure_authenticated_org, resolve_hosted_operator_encrypt_key},
    outcome::Outcome,
    output::StdCtx,
};
use anyhow::{Context, Result, anyhow, ensure};
use clap::{ArgAction, ArgGroup, Args as ClapArgs, builder::RangedU64ValueParser};
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
    time::{SystemTime, UNIX_EPOCH},
};
use turnkey_client::generated::{CreateTvcQuorumKeyIntent, CreateTvcQuorumKeyResult};
use uuid::Uuid;

// `qos_crypto::shamir::shares_generate` supports at most 255 shares. Hosted
// quorum generation keeps the operator key count strictly below that bound.
const MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE: usize = 255;

fn threshold_parser() -> RangedU64ValueParser<u8> {
    RangedU64ValueParser::new().range(2..)
}

/// Create a hosted quorum key encrypted to hosted operator keys.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
#[command(group(
    ArgGroup::new("operator_source")
        .args(["operator_encrypt_keys", "operator_ids"])
        .required(true)
        .multiple(false)
))]
pub struct Args {
    /// Number of operator shares required to reconstruct the quorum key.
    #[arg(
        long,
        env = "TVC_QUORUM_KEY_THRESHOLD",
        value_parser = threshold_parser()
    )]
    threshold: u8,

    /// Comma-separated, uncompressed P-256 operator encryption public keys.
    #[arg(
        long,
        value_name = "HEX",
        value_delimiter = ',',
        action = ArgAction::Set,
        env = "TVC_OPERATOR_ENCRYPT_KEYS"
    )]
    operator_encrypt_keys: Vec<OperatorPublicKey>,

    /// Comma-separated hosted operator UUIDs from the active organization.
    #[arg(
        long,
        value_name = "UUID",
        value_delimiter = ',',
        value_parser = parse_operator_id,
        action = ArgAction::Set,
        env = "TVC_OPERATOR_IDS"
    )]
    operator_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumKeyCreated {
    quorum_key_id: String,
    quorum_public_key: String,
    share_ids: Vec<String>,
}

impl Display for QuorumKeyCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Quorum Key ID: {}\nQuorum Public Key: {}\nShare IDs: {}",
            self.quorum_key_id,
            self.quorum_public_key,
            self.share_ids.join(", ")
        )
    }
}

enum OperatorSource {
    EncryptKeys(Vec<OperatorPublicKey>),
    OperatorIds(Vec<Uuid>),
}

impl OperatorSource {
    fn len(&self) -> usize {
        match self {
            Self::EncryptKeys(keys) => keys.len(),
            Self::OperatorIds(ids) => ids.len(),
        }
    }
}

/// Run the hosted quorum-key creation command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let Args {
        threshold,
        operator_encrypt_keys,
        operator_ids,
    } = args;
    // The required, mutually exclusive Clap group guarantees exactly one
    // source is non-empty.
    let operator_source = if operator_ids.is_empty() {
        OperatorSource::EncryptKeys(operator_encrypt_keys)
    } else {
        OperatorSource::OperatorIds(operator_ids)
    };
    validate_operator_count(operator_source.len())?;
    validate_threshold(threshold, operator_source.len())?;

    let (operator_encrypt_keys, configured_org_id) =
        resolve_operator_encrypt_keys(operator_source).await?;
    let operator_encrypt_keys = normalize_operator_encrypt_keys(operator_encrypt_keys)?;

    let intent = build_create_tvc_quorum_key_intent(threshold, operator_encrypt_keys);
    let expected_share_count = intent.operator_encrypt_keys.len();
    let auth = build_client().await?;
    if let Some(configured_org_id) = configured_org_id {
        ensure_authenticated_org(&auth.org_id, &configured_org_id)?;
    }
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .create_tvc_quorum_key(auth.org_id, timestamp_ms, intent)
        .await
        .map_err(|error| anyhow!("failed to create hosted TVC quorum key: {error}"))?;
    let output = validate_result(result.result, expected_share_count)?;

    Ok(Outcome::KeysCreateQuorumKey(output))
}

fn parse_operator_id(value: &str) -> std::result::Result<Uuid, String> {
    Uuid::parse_str(value.trim()).map_err(|_| "must be a UUID".to_string())
}

async fn resolve_operator_encrypt_keys(
    operator_source: OperatorSource,
) -> Result<(Vec<OperatorPublicKey>, Option<String>)> {
    match operator_source {
        OperatorSource::EncryptKeys(keys) => Ok((keys, None)),
        OperatorSource::OperatorIds(operator_ids) => {
            validate_operator_ids(&operator_ids)?;
            let config = Config::load().await?;
            resolve_operator_ids(&config, &operator_ids)
        }
    }
}

fn resolve_operator_ids(
    config: &Config,
    operator_ids: &[Uuid],
) -> Result<(Vec<OperatorPublicKey>, Option<String>)> {
    let (_, org) = config
        .active_org_config()
        .context("No active organization. Run `tvc login` first.")?;
    let configured_org_id = org.id.clone();
    let mut keys = Vec::with_capacity(operator_ids.len());

    for operator_id in operator_ids {
        keys.push(resolve_hosted_operator_encrypt_key(config, operator_id)?);
    }

    Ok((keys, Some(configured_org_id)))
}

fn validate_operator_ids(operator_ids: &[Uuid]) -> Result<()> {
    let mut seen = HashSet::new();
    for (index, operator_id) in operator_ids.iter().enumerate() {
        ensure!(
            seen.insert(operator_id),
            "duplicate operator ID at index {index}: {operator_id}"
        );
    }
    Ok(())
}

fn normalize_operator_encrypt_keys(input: Vec<OperatorPublicKey>) -> Result<Vec<String>> {
    let normalized: Vec<_> = input.into_iter().map(|key| key.to_string()).collect();
    let mut seen = HashSet::new();
    for (index, key) in normalized.iter().enumerate() {
        ensure!(
            seen.insert(key),
            "duplicate operator encryption public key at index {index}"
        );
    }
    Ok(normalized)
}

fn validate_operator_count(operator_count: usize) -> Result<()> {
    ensure!(
        operator_count < MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE,
        "operator encryption public key count ({operator_count}) must be less than {MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE}"
    );
    Ok(())
}

fn validate_threshold(threshold: u8, operator_count: usize) -> Result<()> {
    ensure!(
        threshold as usize <= operator_count,
        "threshold ({threshold}) cannot exceed operator encryption public key count ({operator_count})"
    );
    Ok(())
}

fn build_create_tvc_quorum_key_intent(
    threshold: u8,
    operator_encrypt_keys: Vec<String>,
) -> CreateTvcQuorumKeyIntent {
    CreateTvcQuorumKeyIntent {
        threshold: u32::from(threshold),
        operator_encrypt_keys,
    }
}

fn validate_result(
    result: CreateTvcQuorumKeyResult,
    expected_share_count: usize,
) -> Result<QuorumKeyCreated> {
    ensure!(
        !result.quorum_key_id.trim().is_empty(),
        "create TVC quorum key response contained an empty quorum key ID"
    );
    ensure!(
        !result.quorum_public_key.trim().is_empty(),
        "create TVC quorum key response contained an empty quorum public key"
    );
    ensure!(
        result.share_ids.len() == expected_share_count,
        "create TVC quorum key response returned {} share IDs for {expected_share_count} operator encryption public keys",
        result.share_ids.len()
    );
    ensure!(
        result.share_ids.iter().all(|id| !id.trim().is_empty()),
        "create TVC quorum key response contained an empty share ID"
    );

    Ok(QuorumKeyCreated {
        quorum_key_id: result.quorum_key_id,
        quorum_public_key: result.quorum_public_key,
        share_ids: result.share_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
    };
    use qos_p256::P256Pair;
    use serde_json::Value;
    use std::{collections::HashMap, path::PathBuf};

    const FIRST_OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
    const SECOND_OPERATOR_ID: &str = "22222222-2222-4222-8222-222222222222";

    fn operator_encrypt_key() -> String {
        let public_key = P256Pair::generate().unwrap().public_key().to_bytes();
        hex::encode(&public_key[..65])
    }

    fn result_with_share_ids(share_ids: &[&str]) -> CreateTvcQuorumKeyResult {
        CreateTvcQuorumKeyResult {
            quorum_key_id: "quorum-key-id".to_string(),
            quorum_public_key: "quorum-public-key".to_string(),
            share_ids: share_ids.iter().map(|id| (*id).to_string()).collect(),
        }
    }

    fn hosted_operator(name: &str, operator_id: &str, wallet_id: &str) -> (OperatorRecord, String) {
        let public = P256Pair::generate().unwrap().public_key().to_bytes();
        let encrypt_public_key = hex::encode(&public[..65]);
        (
            OperatorRecord {
                name: name.to_string(),
                kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                    operator_id: Uuid::parse_str(operator_id).unwrap(),
                    wallet_id: Uuid::parse_str(wallet_id).unwrap(),
                    path: "m/5527107'/0'/0'".to_string(),
                    encrypt_public_key: encrypt_public_key.clone(),
                    sign_public_key: hex::encode(&public[65..]),
                    extra: toml::Table::new(),
                }),
            },
            encrypt_public_key,
        )
    }

    fn config_with_operators(operators: Vec<OperatorRecord>) -> Config {
        Config {
            active_org: Some("active".to_string()),
            orgs: HashMap::from([(
                "active".to_string(),
                OrgConfig {
                    id: "org-id".to_string(),
                    api_key_path: PathBuf::from("api-key.json"),
                    api_base_url: "https://api.turnkey.com".to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators,
                    extra: toml::Table::new(),
                },
            )]),
            ..Config::default()
        }
    }

    #[test]
    fn normalizes_operator_encrypt_keys() {
        let first = operator_encrypt_key();
        let second = operator_encrypt_key();
        let keys = vec![
            format!("  {}  ", first.to_uppercase()).parse().unwrap(),
            second.to_uppercase().parse().unwrap(),
        ];
        let parsed = normalize_operator_encrypt_keys(keys).unwrap();

        assert_eq!(parsed, vec![first, second]);
    }

    #[test]
    fn rejects_duplicate_operator_encrypt_keys_after_normalization() {
        let key = operator_encrypt_key();
        let keys = vec![key.parse().unwrap(), key.to_uppercase().parse().unwrap()];
        let error = normalize_operator_encrypt_keys(keys)
            .unwrap_err()
            .to_string();

        assert_eq!(error, "duplicate operator encryption public key at index 1");
    }

    #[test]
    fn rejects_duplicate_operator_ids() {
        let operator_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
        let error = validate_operator_ids(&[operator_id, operator_id])
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "duplicate operator ID at index 1: 11111111-1111-4111-8111-111111111111"
        );
    }

    #[test]
    fn validates_threshold_boundaries() {
        assert_eq!(
            validate_threshold(3, 2).unwrap_err().to_string(),
            "threshold (3) cannot exceed operator encryption public key count (2)"
        );
        assert!(validate_threshold(2, 2).is_ok());
    }

    #[test]
    fn validates_operator_count_boundaries() {
        assert!(validate_operator_count(254).is_ok());
        assert_eq!(
            validate_operator_count(255).unwrap_err().to_string(),
            "operator encryption public key count (255) must be less than 255"
        );
    }

    #[test]
    fn builds_exact_create_intent() {
        let operator_encrypt_keys = vec![operator_encrypt_key(), operator_encrypt_key()];
        let intent = build_create_tvc_quorum_key_intent(2, operator_encrypt_keys.clone());

        assert_eq!(
            intent,
            CreateTvcQuorumKeyIntent {
                threshold: 2,
                operator_encrypt_keys,
            }
        );
    }

    #[test]
    fn operator_ids_reject_duplicate_resolved_keys() {
        let (first, shared_key) = hosted_operator(
            "first",
            FIRST_OPERATOR_ID,
            "33333333-3333-4333-8333-333333333333",
        );
        let (mut second_record, _) = hosted_operator(
            "second",
            SECOND_OPERATOR_ID,
            "44444444-4444-4444-8444-444444444444",
        );
        let OperatorRecordKind::Hosted(second) = &mut second_record.kind else {
            panic!("expected hosted operator")
        };
        second.encrypt_public_key = shared_key;
        let config = config_with_operators(vec![first, second_record]);
        let operator_ids = [
            Uuid::parse_str(FIRST_OPERATOR_ID).unwrap(),
            Uuid::parse_str(SECOND_OPERATOR_ID).unwrap(),
        ];

        let (keys, _) = resolve_operator_ids(&config, &operator_ids).unwrap();
        assert_eq!(
            normalize_operator_encrypt_keys(keys)
                .unwrap_err()
                .to_string(),
            "duplicate operator encryption public key at index 1"
        );
    }

    #[test]
    fn validates_and_formats_result() {
        let output = validate_result(result_with_share_ids(&["share-1", "share-2"]), 2).unwrap();

        assert_eq!(
            output,
            QuorumKeyCreated {
                quorum_key_id: "quorum-key-id".to_string(),
                quorum_public_key: "quorum-public-key".to_string(),
                share_ids: vec!["share-1".to_string(), "share-2".to_string()],
            }
        );
        assert_eq!(
            output.to_string(),
            "Quorum Key ID: quorum-key-id\nQuorum Public Key: quorum-public-key\nShare IDs: share-1, share-2"
        );

        // The payload serializes without a `reason`; the reason is the
        // `Outcome`'s responsibility.
        let json: Value = serde_json::to_value(&output).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "quorumKeyId": "quorum-key-id",
                "quorumPublicKey": "quorum-public-key",
                "shareIds": ["share-1", "share-2"],
            })
        );
    }
}
