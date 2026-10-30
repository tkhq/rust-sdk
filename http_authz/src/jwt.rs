//! Strict verifier for http-authz/v1 JWTs.
//!
//! Nothing in the token is trusted until the signature is verified with a
//! key that the configuration already trusts.

use std::fmt;

use p256::ecdsa::signature::Verifier as _;
use p256::ecdsa::{Signature, VerifyingKey};
use serde::de::{self, IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::b64;
use crate::digest::is_digest;
use crate::headers::{check_header_names, is_tchar_except_alpha};
use crate::jwks::KeySource;
use crate::{AuthError, ConfigError};

pub(crate) const TYP: &str = "application/vnd.turnkey.http-authz-v1+jwt";
const ALG: &str = "ES256";
/// Profile maximum lifetime, `exp - iat`. A destination can configure a
/// shorter one ([`VerifierConfig::max_lifetime_secs`]), never a longer one.
pub const MAX_LIFETIME_SECS: u64 = 300;
/// Largest clock skew a destination may allow.
pub const MAX_CLOCK_SKEW_SECS: u64 = 30;
/// Default clock skew.
pub const DEFAULT_CLOCK_SKEW_SECS: u64 = 5;
/// Largest accepted compact JWS. The profile allows a 6 KiB claim object,
/// which is 8 KiB in base64url, plus the header and signature.
pub(crate) const MAX_TOKEN_BYTES: usize = 9 * 1024;
pub(crate) const MAX_PATH_BYTES: usize = 2048;
const MAX_QUERY_BYTES: usize = 2048;
/// URN prefix of an organization (`iss`, `tk_parent_organization`).
pub const ORG_URN_PREFIX: &str = "urn:turnkey:org:";
/// URN prefix of a user (`sub`).
pub const USER_URN_PREFIX: &str = "urn:turnkey:user:";

/// Verifier configuration.
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// The app's exact origin, compared byte for byte with `aud`.
    pub audience: String,
    /// The app's parent organization ID (without the URN prefix). A token
    /// must carry `tk_parent_organization` equal to
    /// `urn:turnkey:org:<parent_organization_id>`.
    pub parent_organization_id: String,
    /// Allowed clock skew in seconds, at most [`MAX_CLOCK_SKEW_SECS`].
    pub clock_skew_secs: u64,
    /// Longest accepted `exp - iat`, at most [`MAX_LIFETIME_SECS`].
    pub max_lifetime_secs: u64,
}

