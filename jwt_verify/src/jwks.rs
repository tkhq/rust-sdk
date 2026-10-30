//! Trusted ES256 verification keys.
//!
//! [`crate::verify`] gets keys only through the [`KeySource`] trait. A
//! source sees the verified-shape protected header and returns a key it
//! already trusts. [`PinnedKeySet`] selects by `kid`, the unpadded base64url
//! RFC 7638 thumbprint, from a JWK Set that the application pins in its
//! configuration. `kid` selects among trusted keys; it is never a trust root.
//!
//! No source in this crate derives a key location from `iss`, `jku`, `x5u`,
//! or any other token value.

use std::fmt;

use serde::Deserialize;

use crate::ProtectedHeader;
use crate::b64;

/// A trusted ES256 public key.
#[derive(Clone)]
pub struct VerificationKey {
    /// RFC 7638 thumbprint of the key.
    pub kid: String,
    /// SEC1 uncompressed point, `04 || x || y`.
    pub point: [u8; 65],
}

/// A source of trusted keys.
///
/// [`crate::verify`] calls [`KeySource::key`] after the header passes the
/// `alg` and `typ` checks and before the signature check. When the header has
/// a `kid`, the returned key's `kid` must equal it.
pub trait KeySource: Send + Sync {
    /// The trusted key that `header` selects, if any.
    fn key(&self, header: &ProtectedHeader) -> Option<VerificationKey>;
}

/// A JWK Set parse or validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwksError(pub String);

impl fmt::Display for JwksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid trusted JWK set: {}", self.0)
    }
}

impl std::error::Error for JwksError {}

#[derive(Deserialize)]
struct JwkSet {
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    kty: String,
    crv: String,
    x: String,
    y: String,
    #[serde(default, deserialize_with = "reject_null")]
    kid: Option<String>,
    #[serde(default, deserialize_with = "reject_null")]
    alg: Option<String>,
    #[serde(rename = "use", default, deserialize_with = "reject_null")]
    use_: Option<String>,
    #[serde(default, deserialize_with = "reject_null")]
    d: Option<serde::de::IgnoredAny>,
}

/// Deserialize a value that must not be JSON null. Used with `Option` fields so
/// an explicit `null` is an error. A missing field still uses `#[serde(default)]`.
fn reject_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

/// RFC 7638 thumbprint of a P-256 JWK: SHA-256 over the required members in
/// lexicographic order, with no whitespace, as unpadded base64url.
#[must_use]
pub fn thumbprint_p256(x: &str, y: &str) -> String {
    let canonical = format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#);
    b64::sha256_url(canonical.as_bytes())
}

/// A fixed set of keys loaded from configuration.
pub struct PinnedKeySet {
    keys: Vec<VerificationKey>,
}

impl PinnedKeySet {
    /// Parse a JWK Set. Every key must be a public P-256 key. A `kid`, when
    /// present, must equal the RFC 7638 thumbprint.
    ///
    /// # Errors
    ///
    /// Returns an error for a malformed set, an empty set, a private key, a
    /// non-P-256 key, an `alg` other than ES256, a `use` other than sig, an
    /// `x` or `y` that is not 32-byte base64url, a point that is not on the
    /// curve, or a `kid` that is not the thumbprint.
    pub fn from_jwks_json(bytes: &[u8]) -> Result<Self, JwksError> {
        let set: JwkSet =
            serde_json::from_slice(bytes).map_err(|e| JwksError(format!("parse: {e}")))?;
        if set.keys.is_empty() {
            return Err(JwksError("no keys".into()));
        }
        let mut keys = Vec::new();
        for jwk in set.keys {
            if jwk.d.is_some() {
                return Err(JwksError("the set contains a private key".into()));
            }
            if jwk.kty != "EC" || jwk.crv != "P-256" {
                return Err(JwksError("only EC P-256 keys are accepted".into()));
            }
            if jwk.alg.as_deref().is_some_and(|a| a != crate::ES256) {
                return Err(JwksError("alg must be ES256 when present".into()));
            }
            if jwk.use_.as_deref().is_some_and(|u| u != "sig") {
                return Err(JwksError("use must be sig when present".into()));
            }
            let (Some(x), Some(y)) = (
                b64::url_decode(&jwk.x).filter(|v| v.len() == 32),
                b64::url_decode(&jwk.y).filter(|v| v.len() == 32),
            ) else {
                return Err(JwksError("x and y must be 32-byte base64url".into()));
            };
            let mut point = [0u8; 65];
            point[0] = 0x04;
            point[1..33].copy_from_slice(&x);
            point[33..].copy_from_slice(&y);
            if p256::PublicKey::from_sec1_bytes(&point).is_err() {
                return Err(JwksError("point is not on P-256".into()));
            }
            let kid = thumbprint_p256(&jwk.x, &jwk.y);
            if jwk.kid.as_deref().is_some_and(|k| k != kid) {
                return Err(JwksError(format!(
                    "kid is not the RFC 7638 thumbprint ({kid})"
                )));
            }
            keys.push(VerificationKey { kid, point });
        }
        Ok(Self { keys })
    }

    /// The key with this `kid`, if any.
    #[must_use]
    pub fn get(&self, kid: &str) -> Option<&VerificationKey> {
        self.keys.iter().find(|k| k.kid == kid)
    }
}

impl KeySource for PinnedKeySet {
    fn key(&self, header: &ProtectedHeader) -> Option<VerificationKey> {
        header.kid().and_then(|kid| self.get(kid)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const X: &str = "f83OJ3D2xF1Bg8vub9tLe1gHMzV76e8Tus9uPHvRVEU";
    const Y: &str = "x_FEzRu9m36HLN_tue659LNpXW6pCyStikYjKIWI5a0";

    #[test]
    fn thumbprint_matches_rfc7638_for_p256_key() {
        assert_eq!(
            thumbprint_p256(X, Y),
            "oKIywvGUpTVTyxMQ3bwIIeQUudfr_CkLMjCE19ECD-U"
        );
    }

    #[test]
    fn rejects_private_key_and_wrong_kid() {
        let private =
            format!(r#"{{"keys":[{{"kty":"EC","crv":"P-256","x":"{X}","y":"{Y}","d":"AA"}}]}}"#);
        assert!(PinnedKeySet::from_jwks_json(private.as_bytes()).is_err());
        let wrong_kid = format!(
            r#"{{"keys":[{{"kty":"EC","crv":"P-256","x":"{X}","y":"{Y}","kid":"nope"}}]}}"#
        );
        assert!(PinnedKeySet::from_jwks_json(wrong_kid.as_bytes()).is_err());
        let ok = format!(r#"{{"keys":[{{"kty":"EC","crv":"P-256","x":"{X}","y":"{Y}"}}]}}"#);
        let set = PinnedKeySet::from_jwks_json(ok.as_bytes()).unwrap();
        assert!(
            set.get("oKIywvGUpTVTyxMQ3bwIIeQUudfr_CkLMjCE19ECD-U")
                .is_some()
        );
    }

    #[test]
    fn rejects_off_curve_point() {
        let bad_y = b64::url_encode(&[1u8; 32]);
        let set = format!(r#"{{"keys":[{{"kty":"EC","crv":"P-256","x":"{X}","y":"{bad_y}"}}]}}"#);
        assert!(PinnedKeySet::from_jwks_json(set.as_bytes()).is_err());
    }
}
