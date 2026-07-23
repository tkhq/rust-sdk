//! TVC operator creation and manifest approval.

use crate::approvals::verify_own_approval;
use crate::client::AuthenticatedClient;
use crate::config::turnkey::{
    Config, HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind,
};
use crate::local_operator_key::{
    LocalOperatorSeedSource, resolve_local_operator, resolve_registered_local_operator,
};
use crate::pair::{LocalPair, Pair};
use anyhow::{Context, Result, anyhow, bail, ensure};
use p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use qos_core::protocol::services::boot::{Approval, VersionedManifest};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::TurnkeyClientError;
use turnkey_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
use turnkey_client::generated::{
    CreateTvcOperatorIntent, CreateTvcOperatorResult, SignRawPayloadIntentV2, SignRawPayloadResult,
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

fn composite_public_key(record: &HostedOperatorRecord) -> Result<Vec<u8>> {
    let encrypt = hex::decode(normalize_public_key(
        &record.encrypt_public_key,
        "hosted operator encryption public key",
    )?)?;
    let sign = hex::decode(normalize_public_key(
        &record.sign_public_key,
        "hosted operator signing public key",
    )?)?;
    ensure!(
        encrypt != sign,
        "hosted operator encryption and signing public keys must be distinct"
    );

    Ok(encrypt.into_iter().chain(sign).collect())
}

fn validate_hosted_record(
    name: &str,
    record: HostedOperatorRecord,
) -> Result<HostedOperatorRecord> {
    let HostedOperatorRecord {
        operator_id,
        wallet_id,
        path,
        encrypt_public_key,
        sign_public_key,
        extra,
    } = record;
    ensure!(
        !name.trim().is_empty(),
        "hosted operator name must not be empty"
    );
    ensure!(
        !path.trim().is_empty(),
        "hosted operator account path must not be empty"
    );

    let validated = HostedOperatorRecord {
        operator_id,
        wallet_id,
        path,
        encrypt_public_key: normalize_public_key(
            &encrypt_public_key,
            "hosted operator encryption public key",
        )?,
        sign_public_key: normalize_public_key(
            &sign_public_key,
            "hosted operator signing public key",
        )?,
        extra,
    };
    let _ = composite_public_key(&validated)?;
    Ok(validated)
}

/// A non-serializable operator with its credentials resolved for use.
pub(crate) struct ResolvedOperator {
    /// Absent for an ad-hoc local seed override.
    name: Option<String>,
    /// Always present for hosted operators and optional for local operators.
    operator_id: Option<Uuid>,
    /// Organization from which a registered operator was resolved.
    organization_id: Option<String>,
    pub(crate) kind: ResolvedOperatorKind,
}

/// Kind-specific runtime capability held by a [`ResolvedOperator`].
pub(crate) enum ResolvedOperatorKind {
    Local(LocalPair),
    Hosted(HostedOperatorRecord),
}

/// Shared dependencies for operator-level workflows.
pub(crate) struct OperatorCtx<'a> {
    pub(crate) auth: Option<&'a AuthenticatedClient>,
}

impl ResolvedOperator {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn id(&self) -> Option<Uuid> {
        self.operator_id
    }

    pub(crate) fn is_hosted(&self) -> bool {
        matches!(self.kind, ResolvedOperatorKind::Hosted(_))
    }

    pub(crate) fn public_key(&self) -> Result<Vec<u8>> {
        match &self.kind {
            ResolvedOperatorKind::Local(pair) => Ok(pair.public_key()),
            ResolvedOperatorKind::Hosted(record) => composite_public_key(record),
        }
    }

    pub(crate) async fn approve_manifest(
        &self,
        ctx: &OperatorCtx<'_>,
        manifest: &VersionedManifest,
    ) -> Result<Approval> {
        let public_key = self.public_key()?;
        let member = manifest_member(manifest, &public_key, self.name())?;
        let signature = match &self.kind {
            ResolvedOperatorKind::Local(pair) => {
                pair.sign(manifest.manifest_hash().to_vec()).await?
            }
            ResolvedOperatorKind::Hosted(record) => {
                let auth = ctx
                    .auth
                    .context("authenticated client required for hosted operator approval")?;
                let organization_id = self
                    .organization_id
                    .as_deref()
                    .context("configured organization required for hosted operator approval")?;
                ensure_authenticated_org(&auth.org_id, organization_id)?;
                sign_hosted_manifest(auth, record, manifest).await?
            }
        };

        let approval = Approval { signature, member };

        verify_own_approval(manifest, &approval).context(
            "freshly generated approval failed verification; \
             check that the operator key matches the manifest set member key",
        )?;

        Ok(approval)
    }
}

