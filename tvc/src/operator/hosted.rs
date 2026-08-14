//! Turnkey-hosted operators: creation, registry resolution, and the
//! API-signing [`Signer`] adapter.

use super::{OperatorPublicKey, timestamp_ms};
use crate::client::AuthenticatedClient;
use crate::config::turnkey::{Config, HostedOperatorRecord, OperatorRecord, OperatorRecordKind};
use crate::pair::{Signer, SignerFuture};
use anyhow::{Context, Result, anyhow, bail, ensure};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use std::ops::Deref;
use std::str::FromStr;
use turnkey_client::{
    TurnkeyClientError,
    generated::{
        CreateTvcOperatorResult, SignRawPayloadIntentV2, SignRawPayloadResult,
        immutable::common::v1::{HashFunction, PayloadEncoding},
    },
};
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

#[derive(Debug, PartialEq, Eq)]
pub struct EncryptPubKey(OperatorPublicKey);
#[derive(Debug, PartialEq, Eq)]
pub struct SignPubKey(OperatorPublicKey);

impl From<EncryptPubKey> for OperatorPublicKey {
    fn from(key: EncryptPubKey) -> Self {
        key.0
    }
}

impl From<SignPubKey> for OperatorPublicKey {
    fn from(key: SignPubKey) -> Self {
        key.0
    }
}

impl FromStr for EncryptPubKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Ok(Self(s.parse().context("encrypt public key parsing error")?))
    }
}

impl FromStr for SignPubKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Ok(Self(s.parse().context("sign public key parsing error")?))
    }
}

impl Deref for EncryptPubKey {
    type Target = OperatorPublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for SignPubKey {
    type Target = OperatorPublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A hosted operator's encryption and signing keys: two distinct P-256
/// points, together forming the qos composite public key. `R` is the role
/// whose keys these are; its `Display` form prefixes parse errors.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
struct OperatorKeyPair {
    encrypt: EncryptPubKey,
    sign: SignPubKey,
}

impl OperatorKeyPair {
    /// Parse each key and require the pair to be distinct; the role's
    /// `Display` form says whose keys these are in errors.
    fn try_new(encrypt: EncryptPubKey, sign: SignPubKey) -> Result<Self> {
        ensure!(
            encrypt.0 != sign.0,
            "encryption and signing public keys must be distinct"
        );

        Ok(Self { encrypt, sign })
    }

    /// The qos composite public key: `encrypt ‖ sign`, both as uncompressed
    /// SEC1 points.
    fn composite(&self) -> Vec<u8> {
        // Two 65-byte uncompressed SEC1 points.
        const COMPOSITE_PUBLIC_KEY_LEN: usize = 130;

        let mut composite = Vec::with_capacity(COMPOSITE_PUBLIC_KEY_LEN);
        composite.extend_from_slice(self.encrypt.deref().0.to_encoded_point(false).as_bytes());
        composite.extend_from_slice(self.sign.deref().0.to_encoded_point(false).as_bytes());
        composite
    }
}

/// The context of a create-operator request coupled with its API result,
/// ready to parse into a registry-ready record.
pub(crate) struct CreateOperatorRequestResult {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) result: CreateTvcOperatorResult,
}

impl TryFrom<CreateOperatorRequestResult> for OperatorRecord {
    type Error = anyhow::Error;

    fn try_from(result: CreateOperatorRequestResult) -> Result<Self> {
        let CreateOperatorRequestResult { name, path, result } = result;

        let CreateTvcOperatorResult {
            wallet_id,
            operator_id,
            encrypt_public_key,
            sign_public_key,
        } = result;

        let wallet_id =
            Uuid::parse_str(&wallet_id).map_err(|_| anyhow!("created wallet ID must be a UUID"))?;
        let operator_id = Uuid::parse_str(&operator_id)
            .map_err(|_| anyhow!("created operator ID must be a UUID"))?;
        let encrypt_public_key = encrypt_public_key
            .parse()
            .context("create operator key parsing error")?;
        let sign_public_key = sign_public_key
            .parse()
            .context("create operator key parsing error")?;
        let keys = OperatorKeyPair::try_new(encrypt_public_key, sign_public_key)?;

        Ok(Self {
            name,
            kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                operator_id,
                wallet_id,
                path,
                encrypt_public_key: keys.encrypt.to_string(),
                sign_public_key: keys.sign.to_string(),
                extra: toml::Table::new(),
            }),
        })
    }
}

