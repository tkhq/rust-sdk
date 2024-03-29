# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust.

Unlike other languages ([Typescript](https://github.com/tkhq/sdk), [Ruby](https://github.com/tkhq/ruby-sdk)), we do not yet offer a full SDK for Rust.

If you are working on a project in Rust and would benefit from a Rust SDK please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.

The main challenge when making requests to the Turnkey API is [request stamping](https://docs.turnkey.com/api-design/stamps). This repo includes code to sign API requests. Credits go to [@luca992](https://github.com/luca992) for the initial snippet!

==> [`lib.rs`](./src/lib.rs)

There is another Rust client available, maintained by [@Eliascm17](https://github.com/Eliascm17): [Eliascm17/turnkey](https://github.com/Eliascm17/turnkey). This client offers some structure around API requests/responses on top of bare request signing.
