# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust.

Unlike other languages ([Typescript](https://github.com/tkhq/sdk), [Ruby](https://github.com/tkhq/ruby-sdk)), we do not yet offer a full SDK to make API requests to Turnkey using Rust.

The main challenge when making requests to the Turnkey API is [request stamping](https://docs.turnkey.com/api-design/stamps). This repository includes code to sign API requests. Credits go to [@luca992](https://github.com/luca992) for the initial snippet!

==> [`api_key_stamper`](./src/lib.rs)

## Usage

To make a request to Turnkey:
* use any HTTP library ([`reqwest`](https://docs.rs/reqwest/latest/reqwest/) is a popular choice)
* look at our [API reference](https://docs.turnkey.com/api-reference/overview) to learn about the JSON schema of our requests and responses
* create a POST request body (all Turnkey API requests are POST requests). You can use [`serde_json`](https://docs.rs/serde_json/latest/serde_json/) to help with JSON serialization.
* use the the provided `stamp` function to sign your request body with a `TurnkeyApiKey`
* add the stamp value (bytes) inside of a new `X-Stamp` header, and attach it to the request
* POST the request, read the response, and use the activity result

## Examples

For fully working examples, see our ['examples/' folder](./examples/).

## Related projects

There is another Rust client available, maintained by [@Eliascm17](https://github.com/Eliascm17): [Eliascm17/turnkey](https://github.com/Eliascm17/turnkey). This client offers some structure around API requests/responses on top of bare request signing.

## Feature requests and support

If you are working on a project in Rust and would benefit from a Rust SDK please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.
