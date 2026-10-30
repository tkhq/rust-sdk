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

`VerifiedJws::claims` then parses the payload. The payload must be a JSON
object with no nulls and no repeated member names at any depth. A payload
that is not JSON is `Malformed`. A payload that is not a strict object or
does not match the struct is `BadClaim`.

### Defining a claims struct

`claims` checks only the top level and the rules above. The struct decides
the rest, so define it as follows:

- Derive `Deserialize` with `#[serde(deny_unknown_fields)]`. Without it,
  serde ignores claims the profile does not define.
- Put `#[serde(deny_unknown_fields)]` on every nested struct too.
- Put `#[serde(deserialize_with = "turnkey_jwt_verify::object")]` on each
  member whose type is a struct. Serde also accepts a struct as a JSON array
  of its fields in order, and `object` rejects that form. A `Vec<Inner>` or
  `Option<Inner>` member is not covered: each `Inner` still accepts the array
  form.
- Use `Option<_>` with `#[serde(default)]` for an optional claim.

```rust
use serde::Deserialize;
use turnkey_jwt_verify::object;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Claims {
    sub: String,
    exp: u64,
    iat: u64,
    #[serde(deserialize_with = "object")]
    request: Request,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    method: String,
    path: String,
}
```
