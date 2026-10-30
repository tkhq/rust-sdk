//! Types for parsing and accessing already trusted ES256 verification keys.

use std::fmt;

use p256::ecdsa::VerifyingKey;
use serde::Deserialize;

use crate::ProtectedHeader;
use crate::b64;

const P256_CRV: &str = "P-256";
const SIG_USE: &str = "sig";

/// A trusted ES256 public key, identified by its RFC 7638 thumbprint.
#[derive(Clone)]
pub struct VerificationKey {
    kid: String,
    key: VerifyingKey,
}

impl VerificationKey {
    /// A key from the JWK `x` and `y` members (unpadded base64url), with its
    /// `kid` set to the RFC 7638 thumbprint.
    ///
    /// # Errors
    ///
    /// Returns an error when `x` or `y` is not 32-byte base64url or the point
    /// is not on P-256.
    pub fn from_p256_xy(x: &str, y: &str) -> Result<Self, JwksError> {
        let (Some(x_bytes), Some(y_bytes)) = (
            b64::url_decode(x).filter(|v| v.len() == 32),
            b64::url_decode(y).filter(|v| v.len() == 32),
        ) else {
            return Err(JwksError("x and y must be 32-byte base64url".into()));
        };
        let mut point = Vec::with_capacity(65);
        point.push(0x04);
        point.extend_from_slice(&x_bytes);
        point.extend_from_slice(&y_bytes);
        let key = VerifyingKey::from_sec1_bytes(&point)
            .map_err(|_| JwksError("point is not on P-256".into()))?;
        Ok(Self {
            kid: thumbprint_p256(x, y),
            key,
        })
    }

    /// The RFC 7638 thumbprint of the key.
    #[must_use]
    pub fn kid(&self) -> &str {
        &self.kid
    }

    pub(crate) fn verifying_key(&self) -> &VerifyingKey {
        &self.key
    }
}

/// A source of trusted keys.
///
/// When the header has a `kid`, the returned key's `kid` must equal it.
pub trait KeySource: Send + Sync {
    /// The trusted key that `header` selects, if any.
    fn key(&self, header: &ProtectedHeader) -> Option<VerificationKey>;
}

/// A single key the caller already trusts, for example the `cnf` key that a
/// Key Binding JWT must be signed with. A header `kid`, when present, must
/// still equal the key's thumbprint.
impl KeySource for VerificationKey {
    fn key(&self, _header: &ProtectedHeader) -> Option<VerificationKey> {
        Some(self.clone())
    }
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
    let canonical = format!(r#"{{"crv":"{P256_CRV}","kty":"EC","x":"{x}","y":"{y}"}}"#);
    b64::sha256_url(canonical.as_bytes())
}

/// A fixed set of keys loaded from configuration.
pub struct PinnedKeySet {
    keys: Vec<VerificationKey>,
}

impl PinnedKeySet {
    /// Parse a JWK Set of public ES256 keys. A `kid`, when present, must equal
    /// the RFC 7638 thumbprint.
    ///
    /// # Errors
    ///
    /// Returns an error when the set is empty or malformed, or a key is not a
    /// valid public ES256 key.
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
            if jwk.kty != "EC" || jwk.crv != P256_CRV {
                return Err(JwksError("only EC P-256 keys are accepted".into()));
            }
            if jwk.alg.as_deref().is_some_and(|a| a != crate::ES256) {
                return Err(JwksError("alg must be ES256 when present".into()));
            }
            if jwk.use_.as_deref().is_some_and(|u| u != SIG_USE) {
                return Err(JwksError("use must be sig when present".into()));
            }
            let key = VerificationKey::from_p256_xy(&jwk.x, &jwk.y)?;
            if jwk.kid.as_deref().is_some_and(|k| k != key.kid()) {
                return Err(JwksError(format!(
                    "kid is not the RFC 7638 thumbprint ({})",
                    key.kid()
                )));
            }
            keys.push(key);
        }
        Ok(Self { keys })
    }

    /// The key with this `kid`, if any.
    #[must_use]
    pub fn get(&self, kid: &str) -> Option<&VerificationKey> {
        self.keys.iter().find(|k| k.kid() == kid)
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