/// One validated hosted operator resolved from the active organization.
/// Its keys are parsed exactly once, at resolution.
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
pub(crate) struct ResolvedHostedOperator {
    organization_id: String,
    name: String,
    operator_id: Uuid,
    keys: OperatorKeyPair,
}

impl ResolvedHostedOperator {
    /// Parse and validate a hosted registry record into a resolved operator.
    pub(super) fn from_registry(
        organization_id: String,
        name: &str,
        record: &HostedOperatorRecord,
    ) -> Result<Self> {
        let HostedOperatorRecord {
            operator_id,
            wallet_id: _,
            path,
            encrypt_public_key,
            sign_public_key,
            extra: _,
        } = record;

        ensure!(
            !name.trim().is_empty(),
            "hosted operator name must not be empty"
        );
        ensure!(
            !path.trim().is_empty(),
            "hosted operator account path must not be empty"
        );

        let encrypt_public_key = encrypt_public_key
            .parse()
            .context("hosted operator key parsing error")?;
        let sign_public_key = sign_public_key
            .parse()
            .context("hosted operator key parsing error")?;

        let keys = OperatorKeyPair::try_new(encrypt_public_key, sign_public_key)?;

        Ok(Self {
            organization_id,
            name: name.to_string(),
            operator_id: *operator_id,
            keys,
        })
    }

    pub(crate) fn organization_id(&self) -> &str {
        &self.organization_id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn operator_id(&self) -> Uuid {
        self.operator_id
    }

    pub(crate) fn encrypt_public_key(&self) -> &EncryptPubKey {
        &self.keys.encrypt
    }

    pub(crate) fn sign_public_key(&self) -> &SignPubKey {
        &self.keys.sign
    }

    pub(crate) fn composite_public_key(&self) -> Vec<u8> {
        self.keys.composite()
    }
}

impl Config {
    /// Find the hosted registry record with `operator_id` in the active
    /// organization, if any, validated for use.
    pub(super) fn find_hosted_operator(
        &self,
        operator_id: &Uuid,
    ) -> Result<Option<ResolvedHostedOperator>> {
        let Some((_, org)) = self.active_org_config() else {
            return Ok(None);
        };

        let mut matches = org
            .hosted_operators()
            .filter(|(_, hosted)| hosted.operator_id == *operator_id);

        match (matches.next(), matches.next()) {
            (None, _) => Ok(None),
            (Some((name, hosted)), None) => Ok(Some(ResolvedHostedOperator::from_registry(
                org.id.clone(),
                name,
                hosted,
            )?)),
            (Some(_), Some(_)) => bail!("multiple hosted operators have ID {operator_id}"),
        }
    }

    /// Resolve one validated hosted operator from the active organization.
    pub(crate) fn resolve_hosted_operator(
        &self,
        operator_id: &Uuid,
    ) -> Result<ResolvedHostedOperator> {
        let (alias, _) = self
            .active_org_config()
            .context("No active organization. Run `tvc login` first.")?;

        self.find_hosted_operator(operator_id)?.ok_or_else(|| {
            anyhow!("hosted operator ID '{operator_id}' was not found in org '{alias}'")
        })
    }

