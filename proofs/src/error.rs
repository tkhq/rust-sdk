//! Attestation specific logic

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use crate::types;

/// Attestation error.
#[derive(Debug)]
pub enum AttestError {
    /// `webpki::Error` wrapper.
    WebPki(webpki::Error),
    /// Invalid certificate chain.
    InvalidCertChain(webpki::Error),
    /// `aws_nitro_enclaves_nsm_api::api::Error` wrapper.
    Nsm(aws_nitro_enclaves_nsm_api::api::Error),
    /// Invalid end entity certificate. In the case of Nitro this means the
    /// NSM's certificate was invalid.
    InvalidEndEntityCert,
    /// Invalid COSE Sign1 structure signature. In the case of Nitro this means
    /// the end entitys signature of the attestation doc was invalid.
    InvalidCOSESign1Signature,
    /// Invalid COSE Sign1 structure.
    InvalidCOSESign1Structure,
    /// Invalid hash digest.
    InvalidDigest,
    /// Invalid NSM module id.
    InvalidModuleId,
    /// Invalid PCR.
    InvalidPcr,
    /// Invalid certificate authority bundle.
    InvalidCABundle,
    /// Invalid time.
    InvalidTimeStamp,
    /// Invalid public key.
    InvalidPubKey,
    /// Invalid bytes.
    InvalidBytes,
    /// The NSM returned an unexpected response when queried
    UnexpectedNsmResponse(types::NsmResponse),
    /// Error while decoding the attestation document (base64 encoded).
    Base64DecodingError(String),
    /// Error while decoding PEM.
    PemDecodingError,
    /// Error trying to decode the public key in a cert.
    FailedDecodeKeyFromCert,
    /// Error while trying to parse a cert.
    FailedToParseCert,
    /// User data is missing in the attestation doc.
    MissingUserData,
    /// User data (normally manifest hash) does not match the attestation doc.
    DifferentUserData,
    /// The attestation doc has a nonce when none was expected.
    UnexpectedAttestationDocNonce,
    /// The attestation doc does not contain a pcr0.
    MissingPcr0,
    /// The pcr3 in the attestation doc does not match.
    DifferentPcr0,
    /// The attestation doc does not have a pcr1.
    MissingPcr1,
    /// The attestation doc has a different pcr1.
    DifferentPcr1,
    /// The attestation doc does not have a pcr2.
    MissingPcr2,
    /// The attestation doc has a different pcr2.
    DifferentPcr2,
    /// The attestation doc does not have a pcr3.
    MissingPcr3,
    /// The attestation doc has a different pcr3.
    DifferentPcr3,
}

/// `AppProof` error.
#[derive(Debug)]
pub enum AppProofError {
    /// Invalid signature scheme
    InvalidSignatureScheme,
    /// Invalid public key.
    InvalidPublicKey(String),
    /// Invalid signing public key.
    InvalidSigningPublicKeyBytes(String),
    /// Failed signature verification.
    FailedSignatureVerification(String),
    /// Invalid signature.
    InvalidSignature(String),
    /// Invalid proof payload.
    InvalidProofPayload(String),
    /// Missing timestamp.
    MissingTimestamp,
    /// Invalid timestamp.
    InvalidTimestamp(String),
}

/// `BootProofError`
#[derive(Debug)]
pub enum BootProofError {
    /// Invalid timestamp.
    InvalidTimestamp(String),
    /// Missing timestamp.
    MissingTimestamp,
}

/// Verify error.
#[derive(Debug)]
pub enum VerifyError {
    /// Invalid app proof.
    InvalidAppProof(String),
    /// Invalid attestation doc.
    InvalidAttestation(String),
    /// Invalid boot proof.
    InvalidBootProof(String),
    /// Ephemeral keys did not match.
    DifferentEphemeralKey(String),
    /// Manifest digests did not match.
    DifferentManifest(String),
}

impl From<webpki::Error> for AttestError {
    fn from(e: webpki::Error) -> Self {
        Self::WebPki(e)
    }
}

impl From<aws_nitro_enclaves_nsm_api::api::Error> for AttestError {
    fn from(e: aws_nitro_enclaves_nsm_api::api::Error) -> Self {
        Self::Nsm(e)
    }
}

impl std::fmt::Display for AttestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for AppProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::fmt::Display for BootProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
