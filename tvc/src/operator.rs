//! Hosted TVC operator creation.

use crate::client::AuthenticatedClient;
use crate::config::turnkey::{HostedOperatorRecord, OperatorRecord, OperatorRecordKind};
use anyhow::{Context, Result, anyhow, bail, ensure};
use p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::TurnkeyClientError;
use turnkey_client::generated::{CreateTvcOperatorIntent, CreateTvcOperatorResult};
use uuid::Uuid;

/// Default base derivation path for hosted TVC operator accounts.
///
/// `5527107` is `0x545643` (the ASCII bytes for `TVC`) and reserves a
/// TVC-specific hardened BIP32 namespace. The next component is the path
/// version (`0`) and the final component is the operator index (`0`). The
/// Turnkey signer appends `/0` for the encryption account and `/1` for the
/// signing account. Callers creating more than one operator in the same wallet
/// must currently provide a different base path themselves.
pub const DEFAULT_HOSTED_OPERATOR_BASE_PATH: &str = "m/5527107'/0'/0'";

/// Inputs for creating one hosted TVC operator.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct HostedOperatorSpec {
    name: String,
    wallet: HostedOperatorWallet,
    path: String,
}

/// Valid wallet selections for hosted operator creation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HostedOperatorWallet {
    New(String),
    Existing(Uuid),
}

impl HostedOperatorSpec {
    pub(crate) fn new(name: String, wallet: HostedOperatorWallet, path: String) -> Self {
        Self { name, wallet, path }
    }
}

/// Create one Turnkey-hosted TVC operator and return a registry-ready record.
pub(crate) async fn create_hosted_operator(
    auth: &AuthenticatedClient,
    spec: HostedOperatorSpec,
) -> Result<OperatorRecord> {
    let HostedOperatorSpec { name, wallet, path } = spec;
    let (wallet_name, wallet_id) = match wallet {
        HostedOperatorWallet::New(name) => (Some(name), None),
        HostedOperatorWallet::Existing(id) => (None, Some(id.to_string())),
    };

    let intent = CreateTvcOperatorIntent {
        wallet_name,
        wallet_id,
        path: path.clone(),
        operator_name: name.clone(),
    };
    let result = auth
        .client
        .create_tvc_operator(auth.org_id.clone(), timestamp_ms()?, intent)
        .await
        .map_err(|error| hosted_activity_error("create hosted TVC operator", error))?;

    operator_record_from_result(name, path, result.result)
}

fn operator_record_from_result(
    name: String,
    path: String,
    result: CreateTvcOperatorResult,
) -> Result<OperatorRecord> {
    let CreateTvcOperatorResult {
        wallet_id,
        operator_id,
        encrypt_public_key,
        sign_public_key,
    } = result;
    let wallet_id = parse_uuid(&wallet_id, "created wallet ID")?;
    let operator_id = parse_uuid(&operator_id, "created operator ID")?;
    let encrypt_public_key = normalize_public_key(
        &encrypt_public_key,
        "created operator encryption public key",
    )?;
    let sign_public_key =
        normalize_public_key(&sign_public_key, "created operator signing public key")?;
    ensure!(
        encrypt_public_key != sign_public_key,
        "created operator encryption and signing public keys must be distinct"
    );

    Ok(OperatorRecord {
        name,
        kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
            operator_id,
            wallet_id,
            path,
            encrypt_public_key,
            sign_public_key,
            extra: toml::Table::new(),
        }),
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid> {
    Uuid::parse_str(value).map_err(|_| anyhow!("{field} must be a UUID"))
}

fn normalize_public_key(value: &str, field: &str) -> Result<String> {
    let bytes = hex::decode(value).with_context(|| format!("{field} must be hex encoded"))?;
    ensure!(
        bytes.len() == 65 && bytes.first() == Some(&0x04),
        "{field} must be a 65-byte uncompressed P-256 public key"
    );
    let key = PublicKey::from_sec1_bytes(&bytes)
        .with_context(|| format!("{field} is not a valid P-256 point"))?;
    Ok(hex::encode(key.to_encoded_point(false).as_bytes()))
}

pub(crate) fn ensure_authenticated_org(
    authenticated_org_id: &str,
    configured_org_id: &str,
) -> Result<()> {
    ensure!(
        authenticated_org_id == configured_org_id,
        "authenticated organization ({authenticated_org_id}) does not match configured organization ({configured_org_id})"
    );
    Ok(())
}

fn hosted_activity_error(operation: &str, error: TurnkeyClientError) -> anyhow::Error {
    match error {
        TurnkeyClientError::ActivityRequiresApproval(activity_id) => anyhow!(
            "failed to {operation}: activity {activity_id} requires additional approvals or authentication"
        ),
        error => anyhow!("failed to {operation}: {error}"),
    }
}

fn timestamp_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qos_p256::P256Pair;

    const OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
    const WALLET_ID: &str = "22222222-2222-4222-8222-222222222222";

    fn public_keys() -> (String, String) {
        let first = P256Pair::generate().unwrap().public_key().to_bytes();
        let second = P256Pair::generate().unwrap().public_key().to_bytes();
        (hex::encode(&first[..65]), hex::encode(&second[65..]))
    }

    #[test]
    fn validates_and_normalizes_creation_result() {
        let (encrypt_public_key, sign_public_key) = public_keys();
        let record = operator_record_from_result(
            "operator".to_string(),
            DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string(),
            CreateTvcOperatorResult {
                wallet_id: WALLET_ID.to_uppercase(),
                operator_id: OPERATOR_ID.to_uppercase(),
                encrypt_public_key: encrypt_public_key.to_uppercase(),
                sign_public_key: sign_public_key.to_uppercase(),
            },
        )
        .unwrap();

        assert_eq!(
            record,
            OperatorRecord {
                name: "operator".to_string(),
                kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                    operator_id: Uuid::parse_str(OPERATOR_ID).unwrap(),
                    wallet_id: Uuid::parse_str(WALLET_ID).unwrap(),
                    path: DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string(),
                    encrypt_public_key,
                    sign_public_key,
                    extra: toml::Table::new(),
                }),
            }
        );
    }

    #[test]
    fn rejects_malformed_operator_id_from_creation_result() {
        let (encrypt_public_key, sign_public_key) = public_keys();
        let error = operator_record_from_result(
            "operator".to_string(),
            DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string(),
            CreateTvcOperatorResult {
                wallet_id: WALLET_ID.to_string(),
                operator_id: "not-a-uuid".to_string(),
                encrypt_public_key,
                sign_public_key,
            },
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "created operator ID must be a UUID");
    }
}
