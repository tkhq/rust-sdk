# `turnkey_jwt_verify`

Strict compact-JWS parsing and ES256 verification for JWTs that Turnkey
signs. This crate is the shared core of the profile verifiers in
[`turnkey_jwt_profiles`](https://docs.rs/turnkey_jwt_profiles). It has no profile claims, no
request binding, and no HTTP framework code.

Most services use a profile from `turnkey_jwt_profiles` and do not call this
crate directly.

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

## What `verify` checks

- The token is a compact JWS of at most `JwsRules::max_token_bytes` bytes,
  with exactly three parts in strict unpadded base64url.
- The protected header has exactly `alg`, `kid`, and `typ`. Unknown,
  repeated, and null members are rejected. `JwsRules::kid_optional` also
  accepts a header with only `alg` and `typ`.
- `alg` is `ES256`. No other algorithm is accepted.
- `typ` equals `JwsRules::typ` exactly. The caller supplies it, so tokens of
  one profile cannot pass as another.
- A `KeySource` selects the key from the header. When the header has a
  `kid`, the selected key's `kid` must equal it. `PinnedKeySet` selects by
  `kid`, so a header without one only verifies against a single key the
  caller already trusts, such as a `VerificationKey`. Nothing in the token
  can name a key location.
- The ES256 signature verifies over the signing input. No claim is read
  before this.

The protected header must be a JSON object; the array form that serde
accepts for structs is rejected.

`VerifiedJws::claims` then parses the payload. The payload must be a JSON
object with no nulls and no repeated member names at any depth. Use a struct
with `#[serde(deny_unknown_fields)]` for a closed claim schema, and
`#[serde(deserialize_with = "turnkey_jwt_verify::object")]` on nested struct
members so they must be objects too. A payload that is not JSON is
`Malformed`. A payload that is not a strict object or does not match the
struct is `BadClaim`.

`check_times` checks, in order: `exp > iat`, `exp - iat` at most the
maximum lifetime, `iat <= now + skew`, and `now < exp`. `unix_now` returns
`u64::MAX` if the clock is before 1970, so the expiry check fails closed.

Replay protection is not part of this crate.

## Public API

- `verify(token, &JwsRules, &dyn KeySource) -> Result<VerifiedJws, Error>`.
- `JwsRules::new(typ, max_token_bytes)` and `JwsRules::kid_optional`.
- `VerifiedJws::{header, payload, claims}` and
  `ProtectedHeader::{alg, typ, kid}`.
- `KeySource`: returns the trusted `VerificationKey` that a header selects.
  A `VerificationKey` is itself a `KeySource` for that one key.
- `PinnedKeySet::from_jwks_json` and `PinnedKeySet::get`: a fixed set of
  public P-256 keys from a JWK Set. A `kid` in the set must be the RFC 7638
  thumbprint.
- `thumbprint_p256(x, y)`: the RFC 7638 thumbprint of a P-256 JWK.
- `check_times`, `TimeRules::new(clock_skew_secs, max_lifetime_secs)`, and
  `unix_now`.
- `strict_json`: parse any JSON value with no nulls and no repeated names.
- `b64::{url_encode, url_decode, sha256_url}`: strict unpadded base64url.
- `Error`: the verification failure. `JwksError`: a key set error.
