# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust.

Unlike other languages ([Typescript](https://github.com/tkhq/sdk), [Ruby](https://github.com/tkhq/ruby-sdk)), we do not yet offer a full SDK for Rust.

If you are working on a project in Rust and would benefit from a Rust SDK please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.

The main challenge when making requests to the Turnkey API is [request stamping](https://docs.turnkey.com/api-design/stamps). This repo includes a stamper for reference, credits go to @luca992!

==> [`lib.rs`](./src/lib.rs)
