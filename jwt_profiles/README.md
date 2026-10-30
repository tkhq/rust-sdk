# `turnkey_jwt_profiles`

Verifiers for Turnkey JWT profiles. Each profile is a module behind a cargo
feature. The shared JWS and key checks live in
[`turnkey_jwt_verify`](https://docs.rs/turnkey_jwt_verify).

## http-authz/v1

Verify Turnkey http-authz/v1 request-bound JWTs in a Rust HTTP service. This
profile is the `http_authz` module, behind the `http-authz` feature (on by
default). Its module documentation has an example.
