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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn tkms_signature_digest_vectors() {
        let vectors: Vec<Value> = serde_json::from_str(include_str!(
            "../../tests/fixtures/signature_digest_vectors.json"
        ))
        .unwrap();
        for scheme in [1, 4] {
            assert!(vectors.iter().any(|vector| vector["scheme"] == scheme));
        }
        for vector in vectors {
            let scheme = match vector["scheme"].as_i64().unwrap() {
                1 => SignatureScheme::TkApiP256,
                4 => SignatureScheme::TkApiSecp256k1,
                other => panic!("unexpected vector signature scheme {other}"),
            };
            let public_key = vector["publicKey"].as_str().unwrap();
            let message = vector["message"].as_str().unwrap();
            let signature = vector["signature"].as_str().unwrap();
            let actual = digest(scheme, public_key, message, signature).unwrap();
            assert_eq!(
                hex::encode(actual),
                vector["digest"].as_str().unwrap(),
                "{}",
                vector["name"]
            );
        }
    }
}
