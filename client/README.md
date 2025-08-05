# `turnkey_client`

This crate contains an HTTP client to interact with the Turnkey API ([documentation](https://docs.turnkey.com/api-reference/overview)).

## Usage

To make a request to Turnkey:

```rust,no_run
use turnkey_client::generated::SignRawPayloadIntentV2;
use turnkey_client::generated::immutable::common::v1::HashFunction;
use turnkey_client::generated::immutable::common::v1::PayloadEncoding;

// You can load your API key from a file or from env
let api_key = turnkey_client::TurnkeyP256ApiKey::from_strings("<private key hex>", None).expect("api key creation failed");

// Create a new client:
let client = turnkey_client::TurnkeyClient::builder().api_key(api_key).build().expect("client builder failed");

// Make a request (for example, a signature request)
let request = client.sign_raw_payload(
    "your-turnkey-organization-id".to_string(),
    client.current_timestamp(),
    SignRawPayloadIntentV2 {
        sign_with: "0x123456".to_string(), // Turnkey address
        payload: "hello from TKHQ".to_string(),
        encoding: PayloadEncoding::TextUtf8,
        hash_function: HashFunction::Keccak256, // assuming ETH
    },
);

// You can then call `request.await?` to get the signature result
```

## Advanced usage

The Turnkey client uses `reqwest` under the hood. To access the `reqwest` builder, use the following:
```rust
let api_key = turnkey_client::TurnkeyP256ApiKey::generate();

let client = turnkey_client::TurnkeyClient::builder()
    .api_key(api_key)
    .with_reqwest_builder(|b| b.connection_verbose(true));
```

Additional `reqwest` features can be enabled in your `Cargo.toml`:
```toml
[dependencies]
turnkey_client = { version = "0.0.2", features = ["reqwest_native_tls", "reqwest_brotli", "reqwest_zstd", "reqwest_hickory_dns"] }
```
