//! Parsed creation expectations and authenticated key outputs.

use std::{collections::HashSet, fmt};
use turnkey_client::TurnkeyClientError;
use turnkey_enclave_encrypt::QuorumPublicKey;
use uuid::Uuid;

/// A failure to authenticate or accept request-bound TVC output.
#[derive(Debug, thiserror::Error)]
pub enum TvcProvenanceError {
    /// The local API environment has no independently supplied Signer root.
    #[error(
        "unsupported API environment for TVC creation provenance; no creation request was sent"
    )]
    UnsupportedEnvironment,
    /// Required request-bound output claims are absent.
    #[error("required request-bound TVC proof is unavailable or legacy")]
    ProofUnavailable,
    /// Fetching the instance's boot proof failed after completed creation.
    #[error("failed to fetch the Signer boot proof for completed creation")]
    BootProofLookup(#[source] BootProofLookupError),
    /// A context version is not supported.
    #[error("unsupported TVC proof request context")]
    UnsupportedContext,
    /// Evidence violates the strict wire contract or client bounds.
    #[error("malformed TVC evidence: {0}")]
    MalformedEvidence(String),
    /// The endorsement does not match the independent policy.
    #[error("TVC proof endorsement or signature verification failed")]
    EndorsementFailure,
    /// No actual submission matches the required claims together.
    #[error("TVC proof does not match this invocation's submission receipt")]
    ReceiptMismatch,
    /// A claim disagrees with intent or with the returned result.
    #[error("TVC proof claim/result mismatch: {0}")]
    ClaimMismatch(&'static str),
    /// Expected values are invalid before submission.
    #[error("invalid TVC creation expectation: {0}")]
    InvalidExpectation(&'static str),
}

/// A bounded, safe cause for failed boot evidence retrieval.
/// Coordinator response bodies and stamp material are never retained.
#[derive(Debug, thiserror::Error)]
pub enum BootProofLookupError {
    /// The evidence request failed in transport.
    #[error("boot proof transport failed")]
    Transport,
    /// The evidence endpoint returned a non-success status.
    #[error("boot proof HTTP status {0}")]
    HttpStatus(u16),
    /// The endpoint returned malformed or unexpected data.
    #[error("invalid boot proof response")]
    InvalidResponse,
}
impl From<TurnkeyClientError> for BootProofLookupError {
    fn from(error: TurnkeyClientError) -> Self {
        match error {
            TurnkeyClientError::Http(_) => Self::Transport,
            TurnkeyClientError::UnexpectedHttpStatus(status, _)
            | TurnkeyClientError::RefusedRedirect(status, _) => Self::HttpStatus(status),
            _ => Self::InvalidResponse,
        }
    }
}

/// A canonical, valid uncompressed P-256 public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct P256PublicKey(p256::EncodedPoint);

impl TryFrom<&str> for P256PublicKey {
    type Error = TvcProvenanceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 130
            || !value
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(TvcProvenanceError::MalformedEvidence(
                "role key must be 65-byte lowercase hex".into(),
            ));
        }
        let bytes = hex::decode(value)
            .map_err(|_| TvcProvenanceError::MalformedEvidence("invalid role key hex".into()))?;
        if bytes.first() != Some(&4) {
            return Err(TvcProvenanceError::MalformedEvidence(
                "role key must be an uncompressed P-256 point".into(),
            ));
        }
        let key = p256::PublicKey::from_sec1_bytes(&bytes).map_err(|_| {
            TvcProvenanceError::MalformedEvidence(
                "role key must be an uncompressed P-256 point".into(),
            )
        })?;
        Ok(key.into())
    }
}

impl P256PublicKey {
    /// Parse the signed wire form without normalization.
    pub fn parse(value: &str) -> Result<Self, TvcProvenanceError> {
        value.try_into()
    }
}

impl From<p256::PublicKey> for P256PublicKey {
    fn from(key: p256::PublicKey) -> Self {
        use p256::elliptic_curve::sec1::ToEncodedPoint;
        Self(key.to_encoded_point(false))
    }
}

impl fmt::Display for P256PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0.as_bytes()))
    }
}

/// Parsed operator intent to match against authenticated claims.
pub struct OperatorExpectation {
    pub(super) organization: String,
    pub(super) wallet: Option<String>,
    pub(super) base_path: String,
}

impl OperatorExpectation {
    /// Parse the caller-owned organization, wallet and derivation path.
    pub fn parse(
        organization: String,
        wallet: Option<String>,
        base_path: String,
    ) -> Result<Self, TvcProvenanceError> {
        if organization.is_empty()
            || base_path.is_empty()
            || wallet.as_ref().is_some_and(String::is_empty)
        {
            return Err(TvcProvenanceError::InvalidExpectation(
                "organization, wallet and path must be nonempty",
            ));
        }
        Ok(Self {
            organization,
            wallet,
            base_path,
        })
    }
}

