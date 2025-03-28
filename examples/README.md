# `examples`

This crate contains example programs to interact with Turnkey.

* [`whoami.rs`](./src/bin/whoami.rs) shows how to authenticate to the Turnkey API for the simplest endpoint we have
* [`create_sub_organization.rs`](./src/bin/create_sub_organization.rs) shows how to post an activity to the Turnkey API

To run them: `cargo run -p tkhq_examples --bin whoami` (or `cargo run -p tkhq_examples --bin create_sub_organization`)
