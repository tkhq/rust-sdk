# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Initial release: strict compact-JWS parsing and ES256 verification with a
  caller-supplied `typ`, a closed protected header (`alg`, `kid`, `typ`), a
  token size cap, the `KeySource` trait and `PinnedKeySet`, the RFC 7638
  thumbprint, generic `iat`/`exp` checks, and base64url helpers.
- `JwsRules::kid_optional`, which accepts a header without `kid`, and
  `KeySource` for a single `VerificationKey`. For SD-JWT Key Binding JWTs,
  which carry no `kid` and are checked against the `cnf` key. `JwsRules::new`
  still requires `kid`.
- `strict_json`, the claims parser's strictness for JSON that is not an
  object.
