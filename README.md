# Turnkey Rust SDK &emsp; [![Build Status]][actions] 

[Build Status]: https://img.shields.io/github/actions/workflow/status/tkhq/rust-sdk/main.yml?branch=main
[actions]: https://github.com/tkhq/rust-sdk/actions?query=branch%3Amain

This repository contains tooling to interact with the Turnkey API using Rust, and is organized as a collection of Rust crates.

| Crate | Description | crates.io | Docs | Changelog | Source |
| ----- | ----------- | --------- | -----| ----------| ------ |
| `turnkey_api_key_stamper` | Utilities to use Turnkey API keys | [![Latest](https://img.shields.io/crates/v/turnkey_api_key_stamper.svg)](https://crates.io/crates/turnkey_api_key_stamper) | [![docs.rs](https://img.shields.io/docsrs/turnkey_api_key_stamper)](https://docs.rs/turnkey_api_key_stamper) | [CHANGELOG](./api_key_stamper/CHANGELOG.md) | [`api_key_stamper`](./api_key_stamper/) |
| `turnkey_client` | Rust client to interact with the Turnkey API | [![Latest](https://img.shields.io/crates/v/turnkey_client.svg)](https://crates.io/crates/turnkey_client) | [![docs.rs](https://img.shields.io/docsrs/turnkey_client)](https://docs.rs/turnkey_client) | [CHANGELOG](./client/CHANGELOG.md) | [`client`](./client/) |
| `turnkey_enclave_encrypt` | Utilities to decrypt and encrypt data from and to Turnkey secure enclaves | [![Latest](https://img.shields.io/crates/v/turnkey_enclave_encrypt.svg)](https://crates.io/crates/turnkey_enclave_encrypt) | [![docs.rs](https://img.shields.io/docsrs/turnkey_enclave_encrypt)](https://docs.rs/turnkey_enclave_encrypt) | [CHANGELOG](./enclave_encrypt/CHANGELOG.md) | [`enclave_encrypt`](./enclave_encrypt/) |
| `turnkey_proofs` | Utilities to verify Turnkey secure enclave proofs | [![Latest](https://img.shields.io/crates/v/turnkey_proofs.svg)](https://crates.io/crates/turnkey_proofs) | [![docs.rs](https://img.shields.io/docsrs/turnkey_proofs)](https://docs.rs/turnkey_proofs) | [CHANGELOG](./proofs/CHANGELOG.md) | [`proofs`](./proofs/) |


## Examples

For fully working code examples, see our [`examples folder`](./examples/README.md).

## Development

This project uses `make` to encapsulate common tasks:
* `make lint` will run rust fmt and clippy
* `make test` runs the unit tests
* `make examples` runs the [examples](./examples/) (requires a local `.env` file, see [instructions](./examples/README.md))
* `make generate` re-generates `client::generated` from the [`proto`](./proto/) folder. Code generation logic lives in [`codegen`](./codegen/)

## Releases

This project uses [`release-plz`](https://github.com/release-plz/release-plz). Install it with:

```sh
cargo install --locked release-plz
```

Once you have it installed you can try a release locally, to see what the release PR would be:

```
release-plz update
```

## Feature requests and support

If you are working on a project in Rust and would benefit from improvements to this SDK, please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.
