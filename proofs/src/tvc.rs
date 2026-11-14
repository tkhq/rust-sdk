//! Request-bound TVC key acceptance.
//!
//! Submission receipts establish which requests this client sent. Only an
//! independently pinned Signer quorum key and AWS boot evidence authenticate
//! the ephemeral key that signs each returned claim.

mod claims;
mod endorsement;
mod report;
pub use endorsement::ENDORSEMENT_PREFIX;
pub use report::*;
use report::{
    CheckState::{Fail, Pass},
    ProofCheck::*,
};

use claims::{Claim, Context};
use std::{collections::HashSet, fmt};
use turnkey_client::{
    ActivityReceipt, TurnkeyClient, TurnkeyClientError,
    generated::{ActivityStatus, ActivityType, immutable::activity::v1::result::Inner},
};

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

use TvcProvenanceError as Error;

/// A canonical, valid uncompressed P-256 role point.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RolePublicKey([u8; 65]);

impl RolePublicKey {
    /// Parse the signed wire form without normalization.
    pub fn parse(value: &str) -> Result<Self, Error> {
        if value.len() != 130
            || !value
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(Error::MalformedEvidence(
                "role key must be 65-byte lowercase hex".into(),
            ));
        }
        let bytes = hex::decode(value)
            .map_err(|_| Error::MalformedEvidence("invalid role key hex".into()))?;
        if bytes.first() != Some(&4) || p256::PublicKey::from_sec1_bytes(&bytes).is_err() {
            return Err(Error::MalformedEvidence(
                "role key must be an uncompressed P-256 point".into(),
            ));
        }
        Ok(Self(bytes.try_into().map_err(|_| {
            Error::MalformedEvidence("invalid role key length".into())
        })?))
    }
}

impl fmt::Display for RolePublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

/// Parsed operator intent to match against authenticated claims.
pub struct OperatorExpectation {
    organization: String,
    wallet: Option<String>,
    base_path: String,
}

