# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.19.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.18.0...turnkey_enclave_encrypt-v0.19.0) - 2026-11-12

### Other

- Import secrets with enclave_encrypt_v2 only.

## [0.18.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.17.2...turnkey_enclave_encrypt-v0.18.0) - 2026-11-11

### Other

- Merge pull request #306 from tkhq/zeke/enclave-encrypt-purpose-binding
- Keep the purpose binding encoder specific to ImportSecrets and test that another purpose with the same fields fails to decrypt.
- Take the ingress purpose as a parameter of the purpose binding encoder.
- Add purpose bindings to the enclave encrypt HPKE AAD.

## [0.17.2](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.17.1...turnkey_enclave_encrypt-v0.17.2) - 2026-11-09

### Other

- release v0.17.1

## [0.17.1](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.17.0...turnkey_enclave_encrypt-v0.17.1) - 2026-10-30

### Other

- release v0.17.1

## [0.17.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.16.0...turnkey_enclave_encrypt-v0.17.0) - 2026-10-29

### Other

- update Cargo.toml dependencies

## [0.16.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.15.0...turnkey_enclave_encrypt-v0.16.0) - 2026-09-17

### Other

- *(enclave-encrypt)* add secret-named encrypt_secret_bundle / decrypt_secret wrappers and use them in the secrets flow

## [0.15.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.14.0...turnkey_enclave_encrypt-v0.15.0) - 2026-08-28

### Other

- update Cargo.toml dependencies

## [0.11.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.10.0...turnkey_enclave_encrypt-v0.11.0) - 2026-07-15

### Other

- update Cargo.toml dependencies

## [0.10.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.9.0...turnkey_enclave_encrypt-v0.10.0) - 2026-07-07

### Other

- update Cargo.toml dependencies

## [0.8.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.7.0...turnkey_enclave_encrypt-v0.8.0) - 2026-06-30

### Changed

- Updated the crate to Rust edition 2024.

## [0.7.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.6.2...turnkey_enclave_encrypt-v0.7.0) - 2026-05-19

### Added

- Added reusable server-side decryption support for flows that use persistent enclave encryption keys.

### Changed

- Renamed the reusable server receiver API and updated its constructors to use quorum key types.

## [0.6.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.5.0...turnkey_enclave_encrypt-v0.6.0) - 2026-02-20

### Other

- https://github.com/tkhq/rust-sdk/pull/80: Security patch for `bytes` crate

## [0.4.0](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.3.0...turnkey_enclave_encrypt-v0.4.0) - 2025-09-29

### Other

- https://github.com/tkhq/rust-sdk/pull/59: bump Rust Crypto dependencies

## [0.1.1](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.1.0...turnkey_enclave_encrypt-v0.1.1) - 2025-08-05

### Added

- https://github.com/tkhq/rust-sdk/pull/25: Add changelogs to published crates

### Other

- https://github.com/tkhq/rust-sdk/pull/37: Release packages with `release-plz`. Going forward all SDK package versions in this repository will stay consistent with each other on each release.

## 0.1.0

Initial release of this crate.
