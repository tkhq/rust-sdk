# Changelog

Changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Initial release: the `http_authz` module (feature `http-authz`, on by
  default), a verifier for `turnkey.io/http-authz/v1` request-bound JWTs
  (the http-authz/v1 profile of `ACTIVITY_TYPE_MINT_JWT`), with request
  binding helpers and an optional Axum route layer (feature `axum`). The
  JWS, key, and time checks come from `turnkey_jwt_verify`.
- The Axum route layer returns a generic rejection body
  (`{"error": "unauthorized"}` for `401`, `{"error": "payload_too_large"}`
  for `413`) and logs the specific reason. `AuthzLayer::with_detailed_errors`
  turns on the specific code and message in the body for debugging.
  `AuthzLayer::with_max_body_bytes` sets the body limit (4096 bytes by
  default). The layer checks `exp` again after it buffers the body.
