# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust, and is organized as a collection of Rust crates.

The two user-facing crates are:
* [`client`](./client/README.md): fully typed client to send requests to Turnkey
* [`api_key_stamper`](./api_key_stamper/README.md): used by `client` to stamp requests before they're sent to Turnkey
* [`enclave_encrypt`](./enclave_encrypt/README.md): used in the context of features which leverage enclave [secure channels](https://docs.turnkey.com/security/enclave-secure-channels) ([Social Logins](https://docs.turnkey.com/authentication/social-logins), [Export](https://docs.turnkey.com/wallets/export-wallets), [Import](https://docs.turnkey.com/wallets/import-wallets))

## Examples

For fully working code examples, see our ['examples/' folder](./examples/README.md).

## Development

This project uses `make` to encapsulate common tasks:
* `make lint` will run rust fmt and clippy
* `make test` runs the unit tests
* `make examples` runs the examples (requires a local `.env` file, see [instructions](./examples/README.md))
* `make generate` re-generates `client::generated` from the `proto` folder

## Releases

This project uses [`cargo-release`](https://github.com/crate-ci/cargo-release). Install it with:
```
cargo install cargo-release
```

## Related projects

There is another Rust client available, maintained by [@Eliascm17](https://github.com/Eliascm17): [Eliascm17/turnkey](https://github.com/Eliascm17/turnkey). This client offers some structure around API requests/responses on top of bare request signing.

## Feature requests and support

If you are working on a project in Rust and would benefit from improvements to this SDK, please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.