impl VerifierConfig {
    /// A configuration with the profile defaults: skew
    /// [`DEFAULT_CLOCK_SKEW_SECS`] and lifetime [`MAX_LIFETIME_SECS`].
    pub fn new(audience: impl Into<String>, parent_organization_id: impl Into<String>) -> Self {
        Self {
            audience: audience.into(),
            parent_organization_id: parent_organization_id.into(),
            clock_skew_secs: DEFAULT_CLOCK_SKEW_SECS,
            max_lifetime_secs: MAX_LIFETIME_SECS,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.audience.is_empty() {
            return Err(ConfigError("audience must not be empty".into()));
        }
        if !printable_id(&self.parent_organization_id, 128)
            || self.parent_organization_id.starts_with("urn:")
        {
            return Err(ConfigError(
                "parent_organization_id must be a bare organization ID (1 to 128 visible ASCII characters, no urn: prefix)".into(),
            ));
        }
        if self.clock_skew_secs > MAX_CLOCK_SKEW_SECS {
            return Err(ConfigError(format!(
                "clock_skew_secs must be at most {MAX_CLOCK_SKEW_SECS}"
            )));
        }
        if self.max_lifetime_secs == 0 || self.max_lifetime_secs > MAX_LIFETIME_SECS {
            return Err(ConfigError(format!(
                "max_lifetime_secs must be 1 to {MAX_LIFETIME_SECS}"
            )));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    alg: String,
    kid: String,
    typ: String,
}

/// The `tk_request` claim.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDescriptor {
    /// Uppercase HTTP method token.
    pub method: String,
    /// Request path, compared byte for byte with the received path.
    pub path: String,
    /// Raw query without `?`, or empty, compared byte for byte with the
    /// received raw query.
    pub query: String,
    /// The bound header names: lowercase, unique, in the order of
    /// `tk_header_digest`. The pair with `tk_header_digest` has two
    /// encodings: both absent, or both present as empty arrays. Any other
    /// combination is a `BadClaim`.
    #[serde(default, deserialize_with = "non_null")]
    pub headers: Option<Vec<String>>,
}

/// The verified claim set.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claims {
    /// Exact audience origin.
    pub aud: String,
    /// Expiry, NumericDate (`iat + time_to_live_seconds`).
    pub exp: u64,
    /// Issued-at, NumericDate (activity timestamp).
    pub iat: u64,
    /// `urn:turnkey:org:<id>` of the issuing organization (a
    /// sub-organization).
    pub iss: String,
    /// `urn:turnkey:user:<id>`.
    pub sub: String,
    /// `urn:turnkey:org:<id>` of the issuing organization's parent. Required:
    /// a token without it is rejected with `ParentOrganizationMissing`.
    #[serde(default, deserialize_with = "non_null")]
    pub tk_parent_organization: Option<String>,
    /// Request descriptor.
    pub tk_request: RequestDescriptor,
    /// Digest of the body as received, if bound.
    #[serde(default, deserialize_with = "non_null")]
    pub tk_body_digest: Option<String>,
    /// One digest per name in `tk_request.headers`, in the same order. The
    /// pair with `tk_request.headers` has two encodings: both absent, or both
    /// present as empty arrays. Any other combination is a `BadClaim`.
    #[serde(default, deserialize_with = "non_null")]
    pub tk_header_digest: Option<Vec<String>>,
}

impl Claims {
    #[must_use]
    pub(crate) fn bound_headers(&self) -> &[String] {
        self.tk_request.headers.as_deref().unwrap_or_default()
    }
}

struct NoNulls;

impl<'de> Deserialize<'de> for NoNulls {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(NoNullsVisitor)
    }
}

struct NoNullsVisitor;

macro_rules! scalar_visits {
    ($($method:ident($ty:ty)),* $(,)?) => {
        $(
            fn $method<E: de::Error>(self, _: $ty) -> Result<NoNulls, E> {
                Ok(NoNulls)
            }
        )*
    };
}

impl<'de> Visitor<'de> for NoNullsVisitor {
    type Value = NoNulls;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON value without nulls")
    }

    scalar_visits! {
        visit_bool(bool),
        visit_i64(i64),
        visit_u64(u64),
        visit_f64(f64),
        visit_str(&str),
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<NoNulls, A::Error> {
        while seq.next_element::<NoNulls>()?.is_some() {}
        Ok(NoNulls)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<NoNulls, A::Error> {
        while map.next_key::<IgnoredAny>()?.is_some() {
            map.next_value::<NoNulls>()?;
        }
        Ok(NoNulls)
    }
}

fn non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

fn printable_id(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|b| (0x21..=0x7e).contains(&b))
}

fn urn_id<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if !printable_id(s, 256) {
        return None;
    }
    s.strip_prefix(prefix).filter(|id| !id.is_empty())
}

#[must_use]
fn is_method(m: &str) -> bool {
    !m.is_empty()
        && m.len() <= 32
        && m.bytes()
            .all(|b| b.is_ascii_uppercase() || is_tchar_except_alpha(b))
}

