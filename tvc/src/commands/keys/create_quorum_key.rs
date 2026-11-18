//! Create a hosted quorum key from operator encryption public keys.

use crate::{
    client::build_client,
    commands::Run,
    config::turnkey::Config,
    operator::{OperatorPublicKey, timestamp_ms},
    outcome::Outcome,
    output::StdCtx,
    pending_creation::{self, CreationClient, PreparedCreation},
    terminal::TerminalText,
};
use anyhow::{Context, Result, ensure};
use clap::{ArgAction, ArgGroup, Args as ClapArgs, builder::RangedU64ValueParser};
use serde::Serialize;
use std::{
    collections::HashSet,
    fmt::{self, Display, Formatter},
};
use tracing::instrument;
use turnkey_proofs::tvc::{MAX_QUORUM_RECIPIENTS, P256PublicKey, VerifiedQuorumKey};
use uuid::Uuid;

pub mod verify_result;

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
    // Clap also parses these args for `verify-result`, which takes no threshold.
    #[arg(
        long,
        required = true,
        env = "TVC_QUORUM_KEY_THRESHOLD",
        value_parser = threshold_parser()
    )]
    threshold: Option<u8>,

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

    /// Display the API result without verifying quorum key proofs.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_VERIFICATION")]
    dangerous_skip_verification: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumKeyCreated {
    quorum_key_id: String,
    quorum_public_key: String,
    share_ids: Vec<String>,
}

impl From<VerifiedQuorumKey> for QuorumKeyCreated {
    fn from(verified: VerifiedQuorumKey) -> Self {
        let (id, public_key, share_ids) = verified.into_parts();
        Self {
            quorum_key_id: id.to_string(),
            quorum_public_key: hex::encode(public_key.as_ref()),
            share_ids: share_ids.into_iter().map(|id| id.to_string()).collect(),
        }
    }
}

impl Display for QuorumKeyCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Quorum Key ID: {}\nQuorum Public Key: {}\nShare IDs: {}",
            TerminalText(&self.quorum_key_id).single_line(),
            self.quorum_public_key,
            self.share_ids
                .iter()
                .map(|id| TerminalText(id).single_line())
                .collect::<Vec<_>>()
                .join(", ")
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

impl Run for Args {
    type Outcome = Outcome;

    #[instrument(skip_all)]
    async fn run(self, ctx: &mut StdCtx, config: Config) -> Result<Outcome> {
        let Self {
            threshold,
            operator_encrypt_keys,
            operator_ids,
            dangerous_skip_verification,
        } = self;
        let threshold = threshold.context("missing --threshold for quorum-key creation")?;
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
            resolve_operator_encrypt_keys(&config, operator_source)?;
        let recipients = validate_operator_encrypt_keys(operator_encrypt_keys)?;

        let auth = CreationClient::new(build_client(&config).await?, dangerous_skip_verification)?;
        if let Some(configured_org_id) = configured_org_id {
            auth.ensure_configured_org(&configured_org_id)?;
        }
        let creation = PreparedCreation::quorum(auth.org_id, u32::from(threshold), recipients)?;
        pending_creation::submit(ctx, config, auth, creation, timestamp_ms()?).await
    }
}

fn parse_operator_id(value: &str) -> std::result::Result<Uuid, String> {
    Uuid::parse_str(value.trim()).map_err(|_| "must be a UUID".to_string())
}

fn resolve_operator_encrypt_keys(
    config: &Config,
    operator_source: OperatorSource,
) -> Result<(Vec<OperatorPublicKey>, Option<String>)> {
    match operator_source {
        OperatorSource::EncryptKeys(keys) => Ok((keys, None)),
        OperatorSource::OperatorIds(operator_ids) => {
            validate_operator_ids(&operator_ids)?;
            resolve_operator_ids(config, &operator_ids)
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

    let keys = operator_ids
        .iter()
        .map(|id| {
            config
                .resolve_hosted_operator_encrypt_key(id)
                .map(OperatorPublicKey::from)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((keys, Some(org.id.clone())))
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

fn validate_operator_encrypt_keys(input: Vec<OperatorPublicKey>) -> Result<Vec<P256PublicKey>> {
    let keys: Vec<_> = input.into_iter().map(P256PublicKey::from).collect();
    let mut seen = HashSet::new();
    for (index, key) in keys.iter().enumerate() {
        ensure!(
            seen.insert(key),
            "duplicate operator encryption public key at index {index}"
        );
    }
    Ok(keys)
}

fn validate_operator_count(operator_count: usize) -> Result<()> {
    ensure!(
        operator_count <= MAX_QUORUM_RECIPIENTS,
        "operator encryption public key count ({operator_count}) cannot exceed {MAX_QUORUM_RECIPIENTS}"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
    };
    use qos_p256::P256Pair;
    use serde_json::Value;
    use std::{collections::BTreeMap, path::PathBuf};

    const FIRST_OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
    const SECOND_OPERATOR_ID: &str = "22222222-2222-4222-8222-222222222222";

    fn operator_encrypt_key() -> String {
        let public_key = P256Pair::generate().unwrap().public_key().to_bytes();
        hex::encode(&public_key[..65])
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
            orgs: BTreeMap::from([(
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
        let parsed = validate_operator_encrypt_keys(keys).unwrap();

        assert_eq!(
            parsed.iter().map(ToString::to_string).collect::<Vec<_>>(),
            vec![first, second]
        );
    }

    #[test]
    fn rejects_duplicate_operator_encrypt_keys_after_normalization() {
        let key = operator_encrypt_key();
        let keys = vec![key.parse().unwrap(), key.to_uppercase().parse().unwrap()];
        let error = validate_operator_encrypt_keys(keys)
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
    fn accepts_255_quorum_recipients() {
        assert!(validate_operator_count(255).is_ok());
    }

    #[test]
    fn rejects_256_quorum_recipients() {
        assert_eq!(
            validate_operator_count(256).unwrap_err().to_string(),
            "operator encryption public key count (256) cannot exceed 255"
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
            validate_operator_encrypt_keys(keys)
                .unwrap_err()
                .to_string(),
            "duplicate operator encryption public key at index 1"
        );
    }

    #[test]
    fn formats_accepted_quorum_output() {
        let output = QuorumKeyCreated {
            quorum_key_id: "quorum-key-id".to_string(),
            quorum_public_key: "quorum-public-key".to_string(),
            share_ids: vec!["share-1".to_string(), "share-2".to_string()],
        };
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
