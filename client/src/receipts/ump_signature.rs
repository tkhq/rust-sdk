//! Matches TKMS's exact signature encoding scheme for request-bound proofs.
//! Preserve the integer scheme and original field text so fingerprints match
//! the proof's authenticated request digests.

use sha2::{Digest as _, Sha256};

use super::invalid;
use crate::TurnkeyClientError;
use crate::generated::immutable::models::v1::SignatureScheme;

#[derive(borsh::BorshSerialize)]
struct UmpSignature<'a> {
    scheme: i32,
    public_key: &'a str,
    message: &'a str,
    signature: &'a str,
}

pub(super) fn digest(
    scheme: SignatureScheme,
    public_key: &str,
    message: &str,
    signature: &str,
) -> Result<[u8; 32], TurnkeyClientError> {
    let signature = UmpSignature {
        scheme: scheme as i32,
        public_key,
        message,
        signature,
    };
    let encoded = borsh::to_vec(&signature).map_err(|_| invalid("signature encoding failed"))?;
    Ok(Sha256::digest(encoded).into())
}
