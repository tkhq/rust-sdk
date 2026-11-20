# TVC proof fixtures

These files support the inline tests in [tvc.rs](../../src/tvc.rs). They
exercise signed-payload compatibility and the verification chain without a
running Signer or AWS Nitro enclave. Test helpers and synthetic trust roots are
available only in test builds.

## Request-bound app proofs

[tvc_request_bound_proofs.json](tvc_request_bound_proofs.json) contains two
address-derivation app proofs and one quorum-key-creation app proof in the
producer's wire format.

`actual_tkms_claim_bytes_decode_and_verify_under_their_fixture_ephemeral_key`
checks that their original payload bytes verify under the included ephemeral
public key and decode into supported claims. Appending whitespace to a signed
payload must invalidate its signature.

This is a payload-format and signature compatibility test. These samples alone
do not establish a trusted boot chain or match a caller's submission receipt.

## Boot endorsement vector

[boot_endorsement_v1.json](boot_endorsement_v1.json) contains an attestation
document, the endorsement message, the fixture Signer quorum public key, and
positive and negative signatures.

`real_signer_endorsement_vector_fixes_purpose_bytes_and_raw_signature_encoding`
checks agreement on the exact signed message and signature encoding. It rejects
document tampering and the alternative signatures in the vector, including a
missing prefix, missing terminating NUL, extra hashing, a different signed
operand, and DER signature encoding.

This test checks the quorum-key endorsement contract. It does not validate the
document's AWS certificate chain or authenticate a live Signer instance.

## Synthetic attestation certificates

[synthetic-attestation/root.der](synthetic-attestation/root.der) is a test CA
certificate. [leaf.der](synthetic-attestation/leaf.der) is its attestation
signing certificate. [generate.py](synthetic-attestation/generate.py) recreates
these certificates using controlled P-384 keys and requires Python's
`cryptography` package. The keys are deliberately known test data. These are
not AWS certificates.

The `fixture` module in [tvc.rs](../../src/tvc.rs) uses them to construct signed
attestation documents, manifests, boot endorsements, and boot proofs. Its
test-only verifier selects this synthetic root explicitly; production
verification continues to use the AWS root and built-in Signer pins.

Tests using this chain include:

- The [endorsement unit tests](../../src/tvc/endorsement.rs) cover the Signer
  signature, the certificate chain at the signed attestation time, and how the
  manifest, PCRs, and ephemeral key must match the attestation.
- `application_signature_covers_original_payload_and_both_ephemeral_keys`
  rejects changes to the payload, signature, or either half of the composite
  ephemeral key.
- `boot_reuse_is_local_and_never_skips_an_application_signature` checks that
  reusing a verified boot does not skip another app-proof signature or carry
  verification across organizations.

The acceptance tests also use this chain to exercise receipt matching, returned
key substitution, quorum recipient ordering, and partial facts on failure.
They demonstrate verifier behavior under controlled inputs, not live AWS
attestation or production deployment readiness.

Run the TVC verification tests from the repository root:

```sh
cargo test -p turnkey_proofs tvc::
```
