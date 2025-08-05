# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.1.1](https://github.com/tkhq/rust-sdk/compare/turnkey_client-v0.0.2...turnkey_client-v0.1.1) - 2025-08-05

### Added

- add reqwest features

### Fixed

- add http2 to default-features, rename http2 custom features to native-tls, update README
- also enable vendored

### Other

- update README.md with reqwest features
- Add method summary and description to generated client methods
- Re-export TurnkeyP256ApiKey from client
- Add changelogs to published crates

* Re-export `turnkey_api_key_stamper::TurnkeyP256ApiKey` from `turnkey_client`
* Add summary and description to generated method docs

## turnkey_client-v0.0.2

Initial release of this crate.