impl OperatorExpectation {
    /// Parse the caller-owned organization, wallet and derivation path.
    pub fn parse(
        organization: String,
        wallet: Option<String>,
        base_path: String,
    ) -> Result<Self, Error> {
        if organization.is_empty()
            || base_path.is_empty()
            || wallet.as_ref().is_some_and(String::is_empty)
        {
            return Err(Error::InvalidExpectation(
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
    organization: String,
    threshold: u32,
    recipients: Vec<RolePublicKey>,
}

impl QuorumExpectation {
    /// Parse explicit user assertions or previously accepted encryption roles.
    pub fn parse(
        organization: String,
        threshold: u32,
        recipients: Vec<String>,
    ) -> Result<Self, Error> {
        if organization.is_empty()
            || !(2..=255).contains(&threshold)
            || recipients.len() > 255
            || recipients.len() < threshold as usize
        {
            return Err(Error::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255",
            ));
        }
        let recipients = recipients
            .iter()
            .map(|v| RolePublicKey::parse(v))
            .collect::<Result<Vec<_>, _>>()?;
        if recipients.iter().collect::<HashSet<_>>().len() != recipients.len() {
            return Err(Error::InvalidExpectation("duplicate recipient"));
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
    wallet: String,
    base_path: String,
    encrypt: RolePublicKey,
    sign: RolePublicKey,
    untrusted_operator_id: String,
}

impl VerifiedOperatorKeys {
    /// Consume the accepted wallet, path, role keys and unsigned routing label.
    pub fn into_parts(self) -> (String, String, RolePublicKey, RolePublicKey, String) {
        (
            self.wallet,
            self.base_path,
            self.encrypt,
            self.sign,
            self.untrusted_operator_id,
        )
    }
    /// Accepted encryption-role point.
    pub fn encrypt_key(&self) -> RolePublicKey {
        self.encrypt
    }
    /// Accepted signing-role point.
    pub fn sign_key(&self) -> RolePublicKey {
        self.sign
    }
}

/// Authenticated quorum output plus unsigned share routing labels.
#[derive(Debug)]
pub struct VerifiedQuorumKey {
    id: String,
    public_key: String,
    untrusted_share_ids: Vec<String>,
}

impl VerifiedQuorumKey {
    /// Consume the signed ID/key and separately supplied share routing labels.
    pub fn into_parts(self) -> (String, String, Vec<String>) {
        (self.id, self.public_key, self.untrusted_share_ids)
    }
}

/// Opaque capability for the selected independent TVC endorsement policy.
pub struct TvcProvenance {
    endorsement: endorsement::SelectedEndorsement,
}

impl TvcProvenance {
    /// Acquire the approved policy before any creation POST or config write.
    pub fn for_api_url(api_url: &str) -> Result<Self, Error> {
        Ok(Self {
            endorsement: endorsement::SelectedEndorsement::for_api_url(api_url)?,
        })
    }

    /// Authenticate, decode and bind both roles to one actually sent request.
    /// The result comes from the sealed receipt, never a separately supplied result.
    pub async fn accept_operator<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &OperatorExpectation,
        receipt: ActivityReceipt,
    ) -> Result<VerifiedOperatorKeys, Error> {
        self.accept_operator_reported(client, expected, receipt)
            .await
            .into_result()
    }

    /// Accept creation output while retaining diagnostics on failure.
    pub async fn accept_operator_reported<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &OperatorExpectation,
        receipt: ActivityReceipt,
    ) -> ReportedAcceptance<VerifiedOperatorKeys> {
        let mut report = ProofReport::new(
            self.endorsement.environment(),
            self.endorsement.quorum_key(),
            AcceptanceEvidence::Operator {
                organization: expected.organization.clone(),
                wallet: expected.wallet.clone(),
                base_path: expected.base_path.clone(),
                returned: None,
            },
            receipt
                .sent_fingerprints()
                .map(|sent| sent.to_hex())
                .collect(),
            receipt.activity().app_proofs.len(),
        );
        let result = self
            .accept_operator_inner(client, expected, receipt, &mut report)
            .await;
        ReportedAcceptance { result, report }
    }

    async fn accept_operator_inner<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &OperatorExpectation,
        receipt: ActivityReceipt,
        report: &mut ProofReport,
    ) -> Result<VerifiedOperatorKeys, Error> {
        report.set(ActivityShape, Fail);
        let activity = receipt.activity();
        if activity.status != ActivityStatus::Completed
            || activity.r#type != ActivityType::CreateTvcOperator
        {
            return Err(Error::ClaimMismatch("operator activity type/status"));
        }
        let Some(Inner::CreateTvcOperatorResult(result)) =
            activity.result.as_ref().and_then(|r| r.inner.as_ref())
        else {
            return Err(Error::ClaimMismatch("operator result variant"));
        };
        if let AcceptanceEvidence::Operator { returned, .. } = &mut report.acceptance {
            *returned = Some(OperatorResultEvidence {
                wallet: result.wallet_id.clone(),
                encrypt: result.encrypt_public_key.clone(),
                sign: result.sign_public_key.clone(),
                operator_id: result.operator_id.clone(),
            });
        }
        report.set(ActivityShape, Pass);
        let claims = self
            .claims(client, &expected.organization, &activity.app_proofs, report)
            .await?;
        report.set(ClaimSetAndIntent, Fail);
        let mut encrypt = None;
        let mut sign = None;
        for claim in claims {
            let Claim::Operator(claim) = claim else {
                return Err(Error::ClaimMismatch("unexpected quorum claim"));
            };
            if claim.body.organization_id != expected.organization {
                return Err(Error::ClaimMismatch("operator organization"));
            }
            let slot = if claim.body.derivation_path == format!("{}/0", expected.base_path) {
                &mut encrypt
            } else if claim.body.derivation_path == format!("{}/1", expected.base_path) {
                &mut sign
            } else {
                return Err(Error::ClaimMismatch("operator role path"));
            };
            if slot.replace(claim).is_some() {
                return Err(Error::MalformedEvidence(
                    "duplicate operator role claim".into(),
                ));
            }
        }
        let (encrypt, sign) = encrypt.zip(sign).ok_or(Error::ProofUnavailable)?;
        if encrypt.body.wallet_id != sign.body.wallet_id
            || expected
                .wallet
                .as_ref()
                .is_some_and(|wallet| wallet != &encrypt.body.wallet_id)
        {
            return Err(Error::ClaimMismatch("operator wallet"));
        }
        report.set(ClaimSetAndIntent, Pass);
        report.set(CommonSubmission, Fail);
        report.matched_fingerprint =
            common_submission(&receipt, &[&encrypt.context, &sign.context]);
        if report.matched_fingerprint.is_none() {
            return Err(Error::ReceiptMismatch);
        }
        report.set(CommonSubmission, Pass);
        report.set(OutputKeyRoles, Fail);
        let encrypt_key = RolePublicKey::parse(&encrypt.body.address)?;
        let sign_key = RolePublicKey::parse(&sign.body.address)?;
        if encrypt_key == sign_key {
            return Err(Error::ClaimMismatch("operator role keys must differ"));
        }
        report.set(OutputKeyRoles, Pass);
        report.set(ResultBinding, Fail);
        if result.wallet_id != encrypt.body.wallet_id
            || result.encrypt_public_key != encrypt.body.address
            || result.sign_public_key != sign.body.address
        {
            return Err(Error::ClaimMismatch("operator result wallet/keys"));
        }
        report.set(ResultBinding, Pass);
        Ok(VerifiedOperatorKeys {
            wallet: encrypt.body.wallet_id,
            base_path: expected.base_path.clone(),
            encrypt: encrypt_key,
            sign: sign_key,
            untrusted_operator_id: result.operator_id.clone(),
        })
    }

    /// Authenticate a quorum claim and bind its complete output and ordered intent.
    pub async fn accept_quorum<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &QuorumExpectation,
        receipt: ActivityReceipt,
    ) -> Result<VerifiedQuorumKey, Error> {
        self.accept_quorum_reported(client, expected, receipt)
            .await
            .into_result()
    }

    /// Accept creation output while retaining diagnostics on failure.
    pub async fn accept_quorum_reported<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &QuorumExpectation,
        receipt: ActivityReceipt,
    ) -> ReportedAcceptance<VerifiedQuorumKey> {
        let mut report = ProofReport::new(
            self.endorsement.environment(),
            self.endorsement.quorum_key(),
            AcceptanceEvidence::Quorum {
                organization: expected.organization.clone(),
                threshold: expected.threshold,
                recipients: expected.recipients.clone(),
                returned: None,
            },
            receipt
                .sent_fingerprints()
                .map(|sent| sent.to_hex())
                .collect(),
            receipt.activity().app_proofs.len(),
        );
        let result = self
            .accept_quorum_inner(client, expected, receipt, &mut report)
            .await;
        ReportedAcceptance { result, report }
    }

