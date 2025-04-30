# `examples`

This crate contains example programs to interact with Turnkey.

* [`whoami.rs`](./src/bin/whoami.rs) shows how to authenticate to the Turnkey API for the simplest endpoint we have.
* [`sub_organization.rs`](./src/bin/sub_organization.rs) shows the creation and deletion of sub-organizations.
* [`wallet.rs`](./src/bin/wallet.rs) shows basic wallet management (creation, signature, deletion).


## Running examples

Copy the example `.env.example` into a `.env` file:
```
cp .env.example .env
```

Populate your `.env` file with your own organization ID, public key and private key. You can follow the instructions from our [quickstart](https://docs.turnkey.com/getting-started/quickstart).

Run the examples with:
```
cargo run -p tkhq_examples --bin
```

or

```
cargo run -p tkhq_examples --bin sub_organization
```
