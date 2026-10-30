# `turnkey_jwt_profiles`

Verifiers for Turnkey JWT profiles. Each profile is a module behind a cargo
feature. The shared JWS and key checks live in
[`turnkey_jwt_verify`](https://docs.rs/turnkey_jwt_verify).

## http-authz/v1

Verify Turnkey http-authz/v1 request-bound JWTs in a Rust HTTP service.

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
    println!("{} acting in {}", claims.sub, claims.org);
    Ok(())
}
```
