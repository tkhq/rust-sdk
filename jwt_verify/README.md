# `turnkey_jwt_verify`

Core verification utils for JWTs that Turnkey mints. This crate has the generic core verification logic, but does not include support for any specific profiles. Concrete profile support can be found in [`turnkey_jwt_profiles`](https://docs.rs/turnkey_jwt_profiles). Most users are not expected to use this crate directly and should instead use [`turnkey_jwt_profiles`](https://docs.rs/turnkey_jwt_profiles).

## Example

```rust,no_run
use serde::Deserialize;
use turnkey_jwt_verify::{JwsRules, PinnedKeySet, TimeRules, check_times, unix_now, verify};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Claims {
    iat: u64,
    exp: u64,
    sub: String,
}

fn check(jwks: &[u8], token: &str) -> Result<String, Box<dyn std::error::Error>> {
    // In a real service, load the key set once at startup.
    let keys = PinnedKeySet::from_jwks_json(jwks)?;
    let rules = JwsRules::new("application/example+jwt", 4096);
    let claims: Claims = verify(token, &rules, &keys)?.claims()?;
    check_times(claims.iat, claims.exp, unix_now(), &TimeRules::new(5, 300))?;
    Ok(claims.sub)
}
```
