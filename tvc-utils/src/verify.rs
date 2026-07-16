//! Verification of the Turnkey Verifiable Cloud trust chain: an NSM
//! attestation document and a QuorumOS manifest envelope, checked against
//! caller-supplied expected values.
//!
//! [`verify_attestation_and_manifest`] consumes the values a TVC app attaches
//! to its responses (see `tvc-axum`): the attestation document carried
//! base64-encoded in the `x-tvc-attestation-doc` header (DER COSE Sign1) and
//! the manifest envelope carried base64-encoded in the
//! `x-tvc-manifest-envelope` header (canonical storage encoding). Pass the
//! decoded bytes of each header. On success it returns the enclave's
//! ephemeral public key, which callers then use to verify response
//! signatures.

use qos_core::protocol::ProtocolError;
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use qos_nsm::nitro::{self, AttestError};
use qos_p256::{P256Error, P256Public};

/// Errors from [`verify_attestation_and_manifest`], one per failure class.
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    /// The manifest envelope bytes could not be decoded.
    #[error("failed to decode manifest envelope: {0}")]
    ManifestEnvelopeDecode(String),
    /// The manifest namespace name does not match the expected value.
    #[error("manifest namespace name mismatch: expected {expected}, got {actual}")]
    NamespaceNameMismatch {
        /// Expected namespace name.
        expected: String,
        /// Namespace name in the manifest.
        actual: String,
    },
    /// The manifest namespace nonce does not match the expected value.
    #[error("manifest namespace nonce mismatch: expected {expected}, got {actual}")]
    NonceMismatch {
        /// Expected namespace nonce.
        expected: u32,
        /// Namespace nonce in the manifest.
        actual: u32,
    },
    /// The manifest quorum key does not match the expected value.
    #[error("manifest quorum key mismatch: expected {expected}, got {actual}")]
    QuorumKeyMismatch {
        /// Expected quorum public key, hex encoded.
        expected: String,
        /// Quorum public key in the manifest, hex encoded.
        actual: String,
    },
    /// The manifest QOS commit does not match the expected value.
    #[error("manifest QOS commit mismatch: expected {expected}, got {actual}")]
    QosCommitMismatch {
        /// Expected QOS commit reference.
        expected: String,
        /// QOS commit reference in the manifest.
        actual: String,
    },
    /// A manifest PCR does not match the expected value.
    #[error("manifest PCR{index} mismatch: expected {expected}, got {actual}")]
    PcrMismatch {
        /// PCR register index (0 through 3).
        index: u8,
        /// Expected PCR value, hex encoded.
        expected: String,
        /// PCR value in the manifest, hex encoded.
        actual: String,
    },
    /// The manifest pivot (app binary) hash does not match the expected
    /// value.
    #[error("manifest pivot hash mismatch: expected {expected}, got {actual}")]
    PivotHashMismatch {
        /// Expected pivot hash, hex encoded.
        expected: String,
        /// Pivot hash in the manifest, hex encoded.
        actual: String,
    },
    /// The manifest hash does not match the expected value.
    #[error("manifest hash mismatch: expected {expected}, got {actual}")]
    ManifestHashMismatch {
        /// Expected manifest hash, hex encoded.
        expected: String,
        /// Hash of the manifest in the envelope, hex encoded.
        actual: String,
    },
    /// The attestation document carries no user data.
    #[error("attestation document is missing user data")]
    MissingUserData,
    /// The attestation document user data does not match the manifest hash.
    #[error(
        "attestation document user data does not match the manifest hash: \
         manifest {manifest_hash}, user data {user_data}"
    )]
    UserDataMismatch {
        /// Hash of the manifest in the envelope, hex encoded.
        manifest_hash: String,
        /// User data in the attestation document, hex encoded.
        user_data: String,
    },
    /// The attestation document is missing a PCR the manifest specifies.
    #[error("attestation document is missing PCR{index}")]
    MissingAttestationPcr {
        /// PCR register index (0 through 3).
        index: u8,
    },
    /// An attestation document PCR does not match the manifest.
    #[error(
        "attestation document PCR{index} does not match the manifest: \
         manifest {manifest}, attestation document {doc}"
    )]
    AttestationPcrMismatch {
        /// PCR register index (0 through 3).
        index: u8,
        /// PCR value in the manifest, hex encoded.
        manifest: String,
        /// PCR value in the attestation document, hex encoded.
        doc: String,
    },
    /// The attestation document could not be decoded or did not verify up to
    /// the root certificate authority.
    #[error("attestation document verification failed: {0}")]
    AttestationDoc(#[from] AttestError),
    /// The manifest-set approvals over the manifest hash did not verify.
    #[error("manifest set approval verification failed: {0}")]
    ManifestSetApprovals(#[from] ProtocolError),
    /// The attestation document carries no public key.
    #[error("attestation document is missing a public key")]
    MissingPublicKey,
    /// The attestation document public key is not a valid P-256 public key.
    #[error("failed to decode ephemeral public key: {0:?}")]
    EphemeralKeyDecode(P256Error),
}

/// Expected values to check the manifest against. All expectations are
/// optional: only supplied values are compared, and an empty set of
/// expectations still verifies the rest of the trust chain (manifest hash
/// binding, attestation document signature, and manifest-set approvals).
#[derive(Debug, Clone, Default)]
pub struct VerificationExpectations {
    namespace_name: Option<String>,
    nonce: Option<u32>,
    quorum_key: Option<Vec<u8>>,
    qos_commit: Option<String>,
    pcr0: Option<Vec<u8>>,
    pcr1: Option<Vec<u8>>,
    pcr2: Option<Vec<u8>>,
    pcr3: Option<Vec<u8>>,
    pivot_hash: Option<[u8; 32]>,
    manifest_hash: Option<[u8; 32]>,
}

impl VerificationExpectations {
    /// Create an empty set of expectations.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Expect the manifest namespace name.
    #[must_use]
    pub fn namespace_name(mut self, name: &str) -> Self {
        self.namespace_name = Some(name.to_string());
        self
    }

    /// Expect the manifest namespace nonce.
    #[must_use]
    pub fn nonce(mut self, nonce: u32) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Expect the manifest quorum public key bytes.
    #[must_use]
    pub fn quorum_key(mut self, quorum_key: Vec<u8>) -> Self {
        self.quorum_key = Some(quorum_key);
        self
    }

    /// Expect the manifest QOS commit reference.
    #[must_use]
    pub fn qos_commit(mut self, qos_commit: &str) -> Self {
        self.qos_commit = Some(qos_commit.to_string());
        self
    }

    /// Expect the manifest PCR0 value.
    #[must_use]
    pub fn pcr0(mut self, pcr0: Vec<u8>) -> Self {
        self.pcr0 = Some(pcr0);
        self
    }

    /// Expect the manifest PCR1 value.
    #[must_use]
    pub fn pcr1(mut self, pcr1: Vec<u8>) -> Self {
        self.pcr1 = Some(pcr1);
        self
    }

    /// Expect the manifest PCR2 value.
    #[must_use]
    pub fn pcr2(mut self, pcr2: Vec<u8>) -> Self {
        self.pcr2 = Some(pcr2);
        self
    }

    /// Expect the manifest PCR3 value.
    #[must_use]
    pub fn pcr3(mut self, pcr3: Vec<u8>) -> Self {
        self.pcr3 = Some(pcr3);
        self
    }

    /// Expect PCR0 through PCR3.
    #[must_use]
    pub fn pcrs(self, pcr0: Vec<u8>, pcr1: Vec<u8>, pcr2: Vec<u8>, pcr3: Vec<u8>) -> Self {
        self.pcr0(pcr0).pcr1(pcr1).pcr2(pcr2).pcr3(pcr3)
    }

    /// Expect the manifest pivot (app binary) hash.
    #[must_use]
    pub fn pivot_hash(mut self, pivot_hash: [u8; 32]) -> Self {
        self.pivot_hash = Some(pivot_hash);
        self
    }

    /// Expect the manifest hash.
    #[must_use]
    pub fn manifest_hash(mut self, manifest_hash: [u8; 32]) -> Self {
        self.manifest_hash = Some(manifest_hash);
        self
    }
}

/// Verify a TVC trust chain and return the enclave's ephemeral public key.
///
/// # Arguments
///
/// * `attestation_doc` - DER encoded COSE Sign1 attestation document, as
///   carried base64-encoded in the `x-tvc-attestation-doc` header.
/// * `manifest_envelope` - manifest envelope in its canonical storage
///   encoding, as carried base64-encoded in the `x-tvc-manifest-envelope`
///   header.
/// * `root_ca` - DER encoded root certificate the attestation document must
///   chain up to (the AWS Nitro root for real enclaves). Its authenticity
///   must be validated out of band.
/// * `validation_time_secs` - seconds since the unix epoch at which the
///   certificate chain must be valid, usually the current time.
/// * `expectations` - expected manifest values; only supplied values are
///   checked.
///
/// Verification steps:
///
/// 1. Compare the manifest against `expectations`.
/// 2. Compare the manifest against the attestation document: the manifest
///    hash must match the document's user data and the manifest PCR0-3 must
///    match the document's measurements.
/// 3. Verify the attestation document: COSE Sign1 signature and certificate
///    chain up to `root_ca` at `validation_time_secs`.
/// 4. Verify the manifest envelope carries a threshold of valid manifest-set
///    approvals over the manifest hash.
/// 5. Extract and return the ephemeral public key from the attestation
///    document's public key field.
///
/// Nothing is returned until every step passes; step 2 reads the attestation
/// document before step 3 has verified it, which only affects the error
/// reported for documents that fail both.
///
/// # Errors
///
/// Returns a [`VerifyError`] variant identifying the failed check.
pub fn verify_attestation_and_manifest(
    attestation_doc: &[u8],
    manifest_envelope: &[u8],
    root_ca: &[u8],
    validation_time_secs: u64,
    expectations: &VerificationExpectations,
) -> Result<P256Public, VerifyError> {
    let envelope = VersionedManifestEnvelope::try_from_slice_compat(manifest_envelope)
        .map_err(|e| VerifyError::ManifestEnvelopeDecode(e.to_string()))?;
    let manifest_hash = envelope.manifest_hash();
    let manifest = envelope.clone().manifest();

    // 1. Compare the manifest to the expected values.
    check_expectations(expectations, &manifest, &manifest_hash)?;

    // 2. Compare the manifest to the attestation document.
    let doc = nitro::unsafe_attestation_doc_from_der(attestation_doc)?;
    let user_data: &[u8] = doc
        .user_data
        .as_deref()
        .ok_or(VerifyError::MissingUserData)?;
    if user_data != manifest_hash {
        return Err(VerifyError::UserDataMismatch {
            manifest_hash: qos_hex::encode(&manifest_hash),
            user_data: qos_hex::encode(user_data),
        });
    }
    let enclave = manifest.enclave();
    for (index, manifest_pcr) in [
        (0u8, &enclave.pcr0),
        (1, &enclave.pcr1),
        (2, &enclave.pcr2),
        (3, &enclave.pcr3),
    ] {
        let doc_pcr = doc
            .pcrs
            .get(&usize::from(index))
            .ok_or(VerifyError::MissingAttestationPcr { index })?;
        if doc_pcr.as_slice() != manifest_pcr.as_slice() {
            return Err(VerifyError::AttestationPcrMismatch {
                index,
                manifest: qos_hex::encode(manifest_pcr),
                doc: qos_hex::encode(doc_pcr),
            });
        }
    }

    // 3. Verify the attestation document up to the root CA.
    let doc = nitro::attestation_doc_from_der(attestation_doc, root_ca, validation_time_secs)?;

    // 4. Verify the manifest-set approvals over the manifest hash.
    envelope.check_approvals()?;

    // 5. Return the ephemeral public key bound into the attestation document.
    let public_key = doc.public_key.ok_or(VerifyError::MissingPublicKey)?;
    P256Public::from_bytes(&public_key).map_err(VerifyError::EphemeralKeyDecode)
}

fn check_expectations(
    expectations: &VerificationExpectations,
    manifest: &qos_core::protocol::services::boot::VersionedManifest,
    manifest_hash: &[u8; 32],
) -> Result<(), VerifyError> {
    let namespace = manifest.namespace();
    if let Some(expected) = &expectations.namespace_name
        && expected != &namespace.name
    {
        return Err(VerifyError::NamespaceNameMismatch {
            expected: expected.clone(),
            actual: namespace.name.clone(),
        });
    }
    if let Some(expected) = expectations.nonce
        && expected != namespace.nonce
    {
        return Err(VerifyError::NonceMismatch {
            expected,
            actual: namespace.nonce,
        });
    }
    if let Some(expected) = &expectations.quorum_key
        && expected != &namespace.quorum_key
    {
        return Err(VerifyError::QuorumKeyMismatch {
            expected: qos_hex::encode(expected),
            actual: qos_hex::encode(&namespace.quorum_key),
        });
    }

    let enclave = manifest.enclave();
    if let Some(expected) = &expectations.qos_commit
        && expected != &enclave.qos_commit
    {
        return Err(VerifyError::QosCommitMismatch {
            expected: expected.clone(),
            actual: enclave.qos_commit.clone(),
        });
    }
    for (index, expected, actual) in [
        (0u8, &expectations.pcr0, &enclave.pcr0),
        (1, &expectations.pcr1, &enclave.pcr1),
        (2, &expectations.pcr2, &enclave.pcr2),
        (3, &expectations.pcr3, &enclave.pcr3),
    ] {
        if let Some(expected) = expected
            && expected != actual
        {
            return Err(VerifyError::PcrMismatch {
                index,
                expected: qos_hex::encode(expected),
                actual: qos_hex::encode(actual),
            });
        }
    }

    if let Some(expected) = expectations.pivot_hash
        && &expected != manifest.pivot_hash()
    {
        return Err(VerifyError::PivotHashMismatch {
            expected: qos_hex::encode(&expected),
            actual: qos_hex::encode(manifest.pivot_hash()),
        });
    }
    if let Some(expected) = expectations.manifest_hash
        && &expected != manifest_hash
    {
        return Err(VerifyError::ManifestHashMismatch {
            expected: qos_hex::encode(&expected),
            actual: qos_hex::encode(manifest_hash),
        });
    }

    Ok(())
}
