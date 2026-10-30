//! Strict compact JWS parsing and ES256 verification.
//!
//! Nothing in the token is trusted until the signature is verified with a
//! key that a [`KeySource`] already trusts.

use std::collections::HashSet;
use std::fmt;

use p256::ecdsa::signature::Verifier as _;
use p256::ecdsa::{Signature, VerifyingKey};
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, DeserializeOwned, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::Error;
use crate::b64;
use crate::jwks::KeySource;

/// The only accepted `alg`.
pub const ES256: &str = "ES256";

/// What [`verify`] requires of a token beyond the fixed rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct JwsRules<'a> {
    /// The exact `typ` the header must carry.
    pub typ: &'a str,
    /// Largest accepted compact JWS, in bytes.
    pub max_token_bytes: usize,
}

impl<'a> JwsRules<'a> {
    /// Rules for tokens of media type `typ` and at most `max_token_bytes`.
    #[must_use]
    pub const fn new(typ: &'a str, max_token_bytes: usize) -> Self {
        Self {
            typ,
            max_token_bytes,
        }
    }
}

/// The protected header. The member set is closed: exactly `alg`, `kid`, and
/// `typ`. Unknown and repeated members are rejected.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtectedHeader {
    alg: String,
    kid: String,
    typ: String,
}

impl ProtectedHeader {
    /// The `alg` member.
    #[must_use]
    pub fn alg(&self) -> &str {
        &self.alg
    }

    /// The `typ` member.
    #[must_use]
    pub fn typ(&self) -> &str {
        &self.typ
    }

    /// The `kid` member. Always present in the accepted header set today.
    #[must_use]
    pub fn kid(&self) -> Option<&str> {
        Some(&self.kid)
    }
}

/// A compact JWS whose header and signature passed [`verify`].
#[derive(Debug, Clone)]
pub struct VerifiedJws {
    header: ProtectedHeader,
    payload: Vec<u8>,
}

impl VerifiedJws {
    /// The protected header.
    #[must_use]
    pub fn header(&self) -> &ProtectedHeader {
        &self.header
    }

    /// The decoded payload bytes.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Parse the payload as a claim set of type `T`.
    ///
    /// The payload must be a JSON object with no nulls and no repeated member
    /// names at any depth, and `T` must accept it. Use a struct with
    /// `#[serde(deny_unknown_fields)]` for a closed claim schema, and
    /// [`object`] on nested struct members so they too must be objects.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Malformed`] when the payload is not JSON, and
    /// [`Error::BadClaim`] when it is not an object without nulls or repeated
    /// names, or does not match `T`.
    pub fn claims<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let payload = &self.payload;
        if serde_json::from_slice::<StrictObject>(payload).is_err() {
            if serde_json::from_slice::<IgnoredAny>(payload).is_err() {
                return Err(Error::Malformed("payload"));
            }
            return Err(Error::BadClaim(
                "claims must be an object without nulls or repeated names",
            ));
        }
        from_object(payload)
            .map_err(|_| Error::BadClaim("claim set does not match the profile schema"))
    }
}

/// Deserialize `T` from a JSON object only.
///
/// Serde's derived `Deserialize` for a named struct also accepts a JSON
/// array of its fields in order. Use this with `#[serde(deserialize_with =
/// "object")]` on a struct member to require the object form.
///
/// # Errors
///
/// Fails when the input is not a map or `T` rejects it.
pub fn object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct ObjectVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a JSON object")
        }

        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_map(ObjectVisitor(std::marker::PhantomData))
}