    /// Resolve a hosted operator's encryption public key from the active
    /// organization.
    pub(crate) fn resolve_hosted_operator_encrypt_key(
        &self,
        operator_id: &Uuid,
    ) -> Result<EncryptPubKey> {
        Ok(self.resolve_hosted_operator(operator_id)?.keys.encrypt)
    }
}

/// A Turnkey-hosted operator signing through the API with an owned
/// authenticated client.
pub(super) struct HostedSigner {
    operator: ResolvedHostedOperator,
    auth: AuthenticatedClient,
}

impl HostedSigner {
    /// Assemble a hosted signer from its finished dependencies. The caller
    /// has already verified that `auth` is authenticated against the
    /// operator's organization.
    pub(super) fn new(operator: ResolvedHostedOperator, auth: AuthenticatedClient) -> Self {
        Self { operator, auth }
    }
}

impl Signer for HostedSigner {
    fn sign(&self, message: &[u8]) -> SignerFuture<'_, Result<Vec<u8>>> {
        let intent = {
            let sign_with = self.operator.sign_public_key().to_string();

            SignRawPayloadIntentV2 {
                sign_with,
                payload: hex::encode(message),
                encoding: PayloadEncoding::Hexadecimal,
                hash_function: HashFunction::Sha256,
            }
        };

        Box::pin(async move {
            // PURE-DEPS-REVIEW T24 (low): clock read (timestamp_ms) inside the
            // signer; injecting the timestamp would make intent assembly
            // deterministic. See tvc/PURE_DEPS_PLAN_0.md.
            let result = self
                .auth
                .client
                .sign_raw_payload(self.auth.org_id.clone(), timestamp_ms()?, intent)
                .await
                .map_err(|error| {
                    hosted_activity_error("sign manifest with hosted operator", error)
                })?;

            let SignRawPayloadResult { r, s, v: _ } = result.result;

            [('r', r), ('s', s)]
                .into_iter()
                .map(|(component, val)| {
                    let bytes = hex::decode(val).context(format!(
                        "hosted signature component {component} must be hex encoded"
                    ))?;

                    ensure!(
                        bytes.len() == 32,
                        "hosted signature component {component} must be exactly 32 bytes"
                    );

                    Ok(bytes)
                })
                .try_fold(Vec::with_capacity(64), |mut combined, val| {
                    combined.extend(val?);
                    Ok(combined)
                })
        })
    }

    fn public_key(&self) -> Vec<u8> {
        self.operator.composite_public_key()
    }
}

