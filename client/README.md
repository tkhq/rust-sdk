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
        external_signer_pubkey: None,
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

The built-in P-256 and secp256k1 keys also support `create_tvc_operator_with_receipt`
and `create_tvc_quorum_key_with_receipt`. The returned `SubmittedActivityResult`
exposes `activity_result()` and `receipt().sent_fingerprints()`. These fingerprints
identify this invocation's fresh POST submissions, including Pending resubmissions.
They are local submission metadata. Output proofs still need independent verification.
Custom `Stamp` implementations continue to use the ordinary methods.

Receipt failures return `TurnkeyClientError::SubmissionFailed`. Inspect its
`submission_state()` before deciding whether creation could have happened.
`MayHaveSubmitted` includes a Pending response followed by a later stamp failure.
Its optional `activity_id()` is an untrusted routing label, not accepted output.
The error cause excludes request, stamp, and response contents. Dropping an
in-progress future yields no outcome and must not justify recreating an activity.

## Release compatibility

The receipt APIs add `TurnkeyClientError::InvalidReceiptStamp` and
`TurnkeyClientError::SubmissionFailed`. Downstream exhaustive matches on
`TurnkeyClientError` must handle both variants, including
callers that use only the ordinary methods. This change requires a breaking
release, not a compatible patch release. The new stamper error variants have
the same requirement. See [stamper release compatibility](../api_key_stamper/README.md#release-compatibility).

### Generated proof schema source compatibility

The request-bound proof schema adds a field and enum variants. These compiler
fixtures preserve evidence that older exhaustive source forms require changes.

```compile_fail
use turnkey_client::generated::external::data::v1::{AppProofPayload, AppProofType};
let old_literal = AppProofPayload {
    r#type: AppProofType::AddressDerivation,
    timestamp_ms: "100".into(),
    proof_payload: None,
};
```

```compile_fail
use turnkey_client::generated::external::data::v1::AppProofType;
fn old_match(kind: AppProofType) {
    match kind {
        AppProofType::Unspecified => (),
        AppProofType::AddressDerivation => (),
        AppProofType::PolicyOutcome => (),
    }
}
```
