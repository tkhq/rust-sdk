//! Shared quorum key metadata JSON format.

use anyhow::{Context, anyhow};
use qos_p256::P256Public;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QuorumKeyMetadata {
    pub(crate) quorum_key_public: String,
    pub(crate) threshold: u32,
    pub(crate) shares: Vec<EncryptedShareMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncryptedShareMetadata {
    pub(crate) operator_public_key: String,
    pub(crate) share: String,
}

/// A disagreement between local quorum key metadata and the manifest a share
/// is about to be handed over against.
///
/// The metadata records the quorum key and the Shamir parameters it was
/// really split with; the manifest's share set is what the enclave will
/// enforce. The two come from independent inputs, so each difference is
/// refused rather than reconciled: a manifest that overstates the threshold
/// lets fewer custodians than advertised reconstruct the key, and one that
/// understates it has the enclave attempt reconstruction before enough real
/// shares exist. Operators must be distinct on both sides, because the enclave
/// counts each provisioned share toward the threshold without deduplicating
/// by member: one operator named twice can meet a threshold of two alone.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ManifestMismatch {
    #[error(
        "quorum key metadata quorumKeyPublic ({metadata}) does not match provision bundle manifest quorumKey ({manifest})"
    )]
    QuorumKey { metadata: String, manifest: String },
    #[error(
        "quorum key metadata threshold ({metadata}) does not match provision bundle manifest share set threshold ({manifest})"
    )]
    Threshold { metadata: u32, manifest: u32 },
    #[error(
        "quorum key metadata holds {metadata} shares but provision bundle manifest share set has {manifest} members"
    )]
    ShareCount { metadata: usize, manifest: usize },
    #[error(
        "quorum key metadata holds a share for operator {operator_public_key}, which is not a provision bundle manifest share set member"
    )]
    ShareWithoutMember { operator_public_key: String },
    #[error("quorum key metadata holds more than one share for operator {operator_public_key}")]
    DuplicateShare { operator_public_key: String },
    #[error(
        "provision bundle manifest share set names operator {operator_public_key} more than once"
    )]
    DuplicateMember { operator_public_key: String },
}

impl QuorumKeyMetadata {
    pub(crate) fn quorum_key_public_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(decode_p256_public_key_hex(&self.quorum_key_public)
            .context("invalid quorumKeyPublic in quorum key metadata")?
            .to_bytes())
    }
}

pub(crate) fn decode_p256_public_key_hex(public_key_hex: &str) -> anyhow::Result<P256Public> {
    let bytes =
        hex::decode(public_key_hex.trim()).context("public key must be bare hex encoded")?;

    P256Public::from_bytes(&bytes).map_err(|e| anyhow!("invalid QOS P-256 key: {e:?}"))
}

pub(crate) fn normalize_p256_public_key_hex(public_key_hex: &str) -> anyhow::Result<String> {
    Ok(hex::encode(
        decode_p256_public_key_hex(public_key_hex)?.to_bytes(),
    ))
}
