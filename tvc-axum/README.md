# tvc-axum

Axum middleware for Turnkey Verifiable Cloud applications.

`ResponseSigningLayer` signs every HTTP response with the enclave's ephemeral
key (and optionally the quorum key) using RFC 9421 HTTP Message Signatures
and an RFC 9530 `Content-Digest`, and attaches the NSM attestation document
and QOS manifest envelope as response headers so clients can verify the full
trust chain: manifest → attestation document → response signature.

## Response headers

| Header | Contents |
| --- | --- |
| `Content-Digest` | `sha-256=:BASE64(sha256(body)):` (RFC 9530) |
| `Signature-Input` | `ephemeral=("@method" "@path" "@status" "content-digest");created=UNIX;keyid="ephemeral";alg="ecdsa-p256-sha256"` plus a `quorum=(...)` entry when a quorum key is configured (RFC 9421) |
| `Signature` | `ephemeral=:BASE64:` plus `quorum=:BASE64:` when configured (RFC 9421) |
| `x-tvc-attestation-doc` | base64 NSM attestation document; `user_data` = manifest hash, `public_key` = ephemeral public key |
| `x-tvc-manifest-envelope` | base64 canonical manifest envelope bytes (`VersionedManifestEnvelope::to_storage_vec`) |

The ephemeral public key is intentionally never sent as its own header:
verifiers must extract it from the attestation document.

## Usage

```rust,ignore
let layer = ResponseSigningLayer::builder()
    .ephemeral_key(ephemeral_pair)   // Arc<qos_p256::P256Pair>, required
    .quorum_key(quorum_pair)         // optional
    .nsm(nsm)                        // Arc<dyn qos_nsm::NsmProvider + Send + Sync>, required
    .manifest_envelope(envelope)     // qos_core VersionedManifestEnvelope, required
    .build()?;

let app = Router::new().route("/hello", get(hello)).layer(layer);
```

Use `QosJson<T>` in handlers to serialize response bodies with `qos_json`
canonical JSON.

See `tests/response_signing.rs` for a complete verification walkthrough
against an NSM.
