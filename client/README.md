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

The built-in P-256 and secp256k1 keys support
`create_tvc_operator_with_receipt` and `create_tvc_quorum_key_with_receipt`.
Both require an async callback from `&SubmissionCheckpoint` to
`Result<(), CheckpointSaveError>`. Durably persist the checkpoint before returning
`Ok(())`. A failed save forbids that POST. The SDK owns no storage paths.
Custom stampers continue to use ordinary methods. Built-in keys use fresh OS
randomness for every stamp, including ordinary methods.

`SubmissionOutcome::Completed` contains the typed result and an unverified receipt.
`Pending { checkpoint, reason }` retains origin evidence for processing, consensus,
or authenticator approval. Persist the returned checkpoint to save updated lookup
hints. `receipt().sent_fingerprints()` delegates to the checkpoint's original
signed creation attempts. Prepared evidence does not establish delivery, and
TKMS output proofs still require independent verification.

After restart, deserialize only caller-owned checkpoint JSON and call
`resume_submission(checkpoint, candidate_id)`. It queries GetActivity once using
the original organization and never resubmits creation or adds query fingerprints.
An explicit candidate ID can replace a missing hint. Both IDs remain untrusted,
and a completed receipt still requires independent proof acceptance.

`TurnkeyClientError::SubmissionFailed` exposes sanitized causes,
`submission_state()`, and an optional `checkpoint()` for recovery. A failed first
save is `NeverSubmitted`; failure after an earlier send is `MayHaveSubmitted` and
retains the earlier checkpoint. Cancellation returns no outcome but leaves the
last persisted evidence usable. A callback that writes and then fails or is
canceled can leave an extra prepared, unsent fingerprint on disk. Save updated
hints from errors when available. Do not infer creation failure from missing hints.

## Release compatibility

The receipt APIs add `InvalidReceiptStamp`, `CheckpointSaveFailed`,
`InvalidSubmissionActivity`, and `SubmissionFailed` to `TurnkeyClientError`.
Downstream exhaustive matches must handle these variants, including callers of
ordinary methods. This change and the new stamper error variant require a
breaking release. Receipt methods require the persistence callback and return
`SubmissionOutcome` instead of a completion-only result.

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