pub(crate) fn hosted_activity_error(operation: &str, error: TurnkeyClientError) -> anyhow::Error {
    let context = match &error {
        TurnkeyClientError::ActivityRequiresApproval(activity_id) => format!(
            "failed to {operation}: activity {activity_id} requires additional approvals or authentication"
        ),
        _ => format!("failed to {operation}"),
    };

    anyhow::Error::new(error).context(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        LocalOperatorRecord, OperatorKind, OrgConfig, SelectHostedOperatorError,
    };
    use crate::operator::OperatorPublicKeyParseError;
    use qos_p256::P256Pair;
    use std::collections::HashMap;
    use std::path::PathBuf;

    const OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
    const WALLET_ID: &str = "22222222-2222-4222-8222-222222222222";

    fn public_keys() -> (String, String) {
        let first = P256Pair::generate().unwrap().public_key().to_bytes();
        let second = P256Pair::generate().unwrap().public_key().to_bytes();
        (hex::encode(&first[..65]), hex::encode(&second[65..]))
    }

    fn hosted_record() -> HostedOperatorRecord {
        let (encrypt_public_key, sign_public_key) = public_keys();
        HostedOperatorRecord {
            operator_id: Uuid::parse_str(OPERATOR_ID).unwrap(),
            wallet_id: Uuid::parse_str(WALLET_ID).unwrap(),
            path: DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string(),
            encrypt_public_key,
            sign_public_key,
            extra: toml::Table::new(),
        }
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

    fn hosted_operator(name: &str, record: HostedOperatorRecord) -> OperatorRecord {
        OperatorRecord {
            name: name.to_string(),
            kind: OperatorRecordKind::Hosted(record),
        }
    }

    #[test]
    fn resolves_hosted_operator_encrypt_key_from_active_org() {
        let record = hosted_record();
        let expected = record.encrypt_public_key.parse().unwrap();
        let config = config_with_operators(vec![hosted_operator("hosted", record)]);
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();

        let resolved = config
            .resolve_hosted_operator_encrypt_key(&operator_id)
            .unwrap();

        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolves_complete_hosted_operator_from_active_org() {
        let record = hosted_record();
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();
        let expected = ResolvedHostedOperator {
            organization_id: "org-id".to_string(),
            name: "hosted".to_string(),
            operator_id,
            keys: OperatorKeyPair {
                encrypt: record.encrypt_public_key.parse().unwrap(),
                sign: record.sign_public_key.parse().unwrap(),
            },
        };
        let expected_composite = expected.composite_public_key();
        let config = config_with_operators(vec![hosted_operator("hosted", record)]);

        let resolved = config.resolve_hosted_operator(&operator_id).unwrap();

        assert_eq!(resolved, expected);
        assert_eq!(resolved.composite_public_key(), expected_composite);
    }

    #[test]
    fn hosted_operator_resolution_requires_active_org_and_matching_hosted_record() {
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();
        let no_active = Config::default();
        assert_eq!(
            no_active
                .resolve_hosted_operator_encrypt_key(&operator_id)
                .unwrap_err()
                .to_string(),
            "No active organization. Run `tvc login` first."
        );

        let local = OperatorRecord {
            name: "local".to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path: PathBuf::from("operator.json"),
                operator_id: Some(OPERATOR_ID.to_string()),
                extra: toml::Table::new(),
            }),
        };
        let config = config_with_operators(vec![local]);
        assert_eq!(
            config
                .resolve_hosted_operator_encrypt_key(&operator_id)
                .unwrap_err()
                .to_string(),
            "hosted operator ID '11111111-1111-4111-8111-111111111111' was not found in org 'active'"
        );

        let mut cross_org = config_with_operators(Vec::new());
        let mut inactive_org = cross_org.orgs["active"].clone();
        inactive_org.id = "inactive-org-id".to_string();
        inactive_org.operators = vec![hosted_operator("hosted", hosted_record())];
        cross_org.orgs.insert("inactive".to_string(), inactive_org);
        assert_eq!(
            cross_org
                .resolve_hosted_operator_encrypt_key(&operator_id)
                .unwrap_err()
                .to_string(),
            "hosted operator ID '11111111-1111-4111-8111-111111111111' was not found in org 'active'"
        );
    }

    #[test]
    fn hosted_operator_resolution_rejects_duplicate_and_malformed_records() {
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();
        let record = hosted_record();
        let duplicate = config_with_operators(vec![
            hosted_operator("first", record.clone()),
            hosted_operator("second", record.clone()),
        ]);
        assert_eq!(
            duplicate
                .resolve_hosted_operator_encrypt_key(&operator_id)
                .unwrap_err()
                .to_string(),
            "multiple hosted operators have ID 11111111-1111-4111-8111-111111111111"
        );

        let mut malformed = record;
        malformed.encrypt_public_key = "not-hex".to_string();
        let malformed = config_with_operators(vec![hosted_operator("hosted", malformed)]);
        let error = malformed
            .resolve_hosted_operator_encrypt_key(&operator_id)
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<OperatorPublicKeyParseError>(),
            Some(&OperatorPublicKeyParseError::InvalidHex)
        );
    }

    #[test]
    fn selects_the_sole_hosted_operator_ignoring_locals() {
        let local = OperatorRecord {
            name: "local".to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path: PathBuf::from("operator.json"),
                operator_id: None,
                extra: toml::Table::new(),
            }),
        };
        let config = config_with_operators(vec![local, hosted_operator("hosted", hosted_record())]);
        let org = &config.orgs["active"];

        let (name, hosted) = org.select_hosted_operator().unwrap();

        assert_eq!(name, "hosted");
        assert_eq!(hosted.operator_id, Uuid::parse_str(OPERATOR_ID).unwrap());
    }

    #[test]
    fn selecting_a_hosted_operator_requires_one_to_exist() {
        let config = config_with_operators(Vec::new());
        let org = &config.orgs["active"];

        assert!(matches!(
            org.select_hosted_operator(),
            Err(SelectHostedOperatorError::NoHostedOperator)
        ));
    }

    #[test]
    fn selecting_a_hosted_operator_refuses_multiple() {
        let config = config_with_operators(vec![
            hosted_operator("first", hosted_record()),
            hosted_operator("second", hosted_record()),
        ]);
        let org = &config.orgs["active"];

        assert!(matches!(
            org.select_hosted_operator(),
            Err(SelectHostedOperatorError::MultipleHostedOperators)
        ));
    }

    #[test]
    fn hosted_operator_lookup_is_scoped_to_active_organization() {
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();
        let config = Config {
            active_org: Some("active".to_string()),
            orgs: HashMap::from([
                (
                    "active".to_string(),
                    OrgConfig {
                        id: "active-org".to_string(),
                        api_key_path: "active-api.json".into(),
                        api_base_url: "https://api.turnkey.com".to_string(),
                        default_operator_kind: OperatorKind::Local,
                        operators: Vec::new(),
                        extra: toml::Table::new(),
                    },
                ),
                (
                    "inactive".to_string(),
                    OrgConfig {
                        id: "inactive-org".to_string(),
                        api_key_path: "inactive-api.json".into(),
                        api_base_url: "https://api.turnkey.com".to_string(),
                        default_operator_kind: OperatorKind::Hosted,
                        operators: vec![OperatorRecord {
                            name: "hosted".to_string(),
                            kind: OperatorRecordKind::Hosted(hosted_record()),
                        }],
                        extra: toml::Table::new(),
                    },
                ),
            ]),
            ..Config::default()
        };

        assert!(config.find_hosted_operator(&operator_id).unwrap().is_none());
    }

    #[test]
    fn validates_and_normalizes_creation_result() {
        let (encrypt_public_key, sign_public_key) = public_keys();
        let result = CreateTvcOperatorResult {
            wallet_id: WALLET_ID.to_uppercase(),
            operator_id: OPERATOR_ID.to_uppercase(),
            encrypt_public_key: encrypt_public_key.to_uppercase(),
            sign_public_key: sign_public_key.to_uppercase(),
        };

        let result = CreateOperatorRequestResult {
            name: "operator".into(),
            path: DEFAULT_HOSTED_OPERATOR_BASE_PATH.into(),
            result,
        };

        let record = OperatorRecord::try_from(result).unwrap();

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
        let name = "operator".into();
        let path = DEFAULT_HOSTED_OPERATOR_BASE_PATH.into();

        let result = CreateTvcOperatorResult {
            wallet_id: WALLET_ID.to_string(),
            operator_id: "not-a-uuid".to_string(),
            encrypt_public_key,
            sign_public_key,
        };

        let result = CreateOperatorRequestResult { name, path, result };
        let error = OperatorRecord::try_from(result).unwrap_err();

        assert_eq!(error.to_string(), "created operator ID must be a UUID");
    }

    #[test]
    fn policy_error_includes_activity_id_and_operation() {
        let error = hosted_activity_error(
            "sign manifest with hosted operator",
            TurnkeyClientError::ActivityRequiresApproval("activity-id".to_string()),
        );

        assert_eq!(
            error.to_string(),
            "failed to sign manifest with hosted operator: activity activity-id requires additional approvals or authentication"
        );
        assert!(matches!(
            error.downcast_ref::<TurnkeyClientError>(),
            Some(TurnkeyClientError::ActivityRequiresApproval(activity_id))
                if activity_id == "activity-id"
        ));
    }

    #[test]
    fn hosted_activity_error_preserves_client_error_source() {
        let error = hosted_activity_error(
            "create hosted TVC operator",
            TurnkeyClientError::UnexpectedHttpStatus(403, "forbidden".to_string()),
        );

        assert_eq!(error.to_string(), "failed to create hosted TVC operator");
        assert_eq!(
            crate::errors::render_error_chain(&error),
            "failed to create hosted TVC operator: HTTP response was not successful: 403 (forbidden)"
        );
        let classification = crate::errors::classify(&error);
        assert_eq!(classification.code, crate::errors::ErrorCode::Unauthorized);
        assert_eq!(classification.http_status, Some(403));
        assert!(matches!(
            error.downcast_ref::<TurnkeyClientError>(),
            Some(TurnkeyClientError::UnexpectedHttpStatus(403, body))
                if body == "forbidden"
        ));
    }
}
