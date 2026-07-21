//! Create a hosted quorum key from operator encryption public keys.

use crate::{client::build_client, outcome::Outcome, output::StdCtx};
use anyhow::{Context, Result, anyhow, ensure};
use clap::Args as ClapArgs;
use p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use serde::Serialize;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{CreateTvcQuorumKeyIntent, CreateTvcQuorumKeyResult};

// `qos_crypto::shamir::shares_generate` supports at most 255 shares. Hosted
// quorum generation keeps the operator key count strictly below that bound.
const MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE: usize = 255;

/// Create a hosted quorum key encrypted to hosted operator keys.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Number of operator shares required to reconstruct the quorum key.
    #[arg(long, env = "TVC_QUORUM_KEY_THRESHOLD")]
    pub threshold: u8,

    /// Comma-separated, uncompressed P-256 operator encryption public keys.
    #[arg(long, value_name = "HEX,HEX,...", env = "TVC_OPERATOR_ENCRYPT_KEYS")]
    pub operator_encrypt_keys: String,
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

/// Run the hosted quorum-key creation command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let operator_encrypt_keys = parse_operator_encrypt_keys(&args.operator_encrypt_keys)?;
    validate_operator_count(operator_encrypt_keys.len())?;
    validate_threshold(args.threshold, operator_encrypt_keys.len())?;

    let intent = build_create_tvc_quorum_key_intent(args.threshold, operator_encrypt_keys);
    let expected_share_count = intent.operator_encrypt_keys.len();
    let auth = build_client().await?;
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

fn parse_operator_encrypt_keys(input: &str) -> Result<Vec<String>> {
    let mut seen = HashSet::new();
    input
        .split(',')
        .enumerate()
        .map(|(index, key)| {
            let key = key.trim();
            ensure!(
                !key.is_empty(),
                "operator encryption public key at index {index} is empty"
            );

            let bytes = hex::decode(key).with_context(|| {
                format!(
                    "operator encryption public key at index {index} must be bare hex encoded"
                )
            })?;
            ensure!(
                bytes.len() == 65 && bytes.first() == Some(&0x04),
                "operator encryption public key at index {index} must be a 65-byte uncompressed P-256 public key"
            );

            let public_key = PublicKey::from_sec1_bytes(&bytes).with_context(|| {
                format!("operator encryption public key at index {index} is not a valid P-256 point")
            })?;
            let normalized = hex::encode(public_key.to_encoded_point(false).as_bytes());
            ensure!(
                seen.insert(normalized.clone()),
                "duplicate operator encryption public key at index {index}"
            );

            Ok(normalized)
        })
        .collect()
}

fn validate_operator_count(operator_count: usize) -> Result<()> {
    ensure!(
        operator_count < MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE,
        "operator encryption public key count ({operator_count}) must be less than {MAX_OPERATOR_ENCRYPT_KEY_COUNT_EXCLUSIVE}"
    );
    Ok(())
}

fn validate_threshold(threshold: u8, operator_count: usize) -> Result<()> {
    ensure!(threshold >= 2, "threshold must be at least 2");
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
    use qos_p256::P256Pair;
    use serde_json::Value;

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

    #[test]
    fn parses_trims_and_normalizes_operator_encrypt_keys() {
        let first = operator_encrypt_key();
        let second = operator_encrypt_key();
        let parsed = parse_operator_encrypt_keys(&format!(
            "  {} , {}  ",
            first.to_uppercase(),
            second.to_uppercase()
        ))
        .unwrap();

        assert_eq!(parsed, vec![first, second]);
    }

    #[test]
    fn rejects_empty_operator_encrypt_key() {
        let key = operator_encrypt_key();
        let error = parse_operator_encrypt_keys(&format!("{key},,"))
            .unwrap_err()
            .to_string();

        assert_eq!(error, "operator encryption public key at index 1 is empty");
    }

    #[test]
    fn rejects_non_hex_operator_encrypt_key() {
        let error = parse_operator_encrypt_keys("not-hex")
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "operator encryption public key at index 0 must be bare hex encoded"
        );
    }

    #[test]
    fn rejects_wrong_length_operator_encrypt_key() {
        let error = parse_operator_encrypt_keys("04abcd")
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "operator encryption public key at index 0 must be a 65-byte uncompressed P-256 public key"
        );
    }

    #[test]
    fn rejects_compressed_operator_encrypt_key() {
        let uncompressed = operator_encrypt_key();
        let public_key = PublicKey::from_sec1_bytes(&hex::decode(uncompressed).unwrap()).unwrap();
        let compressed = hex::encode(public_key.to_encoded_point(true).as_bytes());
        let error = parse_operator_encrypt_keys(&compressed)
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "operator encryption public key at index 0 must be a 65-byte uncompressed P-256 public key"
        );
    }

    #[test]
    fn rejects_invalid_curve_point() {
        let invalid = format!("04{}", "00".repeat(64));
        let error = parse_operator_encrypt_keys(&invalid)
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "operator encryption public key at index 0 is not a valid P-256 point"
        );
    }

    #[test]
    fn rejects_duplicate_operator_encrypt_keys_after_normalization() {
        let key = operator_encrypt_key();
        let error = parse_operator_encrypt_keys(&format!("{key},{}", key.to_uppercase()))
            .unwrap_err()
            .to_string();

        assert_eq!(error, "duplicate operator encryption public key at index 1");
    }

    #[test]
    fn validates_threshold_boundaries() {
        assert_eq!(
            validate_threshold(1, 2).unwrap_err().to_string(),
            "threshold must be at least 2"
        );
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
