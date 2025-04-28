# `examples`

This crate contains example programs to interact with Turnkey.

* [`whoami.rs`](./src/bin/whoami.rs) shows how to authenticate to the Turnkey API for the simplest endpoint we have
* [`create_sub_organization.rs`](./src/bin/create_sub_organization.rs) shows how to post an activity to the Turnkey API


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
cargo run -p tkhq_examples --bin create_sub_organization
```
