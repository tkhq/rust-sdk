//! RFC 9530 digests as Turnkey profiles use them.
//!
//! A digest is exactly one dictionary member, `sha-256=:<b64>:`, where
//! `<b64>` is standard base64 with padding (44 characters). No other
//! algorithm, no parameters, no whitespace, and no second member. A digest of
//! body bytes equals a valid `Content-Digest` value for them.
//!
//! The test vectors live in `testdata/digests.json`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha256};

const DIGEST_PREFIX: &str = "sha-256=:";
const DIGEST_SUFFIX: &str = ":";

/// The digest string for `bytes`.
#[must_use]
pub fn sha256_digest(bytes: &[u8]) -> String {
    format_digest(&Sha256::digest(bytes))
}

pub(crate) fn format_digest(d: &[u8]) -> String {
    format!("{DIGEST_PREFIX}{}{DIGEST_SUFFIX}", STANDARD.encode(d))
}

/// Whether `s` is a digest string in this format.
#[must_use]
pub fn is_digest(s: &str) -> bool {
    let Some(b64) = s
        .strip_prefix(DIGEST_PREFIX)
        .and_then(|r| r.strip_suffix(DIGEST_SUFFIX))
    else {
        return false;
    };
    // One byte more than a 32-byte digest. A longer decode fails, and the
    // length check rejects a shorter one.
    let mut buf = [0u8; 33];
    STANDARD.decode_slice(b64, &mut buf).is_ok_and(|n| n == 32)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) const VECTORS: &str = include_str!("../testdata/digests.json");

    #[test]
    fn sha256_digest_matches_shared_body_vectors() {
        let v: serde_json::Value = serde_json::from_str(VECTORS).unwrap();
        let cases = v["body"].as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let body = case["body"].as_str().unwrap();
            let want = case["digest"].as_str().unwrap();
            assert_eq!(sha256_digest(body.as_bytes()), want, "{name}");
            assert!(is_digest(want), "{name}");
        }
    }
}
