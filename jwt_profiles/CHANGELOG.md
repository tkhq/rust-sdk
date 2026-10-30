# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.17.1](https://github.com/tkhq/rust-sdk/compare/turnkey_jwt_profiles-v0.16.0...turnkey_jwt_profiles-v0.17.1) - 2026-10-30

### Other

- Validate organization, user, and issuer ids as UUIDs
- Lead the profiles README with the http-authz description
- Address review: bound headers, trailers, body read errors
- Make the crate docs unconditional and move the example to the module
- Cap the body limit the axum layer accepts
- Move the digest helpers to a top-level rfc9530 module
- Address review: workspace dependencies, README, drop the verify_tokens example
- Document the memory cost of the body limit
- Address review: claims struct guidance, constants, docs
- Update the token size in the profiles README
- Address review: key type, token size cap, docs
- Address review: docs, README, changelogs, release config, checked arithmetic
- Verify iss as the top-level org and require the org claim
- Polish turnkey_jwt_verify and turnkey_jwt_profiles
- Address review: strict JSON objects, public body limit, expiry recheck
- Split http-authz verifier into turnkey_jwt_verify and turnkey_jwt_profiles
