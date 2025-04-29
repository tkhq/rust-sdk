# Turnkey Rust SDK

This repository contains tooling to interact with the Turnkey API using Rust, and is organized as a collection of Rust crates.

The two user-facing crates are:
* [`client`](./client/): fully typed client to send requests to Turnkey
* [`api_key_stamper`](./api_key_stamper/): used by `client` to stamp requests before they're sent to Turnkey

## Usage

To make a request to Turnkey:
* Load an API key:
  ```rust
  let api_key
  ```
* Create a new client:
  ```rust
  let client = tkhq_client::TurnkeyClient::new("https://api.turnkey.com", api_key, RetryConfig::default());
  ```
* Make a request (for example, a signature request)
  ```rust
  let signature_result = client.sign_raw_payload(
    organization_id, // your organization ID
    timestamp_ms, // time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_millis();
    SignRawPayloadIntentV2 {
        sign_with: address, // any Turnkey-generated address
        payload: "hello from TKHQ".to_string(),
        encoding: PayloadEncoding::TextUtf8,
        hash_function: HashFunction::Keccak256,
    },
  ).await;
  ```

## Examples

For fully working examples, see our ['examples/' folder](./examples/README.md).

## Development

This project uses `make` to encapsulate common tasks:
* `make lint` will run rust fmt and clippy
* `make test` runs the unit tests
* `make examples` runs the examples
* `make generate` re-generates `client::generated` from the `proto` folder

## Related projects

There is another Rust client available, maintained by [@Eliascm17](https://github.com/Eliascm17): [Eliascm17/turnkey](https://github.com/Eliascm17/turnkey). This client offers some structure around API requests/responses on top of bare request signing.

## Feature requests and support

If you are working on a project in Rust and would benefit from improvements to this SDK, please open an issue or get in touch with us (hello@turnkey.com) and we can discuss prioritizing this.
