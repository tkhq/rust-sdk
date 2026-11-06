//! Request-bound TVC key acceptance.
//!
//! Submission receipts establish which requests this client sent. Only an
//! independently pinned Signer quorum key and AWS boot evidence authenticate
//! the ephemeral key that signs each returned claim.

mod claims;
mod endorsement;

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
    /// An operator role point already belongs to another stored role.
    #[error("TVC operator key reuses a stored operator role key")]
    KeyReuse,
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

/// Parsed operator intent and a complete local inventory of both role points.
pub struct OperatorExpectation {
    organization: String,
    wallet: Option<String>,
    base_path: String,
    inventory: HashSet<RolePublicKey>,
}

impl OperatorExpectation {
    /// Parse caller-owned intent and every observed role. Missing inventory
    /// files must be reported by the caller rather than omitted from this list.
    pub fn parse(
        organization: String,
        wallet: Option<String>,
        base_path: String,
        inventory: Vec<String>,
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
            inventory: inventory
                .iter()
                .map(|v| RolePublicKey::parse(v))
                .collect::<Result<_, _>>()?,
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
        let claims = self
            .claims(client, &expected.organization, &activity.app_proofs)
            .await?;
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
        if !common_submission(&receipt, &[&encrypt.context, &sign.context]) {
            return Err(Error::ReceiptMismatch);
        }
        let encrypt_key = RolePublicKey::parse(&encrypt.body.address)?;
        let sign_key = RolePublicKey::parse(&sign.body.address)?;
        if encrypt_key == sign_key {
            return Err(Error::ClaimMismatch("operator role keys must differ"));
        }
        if expected.inventory.contains(&encrypt_key) || expected.inventory.contains(&sign_key) {
            return Err(Error::KeyReuse);
        }
        if result.wallet_id != encrypt.body.wallet_id
            || result.encrypt_public_key != encrypt.body.address
            || result.sign_public_key != sign.body.address
        {
            return Err(Error::ClaimMismatch("operator result wallet/keys"));
        }
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
        let mut claims = self
            .claims(client, &expected.organization, &activity.app_proofs)
            .await?
            .into_iter();
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
        if !common_submission(&receipt, &[&claim.context]) {
            return Err(Error::ReceiptMismatch);
        }
        claims::composite(&body.quorum_public_key)?;
        if result.quorum_key_id != body.quorum_key_id
            || result.quorum_public_key != body.quorum_public_key
        {
            return Err(Error::ClaimMismatch("quorum result ID/key"));
        }
        if result.share_ids.len() != expected.recipients.len()
            || result.share_ids.iter().any(String::is_empty)
        {
            return Err(Error::ClaimMismatch("quorum share routing labels"));
        }
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
    ) -> Result<Vec<Claim>, Error> {
        if proofs.len() > 64 {
            return Err(Error::MalformedEvidence(
                "more than 64 proof envelopes".into(),
            ));
        }
        let mut claims = Vec::new();
        for proof in proofs {
            if proof.proof_payload.len() > 256 * 1024 {
                return Err(Error::MalformedEvidence(
                    "inner proof exceeds 256 KiB".into(),
                ));
            }
            if claims::is_policy(&proof.proof_payload)? {
                continue;
            }
            claims::composite(&proof.public_key)?;
            let mut found = None;
            for attempt in 0..3 {
                match crate::get_boot_proof_for_app_proof(client, organization.to_owned(), proof)
                    .await
                {
                    Ok(response) => {
                        found = response.boot_proof;
                        if found.is_some() || attempt == 2 {
                            break;
                        }
                    }
                    Err(source) => {
                        let cause = BootProofLookupError::from(source);
                        let retryable = matches!(
                            cause,
                            BootProofLookupError::Transport
                                | BootProofLookupError::HttpStatus(404 | 429 | 500..=599)
                        );
                        if !retryable || attempt == 2 {
                            return Err(Error::BootProofLookup(cause));
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50 * (attempt + 1))).await;
            }
            let boot = found.ok_or(Error::ProofUnavailable)?;
            let authenticated = self.endorsement.authenticate(proof, &boot)?;
            claims.push(claims::decode(authenticated.bytes())?);
        }
        Ok(claims)
    }
}

fn common_submission(receipt: &ActivityReceipt, contexts: &[&Context]) -> bool {
    receipt.sent_fingerprints().any(|sent| {
        contexts.iter().all(|context| {
            context
                .digests
                .iter()
                .any(|digest| digest == &sent.to_hex())
        })
    })
}

#[cfg(test)]
mod tests;
