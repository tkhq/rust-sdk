//! TVC operator resolution and manifest approval.
//!
//! Operator semantics are isolated here: resolution decides which backend an
//! operator uses and seals that decision behind the [`Signer`] port, so
//! everything downstream treats an operator as identity plus a signing
//! capability. Hosted-specific behavior lives in [`hosted`].

mod hosted;

pub use hosted::DEFAULT_HOSTED_OPERATOR_BASE_PATH;
pub(crate) use hosted::{
    HostedOperatorSpec, HostedOperatorWallet, ResolvedHostedOperator, create_hosted_operator,
    hosted_activity_error, resolve_hosted_operator, resolve_hosted_operator_encrypt_key,
};

use crate::approvals::ValidatedManifest;
use crate::client::build_client;
use crate::config::turnkey::{Config, OperatorKind};
use crate::local_operator_key::{
    LocalOperatorSeedSource, resolve_local_operator, resolve_registered_local_operator,
    select_local_operator,
};
use crate::pair::Signer;
use anyhow::{Context, Result, anyhow, bail, ensure};
use hosted::{
    HostedSigner, find_hosted_operator, select_hosted_operator, validated_hosted_operator,
};
use p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use qos_core::protocol::services::boot::{Approval, VersionedManifest};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::Uuid;

/// A validated, uncompressed P-256 operator public key.
///
/// String parsing accepts bare hexadecimal input with surrounding whitespace.
/// Display always emits canonical lowercase hexadecimal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperatorPublicKey(PublicKey);

/// Error returned when parsing an [`OperatorPublicKey`].
#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum OperatorPublicKeyParseError {
    /// The input was empty after trimming surrounding whitespace.
    #[error("must not be empty")]
    Empty,
    /// The input was not bare hexadecimal.
    #[error("must be bare hex encoded")]
    InvalidHex,
    /// The bytes were not an uncompressed 65-byte SEC1 point.
    #[error("must be a 65-byte uncompressed P-256 public key")]
    InvalidEncoding,
    /// The bytes did not identify a valid point on the P-256 curve.
    #[error("is not a valid P-256 point")]
    InvalidPoint,
}

impl FromStr for OperatorPublicKey {
    type Err = OperatorPublicKeyParseError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err(OperatorPublicKeyParseError::Empty);
        }

        let bytes = hex::decode(value).map_err(|_| OperatorPublicKeyParseError::InvalidHex)?;
        if bytes.len() != 65 || bytes.first() != Some(&0x04) {
            return Err(OperatorPublicKeyParseError::InvalidEncoding);
        }

        PublicKey::from_sec1_bytes(&bytes)
            .map(Self)
            .map_err(|_| OperatorPublicKeyParseError::InvalidPoint)
    }
}

impl Display for OperatorPublicKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0.to_encoded_point(false).as_bytes()))
    }
}

/// A non-serializable operator with its credentials resolved for use: common
/// identity plus the signing capability sealed behind the [`Signer`] port.
pub(crate) struct ResolvedOperator {
    /// Absent for an ad-hoc local seed override.
    name: Option<String>,
    /// Always present for hosted operators and optional for local operators.
    operator_id: Option<Uuid>,
    signer: Box<dyn Signer>,
}

impl ResolvedOperator {
    pub(crate) fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub(crate) fn id(&self) -> Option<Uuid> {
        self.operator_id
    }

    pub(crate) async fn approve_manifest(
        &self,
        manifest: &ValidatedManifest<'_>,
    ) -> Result<Approval> {
        let public_key = self.signer.public_key();
        let member = manifest_member(manifest, &public_key, self.name())?;
        let signature = self.signer.sign(manifest.manifest_hash().to_vec()).await?;
        let approval = Approval { signature, member };

        // Membership is already proven — `member` came out of the manifest
        // set — so verifying the fresh signature is the only remaining check.
        manifest.verify_approval(&approval)?;

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
    authenticated_org_id: Uuid,
    configured_org_id: Uuid,
) -> Result<()> {
    ensure!(
        authenticated_org_id == configured_org_id,
        "authenticated organization ({authenticated_org_id}) does not match configured organization ({configured_org_id})"
    );
    Ok(())
}

/// What the caller requires of the operator being resolved.
///
/// Checked during resolution, so an unsatisfiable requirement is rejected
/// before any backend dependencies (credentials, API clients) are acquired —
/// a machine with no credentials at all still gets the real refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignerRequirement {
    Any,
    /// The approval must be producible without the network (`--skip-post`).
    OfflineApproval,
}

