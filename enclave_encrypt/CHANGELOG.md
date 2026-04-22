# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.6.3](https://github.com/tkhq/rust-sdk/compare/turnkey_enclave_encrypt-v0.6.2...turnkey_enclave_encrypt-v0.6.3) - 2026-04-22

### Added

- *(enclave_encrypt)* add persistent server key support

### Other

- Zeroize target_private_bytes
- remove TryFrom<&p256::SecretKey> for ReusableEnclaveEncryptServerRecv
- Add TryFrom<&qos_p256::P256Pair> for ReusableEnclaveEncryptServerRecv
- Refactor to ReusableEnclaveEncryptClientSend
- clarify enclave -> client forward secrecy
- Use TryFrom for ReusableEnclaveEncryptServerRecv construction
- Use QuorumPublicKey type as encrypt_to_server_target arg
- Update docs; client encryption helper fn
- rename BlobEnclaveEncryptServerRecv to ReusableEnclaveEncryptServerRecv

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