fn from_object<T: DeserializeOwned>(bytes: &[u8]) -> serde_json::Result<T> {
    let mut de = serde_json::Deserializer::from_slice(bytes);
    let value = object(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Parse a compact JWS and verify its ES256 signature.
///
/// The checks run in this order: size, three-part shape, header encoding and
/// closed member set, `alg`, `typ`, key selection, signature, payload
/// encoding. Claims are not checked; see [`VerifiedJws::claims`] and
/// [`crate::check_times`].
///
/// # Errors
///
/// Returns the first failed check as an [`Error`].
pub fn verify(
    token: &str,
    rules: &JwsRules<'_>,
    keys: &dyn KeySource,
) -> Result<VerifiedJws, Error> {
    if token.len() > rules.max_token_bytes {
        return Err(Error::Malformed("token too large"));
    }
    let mut parts = token.split('.');
    let (Some(h64), Some(p64), Some(s64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(Error::Malformed("not a compact JWS"));
    };
    let header_bytes = b64::url_decode(h64).ok_or(Error::Malformed("header encoding"))?;
    let header: ProtectedHeader =
        from_object(&header_bytes).map_err(|_| Error::Malformed("header"))?;
    if header.alg != ES256 {
        return Err(Error::UnsupportedAlgorithm);
    }
    if header.typ != rules.typ {
        return Err(Error::WrongType);
    }
    let key = keys.key(&header).ok_or(Error::UnknownKey)?;
    if header.kid().is_some_and(|kid| kid != key.kid) {
        return Err(Error::UnknownKey);
    }
    let sig = b64::url_decode(s64).ok_or(Error::Malformed("signature encoding"))?;
    let signing_input = &token[..h64.len() + 1 + p64.len()];
    let verifying_key = VerifyingKey::from_sec1_bytes(&key.point).map_err(|_| Error::UnknownKey)?;
    let signature = Signature::from_slice(&sig).map_err(|_| Error::BadSignature)?;
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| Error::BadSignature)?;
    let payload = b64::url_decode(p64).ok_or(Error::Malformed("payload encoding"))?;
    Ok(VerifiedJws { header, payload })
}

/// A JSON object with no nulls and no repeated member names at any depth.
struct StrictObject;

impl<'de> Deserialize<'de> for StrictObject {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_map(Strict)
            .map(|Strict| StrictObject)
    }
}

/// Any JSON value with no nulls and no repeated member names at any depth.
struct Strict;

impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Strict)
    }
}

macro_rules! scalar_visits {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method<E: de::Error>(self, _: $ty) -> Result<Strict, E> {
                Ok(Strict)
            }
        )*
    };
}

