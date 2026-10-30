# `turnkey_jwt_profiles`

Verifiers for Turnkey JWT profiles. Each profile is a module behind a cargo
feature. The shared JWS and key checks live in
[`turnkey_jwt_verify`](https://docs.rs/turnkey_jwt_verify).

| Module | Feature | Profile |
| ------ | ------- | ------- |
| `http_authz` | `http-authz` (default) | `turnkey.io/http-authz/v1` request-bound JWTs |

## http-authz/v1

Verify Turnkey http-authz/v1 request-bound JWTs in a Rust HTTP service.

A Turnkey organization can mint a JWT for one exact HTTP request with
`ACTIVITY_TYPE_MINT_JWT` and the `turnkey.io/http-authz/v1` profile (profile
enum `JWT_PROFILE_HTTP_AUTHZ_V1`). The service that receives the request uses
this crate to check the token and to check that the token matches the request.

### Example

```rust,no_run
use std::sync::Arc;
use turnkey_jwt_profiles::http_authz::jwks::PinnedKeySet;
use turnkey_jwt_profiles::http_authz::{ReceivedRequest, RouteRule, Verifier, VerifierConfig};

fn handle(
    jwks: &[u8],
    token: &str,
    headers: &http::HeaderMap,
    body: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    // In a real service, build the verifier once at startup.
    let keys = PinnedKeySet::from_jwks_json(jwks)?;
    let config = VerifierConfig::new("https://app.example", "parent_org_1");
    let verifier = Verifier::new(config, Arc::new(keys))?;
    let rule = RouteRule {
        required_headers: vec!["accept".into()],
        require_body_digest: true,
    };
    let received = ReceivedRequest {
        method: "POST",
        path: "/v1/things",
        raw_query: Some("a=1"),
        headers,
        body,
    };
    let claims = verifier.authorize(token, &rule, &received, turnkey_jwt_profiles::http_authz::unix_now())?;
    println!("{} acting in {}", claims.sub, claims.iss);
    Ok(())
}
```

### What the verifier checks

- The token is a compact JWS of at most 9 KiB. The protected header has
  exactly `alg`, `kid`, and `typ`. `alg` must be `ES256`. `typ` is
  `application/vnd.turnkey.http-authz-v1+jwt`.
- `kid` is the RFC 7638 thumbprint of the Turnkey signer key. It selects a
  key from a set that the service already trusts. Nothing in the token can
  name a key location. No claim is trusted before the signature verifies.
- The claim set is closed: `iss` (`urn:turnkey:org:<id>`, the issuing
  sub-organization), `sub` (`urn:turnkey:user:<id>`), `aud`, `iat`, `exp`,
  `tk_parent_organization`, `tk_request {method, path, query, headers?}`,
  `tk_header_digest?`, and `tk_body_digest?`. Unknown members, nulls, and
  wrong types are rejected. There is no `jti`.
- `tk_header_digest` has one digest per name in `tk_request.headers`, in the
  same order. With no bound headers, both are absent or both are empty
  arrays. Any other combination is a bad claim.
- `tk_parent_organization` must equal the configured parent organization. A
  token without it (minted by a parent organization) is rejected.
- `aud` equals the configured origin byte for byte. `now < exp`,
  `iat <= now + skew`, `exp > iat`, and `exp - iat` is at most 300 s (a
  service can require less). Skew defaults to 5 s and is at most 30 s.
- `tk_request.method` is an uppercase HTTP method token.
  `tk_request.path` is at most 2048 bytes and `tk_request.query` is at most
  2048 bytes. `tk_request.headers` has at most 32 names.
- Request binding: the method, path, and raw query must equal the received
  request byte for byte (no canonicalization). For each bound header, the
  verifier takes the received field lines in order, trims spaces and tabs
  from each, joins them with `", "`, and hashes the result. A bound header
  that is absent fails. `authorization`, `proxy-authorization`, `host`,
  hop-by-hop headers, and pseudo-headers can never be bound. The body digest
  is over the body bytes as received.
- Digests (body and each header) are exactly one RFC 9530 member,
  `sha-256=:<base64>:`, with standard padded base64. Any other form is a bad
  claim. The verifier computes the digest string again and compares it
  exactly.
- Tokens are not single-use. The holder can repeat the exact request until
  `exp`.

The handler gets the verified claims. Use `iss` (the sub-organization) and
`sub` (the user) to scope each action.

### Public API

All items are in `turnkey_jwt_profiles::http_authz`.

- `Verifier::new(VerifierConfig, Arc<dyn KeySource>)`, then `verify_token` or
  `authorize`. `VerifierConfig::new(audience, parent_organization_id)` sets
  the profile defaults. `clock_skew_secs` and `max_lifetime_secs` are checked
  against the profile bounds. The parent organization ID is the bare ID,
  without the URN prefix.
- `RouteRule { required_headers, require_body_digest }`.
- `ReceivedRequest` and `bind_request` for the request binding step alone.
- The `KeySource` trait and `jwks::PinnedKeySet::from_jwks_json`,
  re-exported from `turnkey_jwt_verify`.
- `middleware::route_layer(verifier, rule)` (feature `axum`): a
  `tower::Layer` for `MethodRouter::route_layer` or `Router::layer`. The
  handler reads the claims from the `middleware::Verified` extension.
  `middleware::protect` wraps a `MethodRouter` directly.
  The layer buffers at most `middleware::DEFAULT_MAX_BODY_BYTES` (4096) of
  body; `AuthzLayer::with_max_body_bytes` changes the limit. After the body
  arrives, the layer checks `iat` and `exp` again: the token must still be
  valid when the whole request has arrived.
  A rejected request gets `401` with `WWW-Authenticate: Bearer` and the
  body `{"error": "unauthorized"}`, or `413` with
  `{"error": "payload_too_large"}` for an oversized body. The body does not
  say which check failed; the layer logs the reason with `tracing`.
  `AuthzLayer::with_detailed_errors(true)` puts the specific code and
  message in the body, for debugging only.
- Helpers: `digest::{sha256_digest, is_digest}`,
  `headers::{header_value, header_digest, header_digests,
  check_header_names}`, `b64::{url_encode, url_decode, sha256_url}`, and
  `unix_now`.
- `jwt::ORG_URN_PREFIX` and `jwt::USER_URN_PREFIX` hold the URN prefixes.

### Trusted keys

Keys come from a JWK Set that the service pins in its configuration. The
`KeySource` trait lets a service supply keys from another source.

### Test vectors

`testdata/digests.json` holds body digests, digest-format accept and reject
cases (base64url, missing padding, `sha-512`, two members, parameters,
whitespace), per-header normalization (repeated fields, trimming, absent
header, bytes outside visible ASCII), and header-name rules. The unit tests
run them.

The `verify_tokens` example checks a file of minted tokens against the
requests they were minted for:

```sh
cargo run -p turnkey_jwt_profiles --example verify_tokens -- tokens.json
```

## Features

- `http-authz` (on by default): the `http_authz` module.
- `axum` (off by default, turns on `http-authz`): the Axum route layer in
  `http_authz::middleware`. Without it, the crate depends on `http` but not
  on Axum, so any HTTP stack can call `Verifier::authorize` with a
  `ReceivedRequest`.
