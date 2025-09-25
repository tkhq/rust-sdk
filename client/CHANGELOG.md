# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.3.0...turnkey_client-v0.4.0) - 2025-09-25

### Other

- sync sdk protos with internal mono repo
- Remove generated private.ump.v1.rs
- Update protos and run gen script

## [0.3.0](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.2.0...turnkey_client-v0.3.0) - 2025-08-28

### Added

- https://github.com/tkhq/rust-sdk/pull/50: Sync proto folder with latest. New client functions:
  - `oauth_login`, `stamp_login`, `otp_login`: authentication methods to resolve sub-organization ID and establish a session.
  - `init_otp`, `verify_otp`: new generic OTP endpoints to init and verify an email or phone number. The result is a verification token that can be used with login endpoints above.
  - `update_user_name`, `update_user_email`, `update_user_phone_number`: user management utilities
  - `create_smart_contract_interface`, `delete_smart_contract_interface`, `get_smart_contract_interface`, `get_smart_contract_interfaces`: management of Ethereum ABI and Solana IDLs (see https://docs.turnkey.com/concepts/policies/smart-contract-interfaces#using-abis-and-idls-to-control-transaction-signing)
  - `init_fiat_on_ramp`: related to on-ramp APIs (https://docs.turnkey.com/products/embedded-wallets/features/fiat-on-ramp)
  - `get_policy_evaluations`: get evaluation traces for activities
  - `create_oauth2_credential`, `update_oauth2_credential`, `delete_oauth2_credential`, `list_oauth2_credentials`, `get_oauth2_credential`, `oauth2_authenticate`: related to the upcoming OAuth2.0 support

## [0.2.0](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.1.1...turnkey_client-v0.2.0) - 2025-08-15

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