fn manifest_member(
    manifest: &VersionedManifest,
    public_key: &[u8],
    operator_name: Option<&str>,
) -> Result<qos_core::protocol::services::boot::QuorumMember> {
    manifest
        .manifest_set()
        .members
        .iter()
        .find(|member| member.pub_key == public_key)
        .cloned()
        .ok_or_else(|| match operator_name {
            Some(name) => anyhow!(
                "operator '{name}' ({}) not part of manifest set",
                hex::encode(public_key)
            ),
            None => anyhow!(
                "operator ({}) not part of manifest set",
                hex::encode(public_key)
            ),
        })
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

pub(crate) async fn resolve_operator(
    explicit: Option<LocalOperatorSeedSource>,
    operator_id: Option<Uuid>,
) -> Result<ResolvedOperator> {
    if let Some(explicit) = explicit {
        if let Some(id) = operator_id {
            let config = Config::load().await?;
            ensure!(
                find_hosted_operator(&config, &id)?.is_none(),
                "explicit local operator seed cannot be used with a hosted operator ID"
            );
        }
        return Ok(ResolvedOperator {
            name: None,
            operator_id,
            organization_id: None,
            kind: ResolvedOperatorKind::Local(resolve_local_operator(Some(explicit)).await?),
        });
    }

    let config = Config::load().await?;
    let hosted = match operator_id {
        Some(id) => find_hosted_operator(&config, &id)?,
        None => None,
    };

    if let Some((organization_id, name, record)) = hosted {
        return Ok(ResolvedOperator {
            name: Some(name),
            operator_id: Some(record.operator_id),
            organization_id: Some(organization_id),
            kind: ResolvedOperatorKind::Hosted(record),
        });
    }

    let (alias, org) = config.active_org_config().ok_or_else(|| {
        anyhow!(
            "No active organization. Run `tvc login` first or provide \
             --operator-seed or --operator-seed-path."
        )
    })?;
    if org.default_operator_kind == OperatorKind::Hosted {
        match operator_id {
            Some(id) => bail!("hosted operator ID '{id}' was not found in org '{alias}'"),
            None => bail!("--operator-id is required to approve with a hosted operator"),
        }
    }

    let operator = org.select_local_operator(alias)?;
    let OperatorRecordKind::Local(local) = &operator.kind else {
        return Err(anyhow!("selected operator is not local"));
    };

    let configured_operator_id = local
        .operator_id
        .as_deref()
        .map(|id| parse_uuid(id, "configured local operator ID"))
        .transpose()?;
    let resolved_operator_id = match (configured_operator_id, operator_id) {
        (Some(configured), Some(requested)) => {
            ensure!(
                configured == requested,
                "requested operator ID ({requested}) does not match configured local operator ID ({configured})"
            );
            Some(configured)
        }
        (configured, requested) => configured.or(requested),
    };

    Ok(ResolvedOperator {
        name: Some(operator.name.clone()),
        operator_id: resolved_operator_id,
        organization_id: Some(org.id.clone()),
        kind: ResolvedOperatorKind::Local(
            resolve_registered_local_operator(local.key_path.clone()).await?,
        ),
    })
}

fn find_hosted_operator(
    config: &Config,
    operator_id: &Uuid,
) -> Result<Option<(String, String, HostedOperatorRecord)>> {
    let Some((_, org)) = config.active_org_config() else {
        return Ok(None);
    };
    let matches: Vec<_> = org
        .operators
        .iter()
        .filter_map(|operator| match &operator.kind {
            OperatorRecordKind::Hosted(hosted) => {
                (hosted.operator_id == *operator_id).then_some((operator.name.as_str(), hosted))
            }
            OperatorRecordKind::Local(_) => None,
        })
        .collect();

    match matches.as_slice() {
        [] => Ok(None),
        [(name, record)] => Ok(Some((
            org.id.clone(),
            (*name).to_string(),
            validate_hosted_record(name, (**record).clone())?,
        ))),
        _ => bail!("multiple hosted operators have ID {operator_id}"),
    }
}

async fn sign_hosted_manifest(
    auth: &AuthenticatedClient,
    record: &HostedOperatorRecord,
    manifest: &VersionedManifest,
) -> Result<Vec<u8>> {
    let intent = hosted_signing_intent(record.sign_public_key.clone(), &manifest.manifest_hash());
    let result = auth
        .client
        .sign_raw_payload(auth.org_id.clone(), timestamp_ms()?, intent)
        .await
        .map_err(|error| hosted_activity_error("sign manifest with hosted operator", error))?;

    signature_bytes(result.result)
}

fn hosted_signing_intent(sign_with: String, manifest_hash: &[u8]) -> SignRawPayloadIntentV2 {
    SignRawPayloadIntentV2 {
        sign_with,
        payload: hex::encode(manifest_hash),
        encoding: PayloadEncoding::Hexadecimal,
        hash_function: HashFunction::Sha256,
    }
}

fn signature_bytes(result: SignRawPayloadResult) -> Result<Vec<u8>> {
    let SignRawPayloadResult { r, s, v: _ } = result;
    let r = signature_component(&r, "r")?;
    let s = signature_component(&s, "s")?;
    Ok(r.into_iter().chain(s).collect())
}

fn signature_component(value: &str, component: &str) -> Result<Vec<u8>> {
    let bytes = hex::decode(value)
        .with_context(|| format!("hosted signature component {component} must be hex encoded"))?;
    ensure!(
        bytes.len() == 32,
        "hosted signature component {component} must be exactly 32 bytes"
    );
    Ok(bytes)
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

    #[test]
    fn resolved_hosted_operator_exposes_common_identity() {
        let record = hosted_record();
        let operator = ResolvedOperator {
            name: Some("operator".to_string()),
            operator_id: Some(record.operator_id),
            organization_id: Some("org-id".to_string()),
            kind: ResolvedOperatorKind::Hosted(record.clone()),
        };

        assert_eq!(operator.name(), Some("operator"));
        assert_eq!(operator.id(), Some(Uuid::parse_str(OPERATOR_ID).unwrap()));
        assert!(matches!(
            &operator.kind,
            ResolvedOperatorKind::Hosted(actual) if actual == &record
        ));
        assert_eq!(
            operator.public_key().unwrap(),
            composite_public_key(&record).unwrap()
        );
    }

    #[test]
    fn hosted_operator_lookup_is_scoped_to_active_organization() {
        let operator_id = Uuid::parse_str(OPERATOR_ID).unwrap();
        let config = Config {
            active_org: Some("active".to_string()),
            orgs: std::collections::HashMap::from([
                (
                    "active".to_string(),
                    crate::config::turnkey::OrgConfig {
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
                    crate::config::turnkey::OrgConfig {
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

        assert!(
            find_hosted_operator(&config, &operator_id)
                .unwrap()
                .is_none()
        );
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
    }
}
