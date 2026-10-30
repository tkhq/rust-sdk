# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.17.1](https://github.com/tkhq/rust-sdk/compare/turnkey_jwt_verify-v0.16.0...turnkey_jwt_verify-v0.17.1) - 2026-10-30

### Other

- Address review: claims struct guidance, constants, docs
- Address review: key type, token size cap, docs
- Address review: docs, README, changelogs, release config, checked arithmetic
- opt-in kid-less headers and strict_json for SD-JWT
- Polish turnkey_jwt_verify and turnkey_jwt_profiles
- Address review: strict JSON objects, public body limit, expiry recheck
- Split http-authz verifier into turnkey_jwt_verify and turnkey_jwt_profiles