/// Verify a compact JWS against the profile, the configuration, and `now`.
///
/// This does not check the request binding. See [`crate::bind_request`] and
/// [`crate::Verifier`].
///
/// # Errors
///
/// Returns the first failed check as an [`AuthError`].
pub(crate) fn verify(
    token: &str,
    cfg: &VerifierConfig,
    keys: &dyn KeySource,
    now: u64,
) -> Result<Claims, AuthError> {
    if token.len() > MAX_TOKEN_BYTES {
        return Err(AuthError::Malformed("token too large"));
    }
    let mut parts = token.split('.');
    let (Some(h64), Some(p64), Some(s64), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(AuthError::Malformed("not a compact JWS"));
    };
    let header_bytes = b64::url_decode(h64).ok_or(AuthError::Malformed("header encoding"))?;
    let header: Header =
        serde_json::from_slice(&header_bytes).map_err(|_| AuthError::Malformed("header"))?;
    if header.alg != ALG {
        return Err(AuthError::UnsupportedAlgorithm);
    }
    if header.typ != TYP {
        return Err(AuthError::WrongType);
    }
    let key = keys.key(&header.kid).ok_or(AuthError::UnknownKey)?;
    if key.kid != header.kid {
        return Err(AuthError::UnknownKey);
    }
    let sig = b64::url_decode(s64).ok_or(AuthError::Malformed("signature encoding"))?;
    let signing_input_len = h64.len() + 1 + p64.len();
    let signing_input = &token[..signing_input_len];
    let verifying_key =
        VerifyingKey::from_sec1_bytes(&key.point).map_err(|_| AuthError::UnknownKey)?;
    let signature = Signature::from_slice(&sig).map_err(|_| AuthError::BadSignature)?;
    verifying_key
        .verify(signing_input.as_bytes(), &signature)
        .map_err(|_| AuthError::BadSignature)?;

    let payload = b64::url_decode(p64).ok_or(AuthError::Malformed("payload encoding"))?;
    let claims: Claims = match serde_json::from_slice(&payload) {
        Ok(claims) => claims,
        Err(_) => {
            let nonulls = serde_json::from_slice::<NoNulls>(&payload);
            if nonulls.is_err() && serde_json::from_slice::<IgnoredAny>(&payload).is_err() {
                return Err(AuthError::Malformed("payload"));
            }
            let is_object = payload.trim_ascii_start().first() == Some(&b'{');
            if !is_object || nonulls.is_err() {
                return Err(AuthError::BadClaim(
                    "claims must be an object without nulls",
                ));
            }
            return Err(AuthError::BadClaim(
                "claim set does not match the profile schema",
            ));
        }
    };

    if claims.aud != cfg.audience {
        return Err(AuthError::WrongAudience);
    }
    if urn_id(&claims.iss, ORG_URN_PREFIX).is_none() {
        return Err(AuthError::BadClaim("iss"));
    }
    if urn_id(&claims.sub, USER_URN_PREFIX).is_none() {
        return Err(AuthError::BadClaim("sub"));
    }
    let parent = claims
        .tk_parent_organization
        .as_deref()
        .ok_or(AuthError::ParentOrganizationMissing)?;
    let parent_id =
        urn_id(parent, ORG_URN_PREFIX).ok_or(AuthError::BadClaim("tk_parent_organization"))?;
    if parent_id != cfg.parent_organization_id {
        return Err(AuthError::WrongParentOrganization);
    }

    if claims.exp <= claims.iat {
        return Err(AuthError::BadClaim("exp must be after iat"));
    }
    if claims.exp - claims.iat > cfg.max_lifetime_secs {
        return Err(AuthError::LifetimeTooLong);
    }
    if claims.iat > now.saturating_add(cfg.clock_skew_secs) {
        return Err(AuthError::NotYetValid);
    }
    if now >= claims.exp {
        return Err(AuthError::Expired);
    }

    let r = &claims.tk_request;
    if !is_method(&r.method) {
        return Err(AuthError::BadClaim("tk_request.method"));
    }
    if r.path.len() > MAX_PATH_BYTES {
        return Err(AuthError::BadClaim("tk_request.path"));
    }
    if r.query.len() > MAX_QUERY_BYTES {
        return Err(AuthError::BadClaim("tk_request.query"));
    }
    if check_header_names(claims.bound_headers()).is_err() {
        return Err(AuthError::BadClaim("tk_request.headers"));
    }
    // No bound headers has two encodings: both members absent, or both
    // present as empty arrays (what the Turnkey minter emits). Anything else
    // must pair each name with one digest.
    match (r.headers.as_deref(), claims.tk_header_digest.as_deref()) {
        (None, None) => {}
        (Some(names), Some(d)) if d.len() == names.len() => {
            if !d.iter().all(|x| is_digest(x)) {
                return Err(AuthError::BadClaim("tk_header_digest"));
            }
        }
        _ => {
            return Err(AuthError::BadClaim(
                "tk_header_digest must have one entry per name in tk_request.headers",
            ));
        }
    }
    if claims
        .tk_body_digest
        .as_deref()
        .is_some_and(|d| !is_digest(d))
    {
        return Err(AuthError::BadClaim("tk_body_digest"));
    }
    Ok(claims)
}
