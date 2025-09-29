# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0](https://github.com/tkhq/rust-sdk/compare/turnkey_api_key_stamper-v0.3.0...turnkey_api_key_stamper-v0.4.0) - 2025-09-29

### Other

- Use workspace defined dependencies

## [0.2.0](https://github.com/tkhq/rust-sdk/compare/turnkey_api_key_stamper-v0.1.1...turnkey_api_key_stamper-v0.2.0) - 2025-08-15

### Added

- https://github.com/tkhq/rust-sdk/pull/41: add support for secp256k1 API key stamping. **Breaking**: consumers of this crate will have to import the new `Stamp` trait to use API keys.

## [0.1.1](https://github.com/tkhq/rust-sdk/compare/turnkey_api_key_stamper-v0.0.2...turnkey_api_key_stamper-v0.1.1) - 2025-08-05

### Added

- https://github.com/tkhq/rust-sdk/pull/25: Add changelogs to published crates

### Other

- https://github.com/tkhq/rust-sdk/pull/37: Release packages with `release-plz`. Going forward all SDK package versions in this repository will stay consistent with each other on each release.

## turnkey_api_key_stamper-v0.0.2

Initial release of this crate.
