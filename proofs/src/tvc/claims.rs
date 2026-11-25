//! Reads the JSON that TKMS signs in TVC app proofs. The signed JSON puts each
//! claim directly in the payload, while generated protobuf types wrap the claim.
use super::{
    P256PublicKey, TvcProvenanceError,
    endorsement::AuthenticatedPayload,
    facts::{CheckOutcome, ProofCheck},
};
use serde::Deserialize;
use turnkey_client::RequestFingerprint;
use turnkey_enclave_encrypt::QuorumPublicKey;

#[derive(Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
enum ClaimWire {
    #[serde(rename = "APP_PROOF_TYPE_ADDRESS_DERIVATION")]
    Operator {
        #[serde(rename = "timestampMs")]
        timestamp_ms: String,
        #[serde(rename = "requestContext")]
        request_context: ContextWire<u32>,
        #[serde(rename = "addressDerivationProof")]
        body: OperatorBodyWire,
    },
    #[serde(rename = "APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE")]
    Quorum {
        #[serde(rename = "timestampMs")]
        timestamp_ms: String,
        #[serde(rename = "requestContext")]
        request_context: ContextWire<u32>,
        #[serde(rename = "tvcQuorumKeyCreateProof")]
        body: QuorumBodyWire,
    },
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContextWire<Version> {
    version: Version,
    verified_signature_digests: Vec<String>,
}

struct V1;

impl ContextWire<u32> {
    fn into_v1(self) -> Result<ContextWire<V1>, TvcProvenanceError> {
        if self.version != 1 {
            return Err(TvcProvenanceError::UnsupportedContext);
        }
        Ok(ContextWire {
            version: V1,
            verified_signature_digests: self.verified_signature_digests,
        })
    }
}

/// Request fingerprints carried in a signed claim.
pub(super) struct Context {
    pub(super) digests: Vec<RequestFingerprint>,
}
/// One signed key derivation claim with its submission context.
pub(super) struct OperatorClaim {
    pub(super) context: Context,
    pub(super) body: OperatorBody,
}
/// A signed quorum key creation claim with its submission context.
pub(super) struct QuorumClaim {
    pub(super) context: Context,
    pub(super) body: QuorumBody,
}
/// A decoded key claim for TVC acceptance.
pub(super) enum Claim {
    Operator(OperatorClaim),
    Quorum(QuorumClaim),
}

impl Claim {
    /// Reads the already-authenticated payload. It must describe a supported
    /// key operation and include valid request fingerprints.
    pub(super) fn decode(payload: AuthenticatedPayload) -> CheckOutcome<Self> {
        CheckOutcome::new(ProofCheck::ClaimDecode, Self::try_from(payload))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperatorBodyWire {
    organization_id: String,
    wallet_id: String,
    derivation_path: String,
    address: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QuorumBodyWire {
    organization_id: String,
    quorum_key_id: String,
    quorum_public_key: String,
    threshold: u32,
    operator_encrypt_keys: Vec<String>,
}

/// Organization, wallet, derivation path, and public key for one operator role.
pub(super) struct OperatorBody {
    pub(super) organization_id: String,
    pub(super) wallet_id: String,
    pub(super) derivation_path: String,
    pub(super) address: P256PublicKey,
}
/// Created quorum key, its threshold, and its recipients in request order.
pub(super) struct QuorumBody {
    pub(super) organization_id: String,
    pub(super) quorum_key_id: String,
    pub(super) quorum_public_key: QuorumPublicKey,
    pub(super) threshold: u32,
    pub(super) operator_encrypt_keys: Vec<P256PublicKey>,
}

#[derive(Deserialize)]
struct Routing {
    #[serde(rename = "type")]
    kind: String,
}

/// Caps the payload size and reads the type so policy proofs can be skipped.
/// The type label alone cannot establish a key claim.
pub(super) fn policy_envelope(bytes: &str) -> CheckOutcome<bool> {
    CheckOutcome::run(ProofCheck::EnvelopeRouting, || {
        if bytes.len() > 256 * 1024 {
            return Err(TvcProvenanceError::MalformedEvidence(
                "inner proof exceeds 256 KiB".into(),
            ));
        }
        // Unauthenticated routing only excludes policy envelopes. It cannot produce a claim.
        let routing: Routing = serde_json::from_str(bytes)?;
        Ok(routing.kind == "APP_PROOF_TYPE_POLICY_OUTCOME")
    })
}

impl From<serde_json::Error> for TvcProvenanceError {
    fn from(error: serde_json::Error) -> Self {
        Self::MalformedEvidence(format!(
            "invalid {:?} claim JSON at line {}, column {}",
            error.classify(),
            error.line(),
            error.column(),
        ))
    }
}

fn validate_timestamp(value: &str) -> Result<(), TvcProvenanceError> {
    if value.is_empty()
        || !value.bytes().all(|c| c.is_ascii_digit())
        || value.parse::<u64>().is_err()
    {
        return Err(TvcProvenanceError::MalformedEvidence(
            "timestampMs must be a decimal u64 string".into(),
        ));
    }
    Ok(())
}
impl TryFrom<ContextWire<u32>> for Context {
    type Error = TvcProvenanceError;

    fn try_from(wire: ContextWire<u32>) -> Result<Self, Self::Error> {
        let ContextWire {
            version: V1,
            verified_signature_digests,
        } = wire.into_v1()?;
        if verified_signature_digests.is_empty()
            || verified_signature_digests.len() > 1024
            || verified_signature_digests.iter().any(|d| {
                d.len() != 64
                    || !d
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            })
        {
            return Err(TvcProvenanceError::MalformedEvidence(
                "expected 1 through 1024 lowercase 32-byte signature digests".into(),
            ));
        }
        Ok(Context {
            digests: verified_signature_digests
                .into_iter()
                .map(|digest| {
                    RequestFingerprint::try_from(digest)
                        .map_err(|error| TvcProvenanceError::MalformedEvidence(error.into()))
                })
                .collect::<Result<_, _>>()?,
        })
    }
}
impl TryFrom<AuthenticatedPayload> for Claim {
    type Error = TvcProvenanceError;
    fn try_from(payload: AuthenticatedPayload) -> Result<Self, Self::Error> {
        match serde_json::from_str::<ClaimWire>(&payload.into_inner())? {
            ClaimWire::Operator {
                timestamp_ms,
                request_context,
                body,
            } => {
                validate_timestamp(&timestamp_ms)?;
                if [
                    &body.organization_id,
                    &body.wallet_id,
                    &body.derivation_path,
                    &body.address,
                ]
                .iter()
                .any(|v| v.is_empty())
                {
                    return Err(TvcProvenanceError::MalformedEvidence(
                        "operator body fields must be nonempty".into(),
                    ));
                }
                let address = body.address.parse()?;
                Ok(Claim::Operator(OperatorClaim {
                    context: request_context.try_into()?,
                    body: OperatorBody {
                        organization_id: body.organization_id,
                        wallet_id: body.wallet_id,
                        derivation_path: body.derivation_path,
                        address,
                    },
                }))
            }
            ClaimWire::Quorum {
                timestamp_ms,
                request_context,
                body,
            } => {
                validate_timestamp(&timestamp_ms)?;
                if body.organization_id.is_empty()
                    || body.quorum_key_id.is_empty()
                    || !(2..=255).contains(&body.threshold)
                    || body.operator_encrypt_keys.len() < body.threshold as usize
                    || body.operator_encrypt_keys.len() > 255
                {
                    return Err(TvcProvenanceError::MalformedEvidence(
                        "invalid quorum fields or cardinality".into(),
                    ));
                }
                let quorum_public_key = body.quorum_public_key.parse().map_err(|error| {
                    TvcProvenanceError::MalformedEvidence(format!("invalid composite key: {error}"))
                })?;
                let operator_encrypt_keys = body
                    .operator_encrypt_keys
                    .iter()
                    .map(|key| key.parse())
                    .collect::<Result<Vec<_>, _>>()?;
                if operator_encrypt_keys
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != operator_encrypt_keys.len()
                {
                    return Err(TvcProvenanceError::MalformedEvidence(
                        "duplicate quorum recipient".into(),
                    ));
                }
                Ok(Claim::Quorum(QuorumClaim {
                    context: request_context.try_into()?,
                    body: QuorumBody {
                        organization_id: body.organization_id,
                        quorum_key_id: body.quorum_key_id,
                        quorum_public_key,
                        threshold: body.threshold,
                        operator_encrypt_keys,
                    },
                }))
            }
        }
    }
}
