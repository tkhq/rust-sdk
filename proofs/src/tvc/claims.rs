//! Strict flattened producer-wire decoding. Generated protobuf serde has a
//! different oneof representation and is intentionally not used as authority.
use super::{Error, RolePublicKey};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperatorWire {
    #[serde(rename = "type")]
    kind: String,
    timestamp_ms: String,
    request_context: ContextWire,
    address_derivation_proof: OperatorBody,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QuorumWire {
    #[serde(rename = "type")]
    kind: String,
    timestamp_ms: String,
    request_context: ContextWire,
    tvc_quorum_key_create_proof: QuorumBody,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContextWire {
    version: u32,
    verified_signature_digests: Vec<String>,
}

pub(super) struct Context {
    pub(super) digests: Vec<String>,
}
pub(super) struct OperatorClaim {
    pub(super) context: Context,
    pub(super) body: OperatorBody,
}
pub(super) struct QuorumClaim {
    pub(super) context: Context,
    pub(super) body: QuorumBody,
}
pub(super) enum Claim {
    Operator(OperatorClaim),
    Quorum(QuorumClaim),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct OperatorBody {
    pub(super) organization_id: String,
    pub(super) wallet_id: String,
    pub(super) derivation_path: String,
    pub(super) address: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct QuorumBody {
    pub(super) organization_id: String,
    pub(super) quorum_key_id: String,
    pub(super) quorum_public_key: String,
    pub(super) threshold: u32,
    pub(super) operator_encrypt_keys: Vec<String>,
}

#[derive(Deserialize)]
struct Routing {
    #[serde(rename = "type")]
    kind: String,
}
fn routing(bytes: &str) -> Result<String, Error> {
    serde_json::from_str::<Routing>(bytes)
        .map(|r| r.kind)
        .map_err(malformed)
}
pub(super) fn is_policy(bytes: &str) -> Result<bool, Error> {
    Ok(routing(bytes)? == "APP_PROOF_TYPE_POLICY_OUTCOME")
}
fn malformed(error: serde_json::Error) -> Error {
    Error::MalformedEvidence(format!(
        "invalid {:?} claim JSON at line {}, column {}",
        error.classify(),
        error.line(),
        error.column(),
    ))
}
fn timestamp(value: &str) -> Result<(), Error> {
    if value.is_empty()
        || !value.bytes().all(|c| c.is_ascii_digit())
        || value.parse::<u64>().is_err()
    {
        return Err(Error::MalformedEvidence(
            "timestampMs must be a decimal u64 string".into(),
        ));
    }
    Ok(())
}
fn context(wire: ContextWire) -> Result<Context, Error> {
    if wire.version != 1 {
        return Err(Error::UnsupportedContext);
    }
    if wire.verified_signature_digests.is_empty()
        || wire.verified_signature_digests.len() > 1024
        || wire.verified_signature_digests.iter().any(|d| {
            d.len() != 64
                || !d
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        })
    {
        return Err(Error::MalformedEvidence(
            "expected 1 through 1024 lowercase 32-byte signature digests".into(),
        ));
    }
    Ok(Context {
        digests: wire.verified_signature_digests,
    })
}
pub(super) fn composite(value: &str) -> Result<[u8; 130], Error> {
    if value.len() != 260 || !value.is_ascii() {
        return Err(Error::MalformedEvidence(
            "composite key must contain two 65-byte points".into(),
        ));
    }
    RolePublicKey::parse(&value[..130])?;
    RolePublicKey::parse(&value[130..])?;
    hex::decode(value)
        .map_err(|_| Error::MalformedEvidence("invalid composite key".into()))?
        .try_into()
        .map_err(|_| Error::MalformedEvidence("invalid composite length".into()))
}
pub(super) fn decode(bytes: &str) -> Result<Claim, Error> {
    match routing(bytes)?.as_str() {
        "APP_PROOF_TYPE_ADDRESS_DERIVATION" => {
            let wire: OperatorWire = serde_json::from_str(bytes).map_err(malformed)?;
            if wire.kind != "APP_PROOF_TYPE_ADDRESS_DERIVATION" {
                return Err(Error::ClaimMismatch("operator type"));
            }
            timestamp(&wire.timestamp_ms)?;
            let body = wire.address_derivation_proof;
            if [
                &body.organization_id,
                &body.wallet_id,
                &body.derivation_path,
                &body.address,
            ]
            .iter()
            .any(|v| v.is_empty())
            {
                return Err(Error::MalformedEvidence(
                    "operator body fields must be nonempty".into(),
                ));
            }
            RolePublicKey::parse(&body.address)?;
            Ok(Claim::Operator(OperatorClaim {
                context: context(wire.request_context)?,
                body,
            }))
        }
        "APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE" => {
            let wire: QuorumWire = serde_json::from_str(bytes).map_err(malformed)?;
            if wire.kind != "APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE" {
                return Err(Error::ClaimMismatch("quorum type"));
            }
            timestamp(&wire.timestamp_ms)?;
            let body = wire.tvc_quorum_key_create_proof;
            if body.organization_id.is_empty()
                || body.quorum_key_id.is_empty()
                || !(2..=255).contains(&body.threshold)
                || body.operator_encrypt_keys.len() < body.threshold as usize
                || body.operator_encrypt_keys.len() > 255
            {
                return Err(Error::MalformedEvidence(
                    "invalid quorum fields or cardinality".into(),
                ));
            }
            composite(&body.quorum_public_key)?;
            let mut seen = std::collections::HashSet::new();
            for recipient in &body.operator_encrypt_keys {
                if !seen.insert(RolePublicKey::parse(recipient)?) {
                    return Err(Error::MalformedEvidence(
                        "duplicate quorum recipient".into(),
                    ));
                }
            }
            Ok(Claim::Quorum(QuorumClaim {
                context: context(wire.request_context)?,
                body,
            }))
        }
        _ => Err(Error::MalformedEvidence("unsupported proof type".into())),
    }
}
