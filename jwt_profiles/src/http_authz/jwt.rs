//! The http-authz/v1 claim set and profile rules.
//!
//! Nothing in the token is trusted until the signature is verified with a
//! key that the configuration already trusts.

use serde::{Deserialize, Deserializer};
use turnkey_jwt_verify::{JwsRules, KeySource, TimeRules, check_times, object};

use super::digest::is_digest;
use super::headers::{check_header_names, is_tchar_except_alpha};
use super::{AuthError, ConfigError};

pub(crate) const TYP: &str = "application/vnd.turnkey.http-authz-v1+jwt";
/// Profile maximum lifetime, `exp - iat`. A destination can configure a
/// shorter one ([`VerifierConfig::max_lifetime_secs`]), never a longer one.
pub const MAX_LIFETIME_SECS: u64 = 300;
/// Largest clock skew a destination may allow.
pub const MAX_CLOCK_SKEW_SECS: u64 = 30;
/// Default clock skew.
pub const DEFAULT_CLOCK_SKEW_SECS: u64 = 5;
/// Largest accepted compact JWS. It covers the minter's combined maximum:
/// audience, method, path, query, and 32 bound header names with their
/// digests.
pub(crate) const MAX_TOKEN_BYTES: usize = 12 * 1024;
pub(crate) const MAX_PATH_BYTES: usize = 2048;
pub(crate) const MAX_QUERY_BYTES: usize = 2048;
/// Prefix of `iss`, followed by the top-level organization ID.
pub const ISSUER_PREFIX: &str = "https://id.turnkey.com/t/";
/// URN prefix of an organization (`org`).
pub const ORG_URN_PREFIX: &str = "urn:turnkey:org:";
/// URN prefix of a user (`sub`).
pub const USER_URN_PREFIX: &str = "urn:turnkey:user:";

/// Verifier configuration.
#[derive(Debug, Clone)]
pub struct VerifierConfig {
    /// The app's exact origin, compared byte for byte with `aud`.
    pub audience: String,
    /// The app's top-level organization ID (without a prefix). A token's
    /// `iss` must equal `https://id.turnkey.com/t/<organization_id>`.
    pub organization_id: String,
    /// Allowed clock skew in seconds, at most [`MAX_CLOCK_SKEW_SECS`].
    pub clock_skew_secs: u64,
    /// Longest accepted `exp - iat`, at most [`MAX_LIFETIME_SECS`].
    pub max_lifetime_secs: u64,
}

impl VerifierConfig {
    /// A configuration with the profile defaults: skew
    /// [`DEFAULT_CLOCK_SKEW_SECS`] and lifetime [`MAX_LIFETIME_SECS`].
    pub fn new(audience: impl Into<String>, organization_id: impl Into<String>) -> Self {
        Self {
            audience: audience.into(),
            organization_id: organization_id.into(),
            clock_skew_secs: DEFAULT_CLOCK_SKEW_SECS,
            max_lifetime_secs: MAX_LIFETIME_SECS,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        if self.audience.is_empty() {
            return Err(ConfigError("audience must not be empty".into()));
        }
        if !printable_id(&self.organization_id, 128)
            || self.organization_id.starts_with("urn:")
            || self.organization_id.contains('/')
        {
            return Err(ConfigError(
                "organization_id must be a bare organization ID (1 to 128 visible ASCII characters, no prefix)".into(),
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
    /// Expiry, NumericDate. At most `iat + max_lifetime_secs` (see `VerifierConfig`).
    pub exp: u64,
    /// Issued-at, NumericDate (activity timestamp).
    pub iat: u64,
    /// `https://id.turnkey.com/t/<id>` of the top-level organization: the
    /// parent of the minting organization, or the minting organization itself.
    pub iss: String,
    /// `urn:turnkey:user:<id>`.
    pub sub: String,
    /// `urn:turnkey:org:<id>` of the organization that minted the token: the
    /// top-level organization or one of its sub-organizations.
    pub org: String,
    /// Request descriptor.
    #[serde(deserialize_with = "object")]
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

/// Check `iat` and `exp` at `now` with the configured skew and lifetime.
pub(crate) fn check_window(
    claims: &Claims,
    cfg: &VerifierConfig,
    now: u64,
) -> Result<(), AuthError> {
    check_times(
        claims.iat,
        claims.exp,
        now,
        &TimeRules::new(cfg.clock_skew_secs, cfg.max_lifetime_secs),
    )?;
    Ok(())
}

/// Verify a compact JWS against the profile, the configuration, and `now`.
///
/// This does not check the request binding. See [`super::bind_request`] and
/// [`super::Verifier`].
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
    let jws = turnkey_jwt_verify::verify(token, &JwsRules::new(TYP, MAX_TOKEN_BYTES), keys)?;
    let claims: Claims = jws.claims()?;

    if claims.aud != cfg.audience {
        return Err(AuthError::WrongAudience);
    }
    let issuer_id = urn_id(&claims.iss, ISSUER_PREFIX)
        .filter(|id| !id.contains('/'))
        .ok_or(AuthError::BadClaim("iss"))?;
    if issuer_id != cfg.organization_id {
        return Err(AuthError::WrongIssuer);
    }
    if urn_id(&claims.org, ORG_URN_PREFIX).is_none() {
        return Err(AuthError::BadClaim("org"));
    }
    if urn_id(&claims.sub, USER_URN_PREFIX).is_none() {
        return Err(AuthError::BadClaim("sub"));
    }

    check_window(&claims, cfg, now)?;

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
    // Both members absent, or both present as empty arrays (what the Turnkey
    // minter emits). See `RequestDescriptor::headers`.
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
