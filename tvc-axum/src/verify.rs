//! Client-side verification of responses signed by
//! [`ResponseSigningLayer`](crate::ResponseSigningLayer).
//!
//! [`verify_quorum_signature`] checks the `quorum` response signature with a
//! known quorum public key. [`verify_response_trust_chain`] verifies the
//! attestation document and manifest envelope headers with
//! [`verify_attestation_and_manifest`] and then checks the `ephemeral`
//! response signature with the attested ephemeral key.

use axum::http::StatusCode;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use qos_core::protocol::services::boot::verify_attestation_and_manifest;
pub use qos_core::protocol::services::boot::{
    ManifestCommitmentKind, VerificationExpectations, VerifyError,
};
use qos_p256::P256Public;

use crate::{ATTESTATION_DOC_HEADER, MANIFEST_ENVELOPE_HEADER, content_digest, signature_base};

/// Signature label used for the ephemeral key.
const EPHEMERAL_LABEL: &str = "ephemeral";
/// Signature label used for the quorum key.
const QUORUM_LABEL: &str = "quorum";

/// Errors from verifying a signed response.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum VerifyResponseError {
    /// The `signature-input` header has no entry for this signature label.
    #[error("signature-input header has no `{0}` entry")]
    MissingSignatureInput(&'static str),
    /// The `signature` header has no entry for this signature label.
    #[error("signature header has no `{0}` entry")]
    MissingSignature(&'static str),
    /// The signature entry is not a base64 byte sequence (`:base64:`).
    #[error("`{0}` signature is not a base64 byte sequence")]
    MalformedSignature(&'static str),
    /// The signature does not verify over the reconstructed signature base.
    #[error("`{0}` signature verification failed")]
    InvalidSignature(&'static str),
    /// A trust chain header value is not valid base64.
    #[error("`{0}` header is not valid base64")]
    HeaderDecode(&'static str),
    /// The attestation document and manifest envelope failed verification.
    #[error(transparent)]
    TrustChain(#[from] VerifyError),
}

/// The parts of a signed response its RFC 9421 signatures cover.
///
/// The body is carried instead of the `content-digest` header value: the
/// digest is recomputed from the body during verification, so a signature
/// only verifies for the exact body the enclave signed.
#[derive(Debug, Clone, Copy)]
pub struct SignedResponseParts<'a> {
    /// Method of the request the response answers.
    pub method: &'a str,
    /// Path of the request the response answers, excluding the query string.
    pub path: &'a str,
    /// Response status code.
    pub status: StatusCode,
    /// Response body bytes.
    pub body: &'a [u8],
    /// Value of the `signature-input` response header.
    pub signature_input: &'a str,
    /// Value of the `signature` response header.
    pub signature: &'a str,
}

fn label_value<'a>(header: &'a str, label: &str) -> Option<&'a str> {
    header
        .split(',')
        .map(str::trim)
        .find_map(|entry| entry.strip_prefix(label)?.strip_prefix('='))
}

/// Reconstruct the RFC 9421 signature base for `label` over `@method`,
/// `@path`, `@status` and `content-digest` — recomputing the RFC 9530
/// content digest from the body — and verify the label's `signature` header
/// entry over it with `key`.
fn verify_signature(
    parts: &SignedResponseParts<'_>,
    label: &'static str,
    key: &P256Public,
) -> Result<(), VerifyResponseError> {
    let params = label_value(parts.signature_input, label)
        .ok_or(VerifyResponseError::MissingSignatureInput(label))?;
    let signature =
        label_value(parts.signature, label).ok_or(VerifyResponseError::MissingSignature(label))?;
    let signature = signature
        .strip_prefix(':')
        .and_then(|value| value.strip_suffix(':'))
        .and_then(|value| STANDARD.decode(value).ok())
        .ok_or(VerifyResponseError::MalformedSignature(label))?;
    let digest = content_digest(parts.body);
    let base = signature_base(parts.method, parts.path, parts.status, &digest, params);
    key.verify(&base, &signature)
        .map_err(|_| VerifyResponseError::InvalidSignature(label))
}

/// Verify the `quorum` signature of a signed response with the quorum public
/// key.
///
/// This proves the response status and body — bound to the request method
/// and path — were signed by the quorum key. It does not check the
/// attestation trust chain; see [`verify_response_trust_chain`].
///
/// # Errors
///
/// Returns a [`VerifyResponseError`] if the response has no `quorum`
/// signature or the signature does not verify.
pub fn verify_quorum_signature(
    parts: &SignedResponseParts<'_>,
    quorum_key: &P256Public,
) -> Result<(), VerifyResponseError> {
    verify_signature(parts, QUORUM_LABEL, quorum_key)
}

/// Verify the full trust chain of a signed response and return the enclave's
/// ephemeral public key.
///
/// * `attestation_doc` / `manifest_envelope` - base64 values of the
///   [`ATTESTATION_DOC_HEADER`] and [`MANIFEST_ENVELOPE_HEADER`] response
///   headers.
/// * `root_ca`, `validation_time_secs`, `commitment_kind` and `expectations`
///   are passed through to [`verify_attestation_and_manifest`], which checks
///   the attestation document up to `root_ca`, the manifest against
///   `expectations`, the manifest-set approvals, and the manifest commitment
///   PCR, and returns the attested ephemeral key.
///
/// The `ephemeral` response signature is then verified with the attested
/// ephemeral key, exactly like [`verify_quorum_signature`] verifies the
/// `quorum` one, and the key is returned.
///
/// # Errors
///
/// Returns a [`VerifyResponseError`] if a header does not decode, the trust
/// chain does not verify, or the response signature is missing or invalid.
pub fn verify_response_trust_chain(
    parts: &SignedResponseParts<'_>,
    attestation_doc: &str,
    manifest_envelope: &str,
    root_ca: &[u8],
    validation_time_secs: u64,
    commitment_kind: ManifestCommitmentKind,
    expectations: &VerificationExpectations,
) -> Result<P256Public, VerifyResponseError> {
    let attestation_doc = STANDARD
        .decode(attestation_doc)
        .map_err(|_| VerifyResponseError::HeaderDecode(ATTESTATION_DOC_HEADER))?;
    let manifest_envelope = STANDARD
        .decode(manifest_envelope)
        .map_err(|_| VerifyResponseError::HeaderDecode(MANIFEST_ENVELOPE_HEADER))?;
    let ephemeral_key = verify_attestation_and_manifest(
        &attestation_doc,
        &manifest_envelope,
        root_ca,
        validation_time_secs,
        commitment_kind,
        expectations,
    )?;
    verify_signature(parts, EPHEMERAL_LABEL, &ephemeral_key)?;
    Ok(ephemeral_key)
}