/// Parsed quorum intent with trusted recipients in their submitted order.
pub struct QuorumExpectation {
    pub(super) organization: String,
    pub(super) threshold: u32,
    pub(super) recipients: Vec<P256PublicKey>,
}

impl QuorumExpectation {
    /// Parse explicit user assertions or previously accepted encryption roles.
    pub fn parse(
        organization: String,
        threshold: u32,
        recipients: impl AsRef<[String]>,
    ) -> Result<Self, TvcProvenanceError> {
        let recipients = recipients.as_ref();
        Self::check_shape(&organization, threshold, recipients.len())?;
        let recipients = recipients
            .iter()
            .map(|key| {
                P256PublicKey::try_from(key.as_str()).map_err(|_| {
                    TvcProvenanceError::InvalidExpectation("invalid recipient public key")
                })
            })
            .collect::<Result<_, _>>()?;
        Self::new(organization, threshold, recipients)
    }

    /// Assemble a quorum expectation from already parsed recipient keys.
    pub fn new(
        organization: String,
        threshold: u32,
        recipients: Vec<P256PublicKey>,
    ) -> Result<Self, TvcProvenanceError> {
        Self::check_shape(&organization, threshold, recipients.len())?;
        if recipients.iter().collect::<HashSet<_>>().len() != recipients.len() {
            return Err(TvcProvenanceError::InvalidExpectation(
                "duplicate recipient",
            ));
        }
        Ok(Self {
            organization,
            threshold,
            recipients,
        })
    }
    fn check_shape(
        organization: &str,
        threshold: u32,
        count: usize,
    ) -> Result<(), TvcProvenanceError> {
        if organization.is_empty()
            || !(2..=255).contains(&threshold)
            || count > 255
            || count < threshold as usize
        {
            return Err(TvcProvenanceError::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255",
            ));
        }
        Ok(())
    }
}

/// Authenticated operator output. The routing label is coordinator supplied.
#[derive(Debug)]
pub struct VerifiedOperatorKeys {
    pub(super) wallet: String,
    pub(super) base_path: String,
    pub(super) encrypt_key: P256PublicKey,
    pub(super) signing_key: P256PublicKey,
    pub(super) untrusted_operator_id: String,
}

impl VerifiedOperatorKeys {
    /// Extracts the fields for a caller to destructure, including the unsigned routing label.
    pub fn into_parts(self) -> VerifiedOperatorParts {
        VerifiedOperatorParts {
            wallet: self.wallet,
            base_path: self.base_path,
            encrypt: self.encrypt_key,
            sign: self.signing_key,
            untrusted_operator_id: self.untrusted_operator_id,
        }
    }
    /// Returns the accepted encryption public key.
    pub fn encryption_key(&self) -> P256PublicKey {
        self.encrypt_key
    }
    /// Returns the accepted signing public key.
    pub fn signing_key(&self) -> P256PublicKey {
        self.signing_key
    }
}

/// Named owned parts of an accepted operator. Constructing these parts does not mint verified keys.
#[derive(Debug)]
pub struct VerifiedOperatorParts {
    pub wallet: String,
    pub base_path: String,
    pub encrypt: P256PublicKey,
    pub sign: P256PublicKey,
    pub untrusted_operator_id: String,
}

/// Authenticated quorum output plus unsigned share routing labels.
#[derive(Debug)]
pub struct VerifiedQuorumKey {
    pub(super) id: Uuid,
    pub(super) public_key: QuorumPublicKey,
    pub(super) untrusted_share_ids: Vec<Uuid>,
}

#[cfg(doctest)]
/// Checks that downstream callers cannot replace a verified key through its fields.
///
/// ```compile_fail,E0451
/// use turnkey_proofs::tvc::{P256PublicKey, VerifiedOperatorKeys};
/// fn substitute(keys: VerifiedOperatorKeys, encrypt_key: P256PublicKey) -> VerifiedOperatorKeys {
///     VerifiedOperatorKeys { encrypt_key, ..keys }
/// }
/// ```
///
/// ```compile_fail,E0451
/// use turnkey_proofs::tvc::VerifiedQuorumKey;
/// use turnkey_enclave_encrypt::QuorumPublicKey;
/// fn substitute(key: VerifiedQuorumKey, public_key: QuorumPublicKey) -> VerifiedQuorumKey {
///     VerifiedQuorumKey { public_key, ..key }
/// }
/// ```
mod verified_key_privacy {}

impl VerifiedQuorumKey {
    /// Consume the signed ID/key and separately supplied share routing labels.
    pub fn into_parts(self) -> (Uuid, QuorumPublicKey, Vec<Uuid>) {
        (self.id, self.public_key, self.untrusted_share_ids)
    }
}
