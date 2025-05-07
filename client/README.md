# Turnkey Client

## Usage

To make a request to Turnkey:
* Load an API key:
  ```rust
  // You can load your API key from a file or from env
  let api_key = TurnkeyP256ApiKey::from_strings(private_key: "<private key hex>", None).expect("api key creation failed");
  ```
* Create a new client:
  ```rust
  let client = tkhq_client::TurnkeyClient::builder().api_key(api_key).build().expect("client builder failed");
  ```
* Make a request (for example, a signature request)
  ```rust
  let signature_result = client.sign_raw_payload(
    organization_id, // your organization ID
    client.current_timestamp(),
    SignRawPayloadIntentV2 {
        sign_with: address, // any Turnkey-generated address
        payload: "hello from TKHQ".to_string(),
        encoding: PayloadEncoding::TextUtf8,
        hash_function: HashFunction::Keccak256,
    },
  ).await;
  ```

## Advanced usage

The Turnkey client uses `reqwest` under the hood. To access the `reqwest` builder, use the following:
```rust
tkhq_client::TurnkeyClient::builder()
    .api_key(api_key)
    .with_reqwest_builder(|b| b.connection_verbose(true))
```