/// Resolve the operator for an approval. The precedence is deliberate and
/// preserved exactly; there is no fallback between backends:
///
/// 1. An explicit seed is a local operator, always. A hosted `--operator-id`
///    alongside it is rejected.
/// 2. An operator ID naming a hosted registry record selects that hosted
///    operator — even when the org's default operator kind is local.
/// 3. Otherwise the active org's `default_operator_kind` decides, and never
///    crosses over: `local` resolves the sole local record (reconciling a
///    given ID against its configured one); `hosted` resolves the sole
///    hosted record when no ID names one, and never falls back to the
///    local key.
pub(crate) async fn resolve_operator(
    explicit: Option<LocalOperatorSeedSource>,
    operator_id: Option<Uuid>,
    requirement: SignerRequirement,
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
            signer: Box::new(resolve_local_operator(Some(explicit)).await?),
        });
    }

    let config = Config::load().await?;

    let hosted = operator_id
        .map(|id| find_hosted_operator(&config, &id))
        .transpose()?
        .flatten();

    if let Some(hosted) = hosted {
        // Hosted operators sign through the API, so an offline requirement
        // is unsatisfiable; refuse before touching credentials.
        if requirement == SignerRequirement::OfflineApproval {
            bail!("--skip-post is only supported for local operators");
        }

        let auth = build_client().await?;
        ensure_authenticated_org(auth.org_id, hosted.organization_id())?;

        return Ok(ResolvedOperator {
            name: Some(hosted.name().to_string()),
            operator_id: Some(hosted.operator_id()),
            signer: Box::new(HostedSigner::new(hosted, auth)),
        });
    }

    let (org_id, org) = config.active_org_config().ok_or_else(|| {
        anyhow!(
            "No active organization. Run `tvc login` first or provide \
             --operator-seed or --operator-seed-path."
        )
    })?;
    let org_name = config.display_name(org_id);

    if org.default_operator_kind == OperatorKind::Hosted {
        match operator_id {
            Some(id) => bail!("hosted operator ID '{id}' was not found in org '{org_name}'"),
            None => {
                // A hosted default cannot satisfy an offline approval;
                // refuse before selection or credentials.
                if requirement == SignerRequirement::OfflineApproval {
                    bail!("--skip-post is only supported for local operators");
                }

                let (record, hosted) =
                    select_hosted_operator(org).with_context(|| format!("org '{org_name}'"))?;
                let hosted = validated_hosted_operator(org_id, &record.name, hosted)?;
                let auth = build_client().await?;
                ensure_authenticated_org(auth.org_id, hosted.organization_id())?;

                return Ok(ResolvedOperator {
                    name: Some(hosted.name().to_string()),
                    operator_id: Some(hosted.operator_id()),
                    signer: Box::new(HostedSigner::new(hosted, auth)),
                });
            }
        }
    }

    let (operator, local) =
        select_local_operator(org).with_context(|| format!("org '{org_name}'"))?;

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
        signer: Box::new(resolve_registered_local_operator(local.key_path.clone()).await?),
    })
}

fn parse_uuid(value: &str, field: &str) -> Result<Uuid> {
    Uuid::parse_str(value).map_err(|_| anyhow!("{field} must be a UUID"))
}

pub(crate) fn timestamp_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use qos_p256::P256Pair;

    fn public_keys() -> (String, String) {
        let first = P256Pair::generate().unwrap().public_key().to_bytes();
        let second = P256Pair::generate().unwrap().public_key().to_bytes();
        (hex::encode(&first[..65]), hex::encode(&second[65..]))
    }

    #[test]
    fn operator_public_key_parses_and_canonicalizes() {
        let (key, _) = public_keys();
        let parsed: OperatorPublicKey = format!("  {}  ", key.to_uppercase()).parse().unwrap();

        assert_eq!(parsed.to_string(), key);
    }

    #[test]
    fn operator_public_key_rejects_invalid_inputs() {
        assert_eq!(
            " ".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::Empty
        );
        assert_eq!(
            "not-hex".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidHex
        );
        assert_eq!(
            "04abcd".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidEncoding
        );

        let (uncompressed, _) = public_keys();
        let public_key = PublicKey::from_sec1_bytes(&hex::decode(uncompressed).unwrap()).unwrap();
        let compressed = hex::encode(public_key.to_encoded_point(true).as_bytes());
        assert_eq!(
            compressed.parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidEncoding
        );

        let invalid_point = format!("04{}", "00".repeat(64));
        assert_eq!(
            invalid_point.parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidPoint
        );
    }

    #[test]
    fn authenticated_org_must_match_configured_org() {
        let authenticated = Uuid::from_u128(0xA1);
        let configured = Uuid::from_u128(0xA2);

        assert_eq!(
            ensure_authenticated_org(authenticated, configured)
                .unwrap_err()
                .to_string(),
            format!(
                "authenticated organization ({authenticated}) does not match \
                 configured organization ({configured})"
            )
        );
    }
}
