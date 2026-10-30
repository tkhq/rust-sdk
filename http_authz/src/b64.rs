//! Strict base64url helpers (RFC 7515 Section 2).

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// Unpadded base64url encoding (RFC 7515 Section 2).
#[must_use]
pub fn url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Strict unpadded base64url decoding. Rejects padding, non-alphabet
/// characters, and non-canonical trailing bits.
#[must_use]
pub fn url_decode(s: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(s).ok()
}

/// Unpadded base64url SHA-256 digest of `bytes`.
#[must_use]
pub fn sha256_url(bytes: &[u8]) -> String {
    url_encode(&Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_padding_and_bad_chars() {
        assert!(url_decode("YQ").is_some());
        assert!(url_decode("YQ==").is_none());
        assert!(url_decode("Y+Q").is_none());
        assert!(url_decode("YR").is_none());
    }
}
