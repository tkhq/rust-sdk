//! The `turnkey.io/http-authz/v1` profile (feature `http-authz`): verify
//! request-bound JWTs minted with `ACTIVITY_TYPE_MINT_JWT` and bind them to
//! the received HTTP request. See the crate documentation for the checks.

pub mod digest;
pub mod headers;
pub mod jwt;
#[cfg(feature = "axum")]
pub mod middleware;

use std::fmt;
use std::sync::Arc;

use http::HeaderMap;

use digest::sha256_digest;
use headers::HeaderError;
pub use jwt::{Claims, VerifierConfig};
pub use turnkey_jwt_verify::{KeySource, b64, jwks, unix_now};

/// What one protected route requires beyond the profile checks.
#[derive(Debug, Clone, Default)]
pub struct RouteRule {
    /// Header names that the token must bind (lowercase). The token can bind
    /// more.
    pub required_headers: Vec<String>,
    /// Require `tk_body_digest`, even for an empty body.
    pub require_body_digest: bool,
}

/// A verification failure. Every variant maps to HTTP 401.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// No usable `Authorization: Bearer` header.
    MissingAuthorization,
    /// The token is not a well-formed compact JWS.
    Malformed(&'static str),
    /// `alg` is not the pinned `ES256`.
    UnsupportedAlgorithm,
    /// `typ` is not the profile media type.
    WrongType,
    /// `kid` does not select a trusted key.
    UnknownKey,
    /// The signature does not verify.
    BadSignature,
    /// `aud` is not the configured origin.
    WrongAudience,
    /// `iss` is not the configured top-level organization.
    WrongIssuer,
    /// `exp - iat` exceeds the configured maximum.
    LifetimeTooLong,
    /// `iat` is in the future beyond skew.
    NotYetValid,
    /// The token has expired.
    Expired,
    /// A claim is missing, has the wrong type, or has a bad value.
    BadClaim(&'static str),
    /// The request does not match the descriptor.
    RequestMismatch(&'static str),
    /// The route requires a body digest and the token has none.
    BodyDigestRequired,
    /// The route requires this header to be bound and the token does not
    /// bind it.
    HeaderBindingRequired(String),
}

impl AuthError {
    /// A stable machine-readable code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingAuthorization => "missing_authorization",
            Self::Malformed(_) => "malformed_token",
            Self::UnsupportedAlgorithm => "unsupported_alg",
            Self::WrongType => "wrong_typ",
            Self::UnknownKey => "unknown_kid",
            Self::BadSignature => "bad_signature",
            Self::WrongAudience => "wrong_aud",
            Self::WrongIssuer => "wrong_iss",
            Self::LifetimeTooLong => "lifetime_too_long",
            Self::NotYetValid => "not_yet_valid",
            Self::Expired => "expired",
            Self::BadClaim(_) => "bad_claim",
            Self::RequestMismatch(_) => "request_mismatch",
            Self::BodyDigestRequired => "body_digest_required",
            Self::HeaderBindingRequired(_) => "header_binding_required",
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(d) | Self::BadClaim(d) | Self::RequestMismatch(d) => {
                write!(f, "{}: {d}", self.code())
            }
            Self::HeaderBindingRequired(h) => write!(f, "{}: {h}", self.code()),
            _ => f.write_str(self.code()),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<turnkey_jwt_verify::Error> for AuthError {
    fn from(e: turnkey_jwt_verify::Error) -> Self {
        use turnkey_jwt_verify::Error as E;
        match e {
            E::Malformed(d) => Self::Malformed(d),
            E::UnsupportedAlgorithm => Self::UnsupportedAlgorithm,
            E::WrongType => Self::WrongType,
            E::UnknownKey => Self::UnknownKey,
            E::BadSignature => Self::BadSignature,
            E::BadClaim(d) => Self::BadClaim(d),
            E::LifetimeTooLong => Self::LifetimeTooLong,
            E::NotYetValid => Self::NotYetValid,
            E::Expired => Self::Expired,
            _ => Self::Malformed("token"),
        }
    }
}

/// An invalid [`VerifierConfig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError(pub String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid verifier configuration: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

/// Check that the received request matches the verified descriptor and the
/// route rule. Path and query are compared byte for byte.
///
/// # Errors
///
/// Returns [`AuthError::RequestMismatch`], [`AuthError::BodyDigestRequired`],
/// or [`AuthError::HeaderBindingRequired`].
pub fn bind_request(
    claims: &Claims,
    rule: &RouteRule,
    method: &str,
    path: &str,
    raw_query: Option<&str>,
    received_headers: &HeaderMap,
    body: &[u8],
) -> Result<(), AuthError> {
    let r = &claims.tk_request;
    if r.method != method {
        return Err(AuthError::RequestMismatch("method"));
    }
    if r.path != path {
        return Err(AuthError::RequestMismatch("path"));
    }
    if r.query != raw_query.unwrap_or("") {
        return Err(AuthError::RequestMismatch("query"));
    }

    let bound = claims.bound_headers();
    if let Some(missing) = rule.required_headers.iter().find(|h| !bound.contains(h)) {
        return Err(AuthError::HeaderBindingRequired(missing.clone()));
    }
    let want_digests: &[String] = claims.tk_header_digest.as_deref().unwrap_or_default();
    if want_digests.len() != bound.len() {
        return Err(AuthError::RequestMismatch("header digest"));
    }
    for (name, want) in bound.iter().zip(want_digests) {
        let got = headers::header_digest(name, received_headers).map_err(|e| match e {
            HeaderError::Absent(_) => AuthError::RequestMismatch("bound header is absent"),
            HeaderError::BadValue(_) => AuthError::RequestMismatch(
                "bound header value has a byte outside visible ASCII, space, and tab",
            ),
            // header_digest only yields Absent or BadValue; names are checked in jwt::verify.
            _ => AuthError::RequestMismatch("bound header is not a valid bound field"),
        })?;
        if got != *want {
            return Err(AuthError::RequestMismatch("header digest"));
        }
    }

    match claims.tk_body_digest.as_deref() {
        Some(want) if sha256_digest(body) != want => Err(AuthError::RequestMismatch("body digest")),
        None if rule.require_body_digest => Err(AuthError::BodyDigestRequired),
        _ => Ok(()),
    }
}

/// The parts of a received request that the profile binds.
#[derive(Debug, Clone, Copy)]
pub struct ReceivedRequest<'a> {
    /// Uppercase HTTP method as received.
    pub method: &'a str,
    /// The request path, without the query.
    pub path: &'a str,
    /// The raw query string without `?`, if any.
    pub raw_query: Option<&'a str>,
    /// The received header fields (lowercase names, repeated fields in
    /// order, as hyper delivers them).
    pub headers: &'a HeaderMap,
    /// The exact body bytes.
    pub body: &'a [u8],
}

/// A configured verifier: settings and trusted keys.
///
/// The verifier is independent of the HTTP framework. The `axum` feature adds
/// `middleware`, which drives it from a route layer.
pub struct Verifier {
    config: VerifierConfig,
    keys: Arc<dyn KeySource>,
}

impl Verifier {
    /// Build a verifier.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] for an empty audience, a malformed
    /// organization ID, a skew above [`jwt::MAX_CLOCK_SKEW_SECS`], or a
    /// lifetime of zero or above [`jwt::MAX_LIFETIME_SECS`].
    pub fn new(config: VerifierConfig, keys: Arc<dyn KeySource>) -> Result<Self, ConfigError> {
        config.validate()?;
        Ok(Self { config, keys })
    }

    /// Run the token checks only, without the request binding.
    ///
    /// # Errors
    ///
    /// The first failed check.
    pub fn verify_token(&self, token: &str, now: u64) -> Result<Claims, AuthError> {
        jwt::verify(token, &self.config, self.keys.as_ref(), now)
    }

    /// Check that already verified `claims` are still within their validity
    /// window at `now`. The middleware calls this after it buffers the body,
    /// so a slow body cannot carry a token past `exp`.
    #[cfg(feature = "axum")]
    pub(crate) fn check_still_valid(&self, claims: &Claims, now: u64) -> Result<(), AuthError> {
        jwt::check_window(claims, &self.config, now)
    }

    /// The full check for one request: verify the token, then bind it to the
    /// received request under `rule`.
    ///
    /// # Errors
    ///
    /// The first failed check.
    pub fn authorize(
        &self,
        token: &str,
        rule: &RouteRule,
        received: &ReceivedRequest<'_>,
        now: u64,
    ) -> Result<Claims, AuthError> {
        let claims = self.verify_token(token, now)?;
        bind_request(
            &claims,
            rule,
            received.method,
            received.path,
            received.raw_query,
            received.headers,
            received.body,
        )?;
        Ok(claims)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::jwt::{TYP, VerifierConfig, verify};
    use super::*;
    use p256::ecdsa::signature::Signer;
    use p256::ecdsa::{Signature, SigningKey};
    use rand_core::OsRng;
    use serde_json::{Value, json};
    use turnkey_jwt_verify::{ProtectedHeader, VerificationKey, thumbprint_p256};

    pub(crate) struct TestKey {
        key: SigningKey,
        point: [u8; 65],
        pub kid: String,
    }

    impl TestKey {
        pub(crate) fn new() -> Self {
            let key = SigningKey::random(&mut OsRng);
            let point = Self::point_of(&key);
            let kid = thumbprint_p256(
                &b64::url_encode(&point[1..33]),
                &b64::url_encode(&point[33..]),
            );
            Self { key, point, kid }
        }

        fn point_of(key: &SigningKey) -> [u8; 65] {
            let encoded = key.verifying_key().to_encoded_point(false);
            let mut point = [0u8; 65];
            point.copy_from_slice(encoded.as_bytes());
            point
        }

        pub(crate) fn sign(&self, header: &Value, claims: &Value) -> String {
            self.sign_raw(&header.to_string(), &claims.to_string())
        }

        fn sign_raw(&self, header: &str, payload: &str) -> String {
            let input = format!(
                "{}.{}",
                b64::url_encode(header.as_bytes()),
                b64::url_encode(payload.as_bytes())
            );
            let sig: Signature = self.key.sign(input.as_bytes());
            format!("{input}.{}", b64::url_encode(&sig.to_bytes()))
        }

        pub(crate) fn keys(&self) -> Keys {
            Keys(vec![VerificationKey {
                kid: self.kid.clone(),
                point: self.point,
            }])
        }
    }

    pub(crate) struct Keys(pub(crate) Vec<VerificationKey>);
    impl KeySource for Keys {
        fn key(&self, header: &ProtectedHeader) -> Option<VerificationKey> {
            self.0
                .iter()
                .find(|k| header.kid() == Some(&k.kid))
                .cloned()
        }
    }

    pub(crate) const AUD: &str = "https://app.example";
    const ORG_ID: &str = "parent_org_1";
    const ISS: &str = "https://id.turnkey.com/t/parent_org_1";
    const ORG: &str = "urn:turnkey:org:sub_org_1";
    const SUB: &str = "urn:turnkey:user:user_1";
    pub(crate) const NOW: u64 = 1_800_000_000;
    pub(crate) const PATH: &str = "/v1/resources/12345/actions";
    pub(crate) const QUERY: &str = "tag=blue&role=viewer%3Aread";
    pub(crate) const BODY: &[u8] = br#"{"recipient_key":"x"}"#;
    const ACCEPT: &str = "application/json";
    const API_VERSION: &str = "2026-01-01";
    /// Bound in this order on purpose: names need not be sorted.
    const BOUND: [&str; 2] = ["x-api-version", "accept"];

    pub(crate) fn rule() -> RouteRule {
        RouteRule {
            required_headers: vec!["accept".into(), "x-api-version".into()],
            require_body_digest: true,
        }
    }

    pub(crate) fn sent_headers() -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("accept", ACCEPT.parse().unwrap());
        h.insert("x-api-version", API_VERSION.parse().unwrap());
        h.insert("content-type", "application/json".parse().unwrap());
        h
    }

    pub(crate) fn cfg() -> VerifierConfig {
        VerifierConfig::new(AUD, ORG_ID)
    }

    pub(crate) fn header(k: &TestKey) -> Value {
        json!({"alg": "ES256", "kid": k.kid, "typ": TYP})
    }

    pub(crate) fn claims() -> Value {
        json!({
            "aud": AUD, "iss": ISS, "sub": SUB, "org": ORG,
            "iat": NOW, "exp": NOW + 300,
            "tk_request": {
                "method": "POST", "path": PATH, "query": QUERY, "headers": BOUND,
            },
            "tk_body_digest": digest::sha256_digest(BODY),
            "tk_header_digest": headers::header_digests(&BOUND, &sent_headers()).unwrap(),
        })
    }

    fn run(k: &TestKey, h: &Value, c: &Value, now: u64) -> Result<Claims, AuthError> {
        let keys = k.keys();
        verify(&k.sign(h, c), &cfg(), &keys, now)
    }

    fn verify_claims(c: &Value) -> Result<Claims, AuthError> {
        let k = TestKey::new();
        run(&k, &header(&k), c, NOW)
    }

    fn claim_error(c: Value) -> AuthError {
        verify_claims(&c).unwrap_err()
    }

    fn is_bad_claim(c: Value) -> bool {
        matches!(verify_claims(&c), Err(AuthError::BadClaim(_)))
    }

    fn bind(c: &Claims, h: &HeaderMap, body: &[u8]) -> Result<(), AuthError> {
        bind_request(c, &rule(), "POST", PATH, Some(QUERY), h, body)
    }

    fn with(mut v: Value, key: &str, val: Value) -> Value {
        v[key] = val;
        v
    }

    fn without(mut v: Value, key: &str) -> Value {
        v.as_object_mut().unwrap().remove(key);
        v
    }

    fn with_request(key: &str, val: Value) -> Value {
        let mut c = claims();
        c["tk_request"][key] = val;
        c
    }

    fn with_raw_headers(names: Value, digests: Value) -> Value {
        let mut c = with_request("headers", names);
        c["tk_header_digest"] = digests;
        c
    }

    fn with_header_digests(c: Value, names: &[&str], h: &HeaderMap) -> Value {
        with(
            c,
            "tk_header_digest",
            json!(headers::header_digests(names, h).unwrap()),
        )
    }

    fn with_headers(names: &[&str], h: &HeaderMap) -> Value {
        with_header_digests(with_request("headers", json!(names)), names, h)
    }

    fn without_request(key: &str) -> Value {
        let mut c = claims();
        c["tk_request"].as_object_mut().unwrap().remove(key);
        c
    }

    fn no_bound_headers_claims() -> Value {
        without(without_request("headers"), "tk_header_digest")
    }

    /// A rule with no required headers. It still requires a body digest.
    fn body_digest_rule() -> RouteRule {
        RouteRule {
            require_body_digest: true,
            ..RouteRule::default()
        }
    }

    #[test]
    fn accepts_valid_token_and_binds_request() {
        let k = TestKey::new();
        let c = run(&k, &header(&k), &claims(), NOW + 1).unwrap();
        assert_eq!(c.iss, ISS);
        assert_eq!(c.sub, SUB);
        assert_eq!(c.org, ORG);
        bind(&c, &sent_headers(), BODY).unwrap();
    }

    #[test]
    fn verifier_authorize_runs_every_step_and_allows_repeats() {
        let k = TestKey::new();
        let v = Verifier::new(cfg(), Arc::new(k.keys())).unwrap();
        let token = k.sign(&header(&k), &claims());
        let h = sent_headers();
        let received = ReceivedRequest {
            method: "POST",
            path: PATH,
            raw_query: Some(QUERY),
            headers: &h,
            body: BODY,
        };
        for _ in 0..3 {
            let c = v.authorize(&token, &rule(), &received, NOW).unwrap();
            assert_eq!((c.iss.as_str(), c.sub.as_str()), (ISS, SUB));
        }
        let wrong = ReceivedRequest {
            body: b"{}",
            ..received
        };
        assert_eq!(
            v.authorize(&token, &rule(), &wrong, NOW),
            Err(AuthError::RequestMismatch("body digest"))
        );
        assert_eq!(
            v.authorize(&token, &rule(), &received, NOW + 300),
            Err(AuthError::Expired)
        );
    }

    #[test]
    fn config_is_validated() {
        let keys: Arc<dyn KeySource> = Arc::new(Keys(vec![]));
        let bad = |c: VerifierConfig| Verifier::new(c, Arc::clone(&keys)).is_err();
        assert!(bad(VerifierConfig::new("", ORG_ID)));
        assert!(bad(VerifierConfig::new(AUD, "")));
        assert!(
            bad(VerifierConfig::new(AUD, "urn:turnkey:org:parent_org_1")),
            "URN instead of an ID"
        );
        assert!(
            bad(VerifierConfig::new(AUD, ISS)),
            "issuer URL instead of an ID"
        );
        assert!(bad(VerifierConfig::new(AUD, "a b")));
        let mut c = cfg();
        c.clock_skew_secs = jwt::MAX_CLOCK_SKEW_SECS + 1;
        assert!(bad(c));
        let mut c = cfg();
        c.max_lifetime_secs = jwt::MAX_LIFETIME_SECS + 1;
        assert!(bad(c));
        let mut c = cfg();
        c.max_lifetime_secs = 0;
        assert!(bad(c));
        assert!(Verifier::new(cfg(), keys).is_ok());
    }

    #[test]
    fn rejects_bad_alg_typ_kid_and_jwk_members() {
        let k = TestKey::new();
        let h = header(&k);
        let e = |h: Value| run(&k, &h, &claims(), NOW).unwrap_err();
        assert_eq!(
            e(with(h.clone(), "alg", json!("HS256"))),
            AuthError::UnsupportedAlgorithm
        );
        assert_eq!(
            e(with(h.clone(), "alg", json!("none"))),
            AuthError::UnsupportedAlgorithm
        );
        assert_eq!(
            e(with(h.clone(), "typ", json!("JWT"))),
            AuthError::WrongType
        );
        assert_eq!(
            e(with(
                h.clone(),
                "typ",
                json!("application/vnd.turnkey.dpop-proof-v1+jwt")
            )),
            AuthError::WrongType
        );
        assert_eq!(
            e(with(h.clone(), "kid", json!("other"))),
            AuthError::UnknownKey
        );
        assert!(matches!(
            e(with(h.clone(), "jwk", json!({}))),
            AuthError::Malformed(_)
        ));
        assert!(matches!(
            e(with(h.clone(), "crit", json!(["b64"]))),
            AuthError::Malformed(_)
        ));
        assert!(matches!(e(without(h, "kid")), AuthError::Malformed(_)));
    }

    #[test]
    fn rejects_forged_swapped_and_oversized_tokens() {
        let k = TestKey::new();
        let other = TestKey::new();
        let forged = other.sign(&header(&k), &claims());
        let keys = k.keys();
        assert_eq!(
            verify(&forged, &cfg(), &keys, NOW).unwrap_err(),
            AuthError::BadSignature
        );
        let good = k.sign(&header(&k), &claims());
        let parts: Vec<&str> = good.split('.').collect();
        let swapped = format!(
            "{}.{}.{}",
            parts[0],
            b64::url_encode(with(claims(), "sub", json!("root")).to_string().as_bytes()),
            parts[2]
        );
        assert_eq!(
            verify(&swapped, &cfg(), &keys, NOW).unwrap_err(),
            AuthError::BadSignature
        );
        assert!(matches!(
            verify(&format!("{good}.x"), &cfg(), &keys, NOW),
            Err(AuthError::Malformed(_))
        ));
        let huge = format!("{}.{}", "a".repeat(jwt::MAX_TOKEN_BYTES), "b.c");
        assert_eq!(
            verify(&huge, &cfg(), &keys, NOW),
            Err(AuthError::Malformed("token too large"))
        );
    }

    #[test]
    fn invalid_json_payload_is_malformed() {
        let k = TestKey::new();
        let token = k.sign_raw(&header(&k).to_string(), "{\"aud\":");
        assert_eq!(
            verify(&token, &cfg(), &k.keys(), NOW),
            Err(AuthError::Malformed("payload"))
        );
    }

    #[test]
    fn audience_must_be_the_configured_string_exactly() {
        assert_eq!(
            claim_error(with(claims(), "aud", json!("https://other.example"))),
            AuthError::WrongAudience
        );
        assert_eq!(
            claim_error(with(claims(), "aud", json!("https://app.example/"))),
            AuthError::WrongAudience
        );
        assert!(is_bad_claim(with(claims(), "aud", json!([AUD]))));
    }

    #[test]
    fn iss_must_be_the_configured_top_level_organization() {
        assert_eq!(
            claim_error(with(
                claims(),
                "iss",
                json!("https://id.turnkey.com/t/parent_org_2")
            )),
            AuthError::WrongIssuer
        );
        for bad in [
            json!(ORG_ID),
            json!("urn:turnkey:org:parent_org_1"),
            json!("https://auth.turnkey.com/t/parent_org_1"),
            json!("https://id.turnkey.com/t/"),
            json!("https://id.turnkey.com/t/parent_org_1/"),
            json!([ISS]),
        ] {
            assert!(is_bad_claim(with(claims(), "iss", bad.clone())), "{bad}");
        }
        assert!(verify_claims(&with(claims(), "org", json!("urn:turnkey:org:sub_org_2"))).is_ok());
        assert!(
            verify_claims(&with(
                claims(),
                "org",
                json!("urn:turnkey:org:parent_org_1")
            ))
            .is_ok()
        );
    }

    #[test]
    fn org_and_sub_need_their_urn_prefix_and_an_id() {
        for (claim, bad) in [
            ("org", "urn:turnkey:organization:sub_org_1"),
            ("org", "urn:turnkey:org:"),
            ("org", "sub_org_1"),
            ("sub", "urn:turnkey:user:"),
            ("sub", "user_1"),
            ("sub", "urn:turnkey:org:user_1"),
            ("sub", ""),
        ] {
            assert!(
                is_bad_claim(with(claims(), claim, json!(bad))),
                "{claim}={bad}"
            );
        }
    }

    #[test]
    fn clock_failure_sentinel_rejects_a_token_valid_at_epoch() {
        let k = TestKey::new();
        let h = header(&k);
        let c = with(with(claims(), "iat", json!(0)), "exp", json!(300));
        assert_eq!(run(&k, &h, &c, u64::MAX).unwrap_err(), AuthError::Expired);
    }

    #[test]
    fn enforces_exp_iat_skew_and_lifetime_bounds() {
        let k = TestKey::new();
        let h = header(&k);
        let t = |iat: u64, exp: u64, now: u64| {
            run(
                &k,
                &h,
                &with(with(claims(), "iat", json!(iat)), "exp", json!(exp)),
                now,
            )
        };
        assert!(t(NOW, NOW + 300, NOW + 299).is_ok());
        assert_eq!(
            t(NOW, NOW + 300, NOW + 300).unwrap_err(),
            AuthError::Expired
        );
        assert_eq!(t(NOW, NOW + 60, NOW + 61).unwrap_err(), AuthError::Expired);
        assert_eq!(
            t(NOW, NOW + 301, NOW).unwrap_err(),
            AuthError::LifetimeTooLong
        );
        assert!(t(NOW + 5, NOW + 30, NOW).is_ok(), "iat inside skew");
        assert_eq!(
            t(NOW + 6, NOW + 30, NOW).unwrap_err(),
            AuthError::NotYetValid
        );
        assert_eq!(
            t(NOW, NOW, NOW).unwrap_err(),
            AuthError::BadClaim("exp must be after iat")
        );
        assert_eq!(
            t(NOW, NOW - 1, NOW).unwrap_err(),
            AuthError::BadClaim("exp must be after iat")
        );
        assert!(matches!(
            run(&k, &h, &with(claims(), "exp", json!(1.8e9)), NOW).unwrap_err(),
            AuthError::BadClaim(_)
        ));
        let mut short = cfg();
        short.max_lifetime_secs = 30;
        let keys = k.keys();
        assert_eq!(
            verify(&k.sign(&h, &claims()), &short, &keys, NOW).unwrap_err(),
            AuthError::LifetimeTooLong
        );
    }

    #[test]
    fn claim_schema_is_closed() {
        assert!(is_bad_claim(with(claims(), "extra", json!(1))));
        assert!(
            is_bad_claim(with(claims(), "jti", json!("jti_1"))),
            "no jti"
        );
        assert!(is_bad_claim(with(claims(), "tk_activity", json!("a"))));
        assert!(is_bad_claim(with(claims(), "tk_body_sha256", json!("x"))));
        assert!(is_bad_claim(with(claims(), "tk_body_digest", Value::Null)));
        assert!(is_bad_claim(with(claims(), "org", Value::Null)));
        assert!(is_bad_claim(with(
            claims(),
            "tk_parent_organization",
            json!("urn:turnkey:org:parent_org_1")
        )));
        for claim in ["aud", "iss", "org", "sub", "iat", "exp", "tk_request"] {
            assert!(is_bad_claim(without(claims(), claim)), "missing {claim}");
        }
        assert!(
            is_bad_claim(with(
                claims(),
                "tk_request",
                json!(["POST", PATH, QUERY, []])
            )),
            "positional tk_request"
        );
        let positional: Vec<Value> = ["aud", "exp", "iat", "iss", "sub"]
            .iter()
            .map(|k| claims()[k].clone())
            .collect();
        assert!(is_bad_claim(json!(positional)), "positional claim set");
        assert!(is_bad_claim(with_request("nonce", json!("n1"))));
        assert!(is_bad_claim(without_request("query")));
        assert!(is_bad_claim(with_request(
            "query",
            json!(format!("a={}", "x".repeat(jwt::MAX_QUERY_BYTES - 1)))
        )));
        assert!(
            verify_claims(&with_request(
                "query",
                json!(format!("a={}", "x".repeat(jwt::MAX_QUERY_BYTES - 2)))
            ))
            .is_ok(),
            "a query of exactly MAX_QUERY_BYTES is accepted"
        );
        assert!(is_bad_claim(with_request(
            "path",
            json!(format!("/{}", "x".repeat(jwt::MAX_PATH_BYTES)))
        )));
        assert!(
            verify_claims(&with_request(
                "path",
                json!(format!("/{}", "x".repeat(jwt::MAX_PATH_BYTES - 1)))
            ))
            .is_ok(),
            "a path of exactly MAX_PATH_BYTES is accepted"
        );
        for m in ["post", "", "GET POST", "G\u{e9}T"] {
            assert!(
                is_bad_claim(with_request("method", json!(m))),
                "method {m:?}"
            );
        }
        let mut c = no_bound_headers_claims();
        c["tk_request"]["method"] = json!("M-SEARCH");
        assert!(verify_claims(&c).is_ok(), "any uppercase token method");
        assert!(is_bad_claim(with(
            claims(),
            "tk_header_digest",
            json!([null, digest::sha256_digest(b"")])
        )));
    }

    #[test]
    fn header_names_must_be_valid_unique_and_paired_with_digests() {
        let d = digest::sha256_digest(b"x");
        for names in [
            json!(["accept", "accept"]),
            json!(["Accept"]),
            json!(["authorization"]),
            json!(["accept", "authorization"]),
            json!(["host"]),
            json!(["proxy-authorization"]),
            json!(["transfer-encoding"]),
            json!([":path"]),
            json!([""]),
        ] {
            let n = names.as_array().unwrap().len();
            assert_eq!(
                claim_error(with_raw_headers(names.clone(), json!(vec![d.clone(); n]))),
                AuthError::BadClaim("tk_request.headers"),
                "{names}"
            );
        }
        for names in [json!([1]), json!("accept")] {
            assert!(
                is_bad_claim(with_raw_headers(names.clone(), json!([d.clone()]))),
                "{names}"
            );
        }
        let too_many = headers::MAX_HEADER_NAMES + 1;
        let many: Vec<String> = (0..too_many).map(|i| format!("x-{i:02}")).collect();
        assert!(is_bad_claim(with_raw_headers(
            json!(many),
            json!(vec![d.clone(); too_many])
        )));
        let mismatch = AuthError::BadClaim(
            "tk_header_digest must have one entry per name in tk_request.headers",
        );
        let no_names = without_request("headers");
        for (label, c) in [
            (
                "fewer digests",
                with(claims(), "tk_header_digest", json!([d])),
            ),
            (
                "names, digests absent",
                without(claims(), "tk_header_digest"),
            ),
            (
                "names, digests empty",
                with(claims(), "tk_header_digest", json!([])),
            ),
            ("names absent, digests", no_names.clone()),
            (
                "names absent, digests empty",
                with(no_names, "tk_header_digest", json!([])),
            ),
            ("names empty, digests", with_request("headers", json!([]))),
            (
                "names empty, digests absent",
                without(with_request("headers", json!([])), "tk_header_digest"),
            ),
        ] {
            assert_eq!(claim_error(c), mismatch, "{label}");
        }
    }

    #[test]
    fn no_bound_headers_is_both_absent_or_both_empty() {
        let body_digest = body_digest_rule();
        let h = sent_headers();
        for c in [
            no_bound_headers_claims(),
            with_raw_headers(json!([]), json!([])),
        ] {
            let c = verify_claims(&c).unwrap();
            assert!(c.bound_headers().is_empty());
            assert_eq!(
                bind_request(&c, &body_digest, "POST", PATH, Some(QUERY), &h, BODY),
                Ok(())
            );
            assert_eq!(
                bind(&c, &h, BODY),
                Err(AuthError::HeaderBindingRequired("accept".into()))
            );
        }
    }

    #[test]
    fn digest_claims_accept_only_the_profile_digest_format() {
        let v: Value = serde_json::from_str(digest::tests::VECTORS).unwrap();
        let cases = v["format"].as_array().unwrap();
        assert!(cases.len() >= 8);
        for case in cases {
            let value = case["value"].clone();
            let ok = case["ok"].as_bool().unwrap();
            assert_eq!(
                is_bad_claim(with(claims(), "tk_body_digest", value.clone())),
                !ok,
                "body {value}"
            );
            let mut c = claims();
            c["tk_header_digest"][1] = value.clone();
            assert_eq!(is_bad_claim(c), !ok, "header {value}");
        }
    }

    #[test]
    fn duplicate_claim_names_are_rejected() {
        let k = TestKey::new();
        let body = claims().to_string();
        let dup = body.replacen('{', r#"{"sub":"urn:turnkey:user:root","#, 1);
        let token = k.sign_raw(&header(&k).to_string(), &dup);
        let keys = k.keys();
        assert!(matches!(
            verify(&token, &cfg(), &keys, NOW),
            Err(AuthError::BadClaim(_))
        ));
    }

    #[test]
    fn request_binding_is_byte_exact() {
        let c = verify_claims(&claims()).unwrap();
        let h = sent_headers();
        let b = |m: &str, p: &str, q: Option<&str>| bind_request(&c, &rule(), m, p, q, &h, BODY);
        assert_eq!(
            b("GET", PATH, Some(QUERY)),
            Err(AuthError::RequestMismatch("method"))
        );
        for p in [
            "/v1/resources/99/actions",
            "/v1/resources/12345/actions/",
            "/v1/resources/12345//actions",
            "/v1/resources/%31%32345/actions",
            "/V1/resources/12345/actions",
        ] {
            assert_eq!(
                b("POST", p, Some(QUERY)),
                Err(AuthError::RequestMismatch("path")),
                "{p}"
            );
        }
        for q in [
            Some("role=viewer%3Aread&tag=blue"),
            Some("tag=blue&role=viewer%3aread"),
            Some("tag=blue&role=viewer:read"),
            Some("tag=blue&role=viewer%3Aread&"),
            Some(""),
            None,
        ] {
            assert_eq!(
                b("POST", PATH, q),
                Err(AuthError::RequestMismatch("query")),
                "{q:?}"
            );
        }
        assert_eq!(
            bind(&c, &h, b"{}"),
            Err(AuthError::RequestMismatch("body digest"))
        );
        let c = verify_claims(&with_request("query", json!(""))).unwrap();
        for q in [None, Some("")] {
            assert_eq!(bind_request(&c, &rule(), "POST", PATH, q, &h, BODY), Ok(()));
        }
    }

    fn with_sent(edit: impl FnOnce(&mut HeaderMap)) -> HeaderMap {
        let mut h = sent_headers();
        edit(&mut h);
        h
    }

    #[test]
    fn unbound_headers_are_ignored_and_bound_values_must_match() {
        let c = verify_claims(&claims()).unwrap();
        let h = with_sent(|h| {
            h.insert("user-agent", "other".parse().unwrap());
            h.insert("content-type", "text/plain".parse().unwrap());
        });
        assert_eq!(bind(&c, &h, BODY), Ok(()));
        let expect = |edit: fn(&mut HeaderMap), want: AuthError| {
            assert_eq!(bind(&c, &with_sent(edit), BODY), Err(want));
        };
        expect(
            |h| {
                h.insert("x-api-version", "2026-03-10".parse().unwrap());
            },
            AuthError::RequestMismatch("header digest"),
        );
        expect(
            |h| {
                h.append("accept", "text/plain".parse().unwrap());
            },
            AuthError::RequestMismatch("header digest"),
        );
        expect(
            |h| {
                h.remove("accept");
            },
            AuthError::RequestMismatch("bound header is absent"),
        );
        let h = with_sent(|h| {
            h.insert("accept", format!(" \t{ACCEPT} ").parse().unwrap());
        });
        assert_eq!(bind(&c, &h, BODY), Ok(()));
        expect(
            |h| {
                h.insert(
                    "accept",
                    http::HeaderValue::from_bytes("caf\u{e9}".as_bytes()).unwrap(),
                );
            },
            AuthError::RequestMismatch(
                "bound header value has a byte outside visible ASCII, space, and tab",
            ),
        );
        let swapped = with_header_digests(claims(), &["accept", "x-api-version"], &sent_headers());
        let c = verify_claims(&swapped).unwrap();
        assert_eq!(
            bind(&c, &sent_headers(), BODY),
            Err(AuthError::RequestMismatch("header digest"))
        );
        let short = with(
            claims(),
            "tk_header_digest",
            json!([headers::header_digests(&BOUND, &sent_headers())
                .unwrap()
                .swap_remove(0)]),
        );
        let c: Claims = serde_json::from_value(short).unwrap();
        assert_eq!(
            bind(&c, &sent_headers(), BODY),
            Err(AuthError::RequestMismatch("header digest"))
        );
    }

    #[test]
    fn repeated_bound_header_binds_all_lines_in_order() {
        let mut h = sent_headers();
        h.append("accept", "text/plain".parse().unwrap());
        let c = with_header_digests(claims(), &BOUND, &h);
        let c = verify_claims(&c).unwrap();
        assert_eq!(bind(&c, &h, BODY), Ok(()));
        let mut reordered = sent_headers();
        reordered.insert("accept", "text/plain".parse().unwrap());
        reordered.append("accept", ACCEPT.parse().unwrap());
        assert_eq!(
            bind(&c, &reordered, BODY),
            Err(AuthError::RequestMismatch("header digest"))
        );
        assert_eq!(
            bind(&c, &sent_headers(), BODY),
            Err(AuthError::RequestMismatch("header digest"))
        );
    }

    #[test]
    fn route_requires_header_binding() {
        let h = sent_headers();
        let one = with_headers(&["accept"], &h);
        let c = verify_claims(&one).unwrap();
        assert_eq!(
            bind(&c, &h, BODY),
            Err(AuthError::HeaderBindingRequired("x-api-version".into()))
        );
        let names = ["content-type", "accept", "x-api-version"];
        let more = with_headers(&names, &h);
        let c = verify_claims(&more).unwrap();
        assert_eq!(bind(&c, &h, BODY), Ok(()));
    }

    #[test]
    fn body_digest_must_match_the_received_bytes_exactly() {
        let c = verify_claims(&without(claims(), "tk_body_digest")).unwrap();
        let h = sent_headers();
        assert_eq!(bind(&c, &h, b""), Err(AuthError::BodyDigestRequired));
        let unbound = RouteRule {
            require_body_digest: false,
            ..rule()
        };
        assert_eq!(
            bind_request(&c, &unbound, "POST", PATH, Some(QUERY), &h, b"anything"),
            Ok(())
        );
        let empty = with(
            claims(),
            "tk_body_digest",
            json!(digest::sha256_digest(b"")),
        );
        let c = verify_claims(&empty).unwrap();
        assert_eq!(bind(&c, &h, b""), Ok(()));
        assert_eq!(
            bind(&c, &h, b" "),
            Err(AuthError::RequestMismatch("body digest"))
        );
    }
}
