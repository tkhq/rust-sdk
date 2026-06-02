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

impl QuorumKeyMetadata {
    pub(crate) fn quorum_key_public_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(decode_p256_public_key_hex(&self.quorum_key_public)
            .context("invalid quorumKeyPublic in quorum key metadata")?
            .to_bytes())
    }

    pub(crate) fn encrypted_share_for_operator(
        &self,
        operator_public_key: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let normalized_operator_public_key = hex::encode(operator_public_key);

        for (index, share) in self.shares.iter().enumerate() {
            let share_operator_public_key =
                normalize_p256_public_key_hex(&share.operator_public_key)
                    .with_context(|| format!("invalid operatorPublicKey at shares[{index}]"))?;

            if share_operator_public_key == normalized_operator_public_key {
                return hex::decode(share.share.trim())
                    .with_context(|| format!("invalid encrypted share at shares[{index}]"));
            }
        }

        anyhow::bail!(
            "operator ({normalized_operator_public_key}) not found in quorum key metadata shares"
        );
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