    async fn accept_quorum_inner<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        expected: &QuorumExpectation,
        receipt: ActivityReceipt,
        report: &mut ProofReport,
    ) -> Result<VerifiedQuorumKey, Error> {
        report.set(ActivityShape, Fail);
        let activity = receipt.activity();
        if activity.status != ActivityStatus::Completed
            || activity.r#type != ActivityType::CreateTvcQuorumKey
        {
            return Err(Error::ClaimMismatch("quorum activity type/status"));
        }
        let Some(Inner::CreateTvcQuorumKeyResult(result)) =
            activity.result.as_ref().and_then(|r| r.inner.as_ref())
        else {
            return Err(Error::ClaimMismatch("quorum result variant"));
        };
        if let AcceptanceEvidence::Quorum { returned, .. } = &mut report.acceptance {
            *returned = Some(QuorumResultEvidence {
                id: result.quorum_key_id.clone(),
                public_key: result.quorum_public_key.clone(),
                share_ids: result.share_ids.clone(),
            });
        }
        report.set(ActivityShape, Pass);
        let mut claims = self
            .claims(client, &expected.organization, &activity.app_proofs, report)
            .await?
            .into_iter();
        report.set(ClaimSetAndIntent, Fail);
        let Some(Claim::Quorum(claim)) = claims.next() else {
            return Err(Error::ProofUnavailable);
        };
        if claims.next().is_some() {
            return Err(Error::MalformedEvidence(
                "duplicate/conflicting quorum claim".into(),
            ));
        }
        let body = claim.body;
        let recipients = body
            .operator_encrypt_keys
            .iter()
            .map(|v| RolePublicKey::parse(v))
            .collect::<Result<Vec<_>, _>>()?;
        if body.organization_id != expected.organization
            || body.threshold != expected.threshold
            || recipients != expected.recipients
        {
            return Err(Error::ClaimMismatch(
                "quorum organization/threshold/ordered recipients",
            ));
        }
        report.set(ClaimSetAndIntent, Pass);
        report.set(CommonSubmission, Fail);
        report.matched_fingerprint = common_submission(&receipt, &[&claim.context]);
        if report.matched_fingerprint.is_none() {
            return Err(Error::ReceiptMismatch);
        }
        report.set(CommonSubmission, Pass);
        report.set(OutputKeyRoles, Fail);
        claims::composite(&body.quorum_public_key)?;
        report.set(OutputKeyRoles, Pass);
        report.set(ResultBinding, Fail);
        if result.quorum_key_id != body.quorum_key_id
            || result.quorum_public_key != body.quorum_public_key
        {
            return Err(Error::ClaimMismatch("quorum result ID/key"));
        }
        report.set(ResultBinding, Pass);
        report.set(ShareRoutingShape, Fail);
        if result.share_ids.len() != expected.recipients.len()
            || result.share_ids.iter().any(String::is_empty)
        {
            return Err(Error::ClaimMismatch("quorum share routing labels"));
        }
        report.set(ShareRoutingShape, Pass);
        Ok(VerifiedQuorumKey {
            id: body.quorum_key_id,
            public_key: body.quorum_public_key,
            untrusted_share_ids: result.share_ids.clone(),
        })
    }

    async fn claims<S: turnkey_api_key_stamper::Stamp>(
        &self,
        client: &TurnkeyClient<S>,
        organization: &str,
        proofs: &[turnkey_client::generated::external::data::v1::AppProof],
        report: &mut ProofReport,
    ) -> Result<Vec<Claim>, Error> {
        report.set(EnvelopeCount, Fail);
        if proofs.len() > 64 {
            return Err(Error::MalformedEvidence(
                "more than 64 proof envelopes".into(),
            ));
        }
        report.set(EnvelopeCount, Pass);
        let mut claims = Vec::new();
        for (proof, entry) in proofs.iter().zip(&mut report.envelopes) {
            entry.set(EnvelopeRouting, Fail);
            if proof.proof_payload.len() > 256 * 1024 {
                return Err(Error::MalformedEvidence(
                    "inner proof exceeds 256 KiB".into(),
                ));
            }
            if claims::is_policy(&proof.proof_payload)? {
                entry.set(EnvelopeRouting, Pass);
                entry.policy_excluded = true;
                continue;
            }
            entry.set(EnvelopeRouting, Pass);
            entry.set(EphemeralKeyFormat, Fail);
            claims::composite(&proof.public_key)?;
            entry.ephemeral_key = Some(proof.public_key.clone());
            entry.set(EphemeralKeyFormat, Pass);
            entry.set(BootProofLookup, Fail);
            let boot = crate::get_boot_proof_for_app_proof(client, organization.to_owned(), proof)
                .await
                .map_err(|error| Error::BootProofLookup(error.into()))?
                .boot_proof
                .ok_or(Error::ProofUnavailable)?;
            entry.set(BootProofLookup, Pass);
            let authenticated = self
                .endorsement
                .authenticate_reported(proof, &boot, entry)?;
            entry.set(ClaimDecode, Fail);
            let claim = claims::decode(authenticated.bytes())?;
            entry.claim = Some(match &claim {
                Claim::Operator(claim) => ClaimEvidence::Operator {
                    organization: claim.body.organization_id.clone(),
                    wallet: claim.body.wallet_id.clone(),
                    path: claim.body.derivation_path.clone(),
                    public_key: claim.body.address.clone(),
                    digests: claim.context.digests.clone(),
                },
                Claim::Quorum(claim) => ClaimEvidence::Quorum {
                    organization: claim.body.organization_id.clone(),
                    id: claim.body.quorum_key_id.clone(),
                    public_key: claim.body.quorum_public_key.clone(),
                    threshold: claim.body.threshold,
                    recipients: claim.body.operator_encrypt_keys.clone(),
                    digests: claim.context.digests.clone(),
                },
            });
            entry.set(ClaimDecode, Pass);
            claims.push(claim);
        }
        Ok(claims)
    }
}

fn common_submission(receipt: &ActivityReceipt, contexts: &[&Context]) -> Option<String> {
    receipt
        .sent_fingerprints()
        .find(|sent| {
            contexts.iter().all(|context| {
                context
                    .digests
                    .iter()
                    .any(|digest| digest == &sent.to_hex())
            })
        })
        .map(|sent| sent.to_hex())
}

#[cfg(test)]
mod tests;