impl<'de> Visitor<'de> for Strict {
    type Value = Strict;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value without nulls or repeated names")
    }

    scalar_visits! {
        visit_bool(bool),
        visit_i64(i64),
        visit_u64(u64),
        visit_f64(f64),
        visit_str(&str),
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Strict, A::Error> {
        while seq.next_element::<Strict>()?.is_some() {}
        Ok(Strict)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Strict, A::Error> {
        let mut names = HashSet::new();
        while let Some(name) = map.next_key::<String>()? {
            if !names.insert(name) {
                return Err(de::Error::custom("repeated member name"));
            }
            map.next_value::<Strict>()?;
        }
        Ok(Strict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jwks::{VerificationKey, thumbprint_p256};
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::signature::Signer;
    use rand_core::OsRng;
    use serde_json::json;

    const TYP: &str = "application/example+jwt";
    const RULES: JwsRules<'static> = JwsRules::new(TYP, 1024);

    struct Key {
        signing: SigningKey,
        public: VerificationKey,
    }

    impl Key {
        fn new() -> Self {
            let signing = SigningKey::random(&mut OsRng);
            let mut point = [0u8; 65];
            point.copy_from_slice(signing.verifying_key().to_encoded_point(false).as_bytes());
            let kid = thumbprint_p256(
                &b64::url_encode(&point[1..33]),
                &b64::url_encode(&point[33..]),
            );
            Self {
                signing,
                public: VerificationKey { kid, point },
            }
        }

        fn sign(&self, header: &str, payload: &str) -> String {
            let input = format!(
                "{}.{}",
                b64::url_encode(header.as_bytes()),
                b64::url_encode(payload.as_bytes())
            );
            let sig: Signature = self.signing.sign(input.as_bytes());
            format!("{input}.{}", b64::url_encode(&sig.to_bytes()))
        }

        fn header(&self) -> serde_json::Value {
            json!({"alg": ES256, "kid": self.public.kid, "typ": TYP})
        }
    }

    impl KeySource for Key {
        fn key(&self, header: &ProtectedHeader) -> Option<VerificationKey> {
            (header.kid() == Some(self.public.kid.as_str())).then(|| self.public.clone())
        }
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Claims {
        sub: String,
    }

    #[test]
    fn verifies_and_parses_a_closed_claim_set() {
        let k = Key::new();
        let jws = verify(
            &k.sign(&k.header().to_string(), r#"{"sub":"a"}"#),
            &RULES,
            &k,
        )
        .unwrap();
        assert_eq!(jws.header().typ(), TYP);
        assert_eq!(jws.claims::<Claims>().unwrap().sub, "a");
        let dup = verify(
            &k.sign(&k.header().to_string(), r#"{"sub":"a","sub":"b"}"#),
            &RULES,
            &k,
        )
        .unwrap();
        assert!(matches!(dup.claims::<Claims>(), Err(Error::BadClaim(_))));
        let null = verify(
            &k.sign(&k.header().to_string(), r#"{"sub":null}"#),
            &RULES,
            &k,
        )
        .unwrap();
        assert_eq!(
            null.claims::<Claims>().unwrap_err(),
            Error::BadClaim("claims must be an object without nulls or repeated names")
        );
    }

    #[test]
    fn claims_must_be_a_strict_object() {
        let k = Key::new();
        let claims = |payload: &str| {
            verify(&k.sign(&k.header().to_string(), payload), &RULES, &k)
                .unwrap()
                .claims::<serde_json::Value>()
        };
        let strict = Err(Error::BadClaim(
            "claims must be an object without nulls or repeated names",
        ));
        for payload in [
            r#"["a"]"#,
            "null",
            "1",
            r#""a""#,
            r#"{"a":1,"a":2}"#,
            r#"{"a":{"b":1,"b":1}}"#,
            r#"{"a":[null]}"#,
        ] {
            assert_eq!(claims(payload), strict, "{payload}");
        }
        assert_eq!(claims("{"), Err(Error::Malformed("payload")));
        assert_eq!(claims(r#"{"a":[1]}"#), Ok(json!({"a": [1]})));

        let positional = verify(&k.sign(&k.header().to_string(), r#"["a"]"#), &RULES, &k).unwrap();
        assert!(matches!(
            positional.claims::<Claims>(),
            Err(Error::BadClaim(_))
        ));

        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Outer {
            #[serde(deserialize_with = "object")]
            inner: Claims,
        }
        let nested = |payload: &str| {
            verify(&k.sign(&k.header().to_string(), payload), &RULES, &k)
                .unwrap()
                .claims::<Outer>()
        };
        assert_eq!(nested(r#"{"inner":{"sub":"a"}}"#).unwrap().inner.sub, "a");
        assert!(matches!(
            nested(r#"{"inner":["a"]}"#),
            Err(Error::BadClaim(_))
        ));
    }

    #[test]
    fn rejects_header_and_size_violations() {
        let k = Key::new();
        let e =
            |h: serde_json::Value| verify(&k.sign(&h.to_string(), "{}"), &RULES, &k).unwrap_err();
        let mut h = k.header();
        h["typ"] = json!("JWT");
        assert_eq!(e(h), Error::WrongType);
        let mut h = k.header();
        h["alg"] = json!("none");
        assert_eq!(e(h), Error::UnsupportedAlgorithm);
        let mut h = k.header();
        h["jwk"] = json!({});
        assert_eq!(e(h), Error::Malformed("header"));
        let positional = json!([ES256, k.public.kid, TYP]).to_string();
        assert_eq!(
            verify(&k.sign(&positional, "{}"), &RULES, &k).unwrap_err(),
            Error::Malformed("header")
        );
        let dup = format!(
            r#"{{"alg":"ES256","alg":"ES256","kid":"{}","typ":"{TYP}"}}"#,
            k.public.kid
        );
        assert_eq!(
            verify(&k.sign(&dup, "{}"), &RULES, &k).unwrap_err(),
            Error::Malformed("header")
        );
        let huge = "a".repeat(RULES.max_token_bytes + 1);
        assert_eq!(
            verify(&huge, &RULES, &k).unwrap_err(),
            Error::Malformed("token too large")
        );
        let other = Key::new();
        let forged = other.sign(&k.header().to_string(), "{}");
        assert_eq!(
            verify(&forged, &RULES, &k).unwrap_err(),
            Error::BadSignature
        );
    }
}
