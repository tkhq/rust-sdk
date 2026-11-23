//! Parsed creation expectations and authenticated key outputs.

use std::{collections::HashSet, fmt, str::FromStr};
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

impl FromStr for P256PublicKey {
    type Err = TvcProvenanceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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
        // The SEC1 decoder checks the tag against the length. At 65 bytes,
        // the only valid P-256 encoding is an uncompressed point.
        let key = p256::PublicKey::from_sec1_bytes(&bytes).map_err(|_| {
            TvcProvenanceError::MalformedEvidence(
                "role key must be an uncompressed P-256 point".into(),
            )
        })?;
        Ok(key.into())
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
    /// Assemble a quorum expectation from already parsed recipient keys.
    pub fn new(
        organization: String,
        threshold: u32,
        recipients: Vec<P256PublicKey>,
    ) -> Result<Self, TvcProvenanceError> {
        let count = recipients.len();
        if organization.is_empty()
            || !(2..=255).contains(&threshold)
            || count > 255
            || count < threshold as usize
        {
            return Err(TvcProvenanceError::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255",
            ));
        }
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

impl VerifiedQuorumKey {
    /// Consume the signed ID/key and separately supplied share routing labels.
    pub fn into_parts(self) -> (Uuid, QuorumPublicKey, Vec<Uuid>) {
        (self.id, self.public_key, self.untrusted_share_ids)
    }
}
