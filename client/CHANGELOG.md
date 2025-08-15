# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.2.0](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.1.1...turnkey_client-v0.2.0) - 2025-08-15

### Other

### Added

- https://github.com/tkhq/rust-sdk/pull/41: add support for secp256k1 API key stamping. **Breaking**: consumers of this crate will have to import the new `Stamp` trait to use `TurnkeyClient`.
### Fixes

- https://github.com/tkhq/rust-sdk/pull/42: fix code generation for large integers so we parse from, and serialize to JSON strings. This is using `serde_with::serde_as`.

## [0.1.1](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.0.2...turnkey_client-v0.1.1) - 2025-08-05

### Added

- https://github.com/tkhq/rust-sdk/pull/34: implement additional `reqwest` features through features, and switch to `default-tls` as a default TLS feature.
* https://github.com/tkhq/rust-sdk/pull/28: Add summary and description to generated method docs
* https://github.com/tkhq/rust-sdk/pull/27: Re-export `turnkey_api_key_stamper::TurnkeyP256ApiKey` from `turnkey_client`
- https://github.com/tkhq/rust-sdk/pull/25: Add changelogs to published crates

### Other

- https://github.com/tkhq/rust-sdk/pull/37: Release packages with `release-plz`. Going forward all SDK package versions in this repository will stay consistent with each other on each release.

## 0.0.2

Initial release of this crate.
