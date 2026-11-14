//! Diagnostic observations from the acceptance path. Only verified key types grant authority.

use super::{Error, RolePublicKey};

/// Outcome of a check actually reached by verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    NotRun,
    Pass,
    Fail,
}

/// A compound operation in the existing fail-fast verifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProofCheck {
    ActivityShape,
    EnvelopeCount,
    EnvelopeRouting,
    EphemeralKeyFormat,
    BootProofLookup,
    DocumentEndorsement,
    AwsAttestationAtIssuance,
    BootBinding,
    ApplicationSignature,
    ClaimDecode,
    ClaimSetAndIntent,
    CommonSubmission,
    OutputKeyRoles,
    ResultBinding,
    ShareRoutingShape,
}

/// A check and its observed outcome.
#[derive(Debug, PartialEq, Eq)]
pub struct Check {
    pub kind: ProofCheck,
    pub state: CheckState,
}

/// AWS-authenticated fields, available only after attestation verification.
#[derive(Debug, PartialEq, Eq)]
pub struct AttestationEvidence {
    pub issued_at_ms: u64,
    pub public_key: Option<String>,
    pub manifest_commitment: Option<String>,
    pub pcrs: Vec<(usize, String)>,
}

/// Supplied manifest operands. Authentication requires the combined boot check.
#[derive(Debug, PartialEq, Eq)]
pub struct BootEvidence {
    pub manifest_hash: String,
    pub envelope_manifest_hash: String,
    pub manifest_quorum_key: String,
    pub ephemeral_key: String,
    pub pcrs: [String; 4],
}

/// Strictly decoded, authenticated signed creation claims.
#[derive(Debug)]
pub enum ClaimEvidence {
    Operator {
        organization: String,
        wallet: String,
        path: String,
        public_key: String,
        digests: Vec<String>,
    },
    Quorum {
        organization: String,
        id: String,
        public_key: String,
        threshold: u32,
        recipients: Vec<String>,
        digests: Vec<String>,
    },
}

/// Returned operator operands; the operator ID is an unsigned routing label.
#[derive(Debug)]
pub struct OperatorResultEvidence {
    pub wallet: String,
    pub encrypt: String,
    pub sign: String,
    pub operator_id: String,
}

/// Returned quorum operands; share IDs are unsigned routing labels.
#[derive(Debug)]
pub struct QuorumResultEvidence {
    pub id: String,
    pub public_key: String,
    pub share_ids: Vec<String>,
}

/// Caller intent and unsigned activity-result operands.
#[derive(Debug)]
pub enum AcceptanceEvidence {
    Operator {
        organization: String,
        wallet: Option<String>,
        base_path: String,
        returned: Option<OperatorResultEvidence>,
    },
    Quorum {
        organization: String,
        threshold: u32,
        recipients: Vec<RolePublicKey>,
        returned: Option<QuorumResultEvidence>,
    },
}

/// One envelope's checks; policy envelopes are deliberately excluded.
#[derive(Debug)]
pub struct EnvelopeReport {
    pub(super) checks: Vec<Check>,
    pub(super) policy_excluded: bool,
    pub(super) ephemeral_key: Option<String>,
    pub(super) attestation: Option<AttestationEvidence>,
    pub(super) boot: Option<BootEvidence>,
    pub(super) claim: Option<ClaimEvidence>,
}

impl EnvelopeReport {
    pub(super) fn new() -> Self {
        use ProofCheck::*;
        Self {
            checks: [
                EnvelopeRouting,
                EphemeralKeyFormat,
                BootProofLookup,
                DocumentEndorsement,
                AwsAttestationAtIssuance,
                BootBinding,
                ApplicationSignature,
                ClaimDecode,
            ]
            .into_iter()
            .map(|kind| Check {
                kind,
                state: CheckState::NotRun,
            })
            .collect(),
            policy_excluded: false,
            ephemeral_key: None,
            attestation: None,
            boot: None,
            claim: None,
        }
    }
    pub(super) fn set(&mut self, kind: ProofCheck, state: CheckState) {
        self.checks
            .iter_mut()
            .find(|check| check.kind == kind)
            .expect("envelope check catalog")
            .state = state;
    }
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }
    pub fn policy_excluded(&self) -> bool {
        self.policy_excluded
    }
    pub fn ephemeral_key(&self) -> Option<&str> {
        self.ephemeral_key.as_deref()
    }
    pub fn attestation(&self) -> Option<&AttestationEvidence> {
        self.attestation.as_ref()
    }
    pub fn boot(&self) -> Option<&BootEvidence> {
        self.boot.as_ref()
    }
    pub fn claim(&self) -> Option<&ClaimEvidence> {
        self.claim.as_ref()
    }
}

/// Owned public diagnostics retained on both success and failure.
#[derive(Debug)]
pub struct ProofReport {
    pub(super) environment: &'static str,
    pub(super) signer_quorum_key: String,
    pub(super) checks: Vec<Check>,
    pub(super) envelopes: Vec<EnvelopeReport>,
    pub(super) acceptance: AcceptanceEvidence,
    pub(super) sent_fingerprints: Vec<String>,
    pub(super) matched_fingerprint: Option<String>,
}
impl ProofReport {
    pub(super) fn new(
        environment: &'static str,
        signer_quorum_key: String,
        acceptance: AcceptanceEvidence,
        sent_fingerprints: Vec<String>,
        count: usize,
    ) -> Self {
        use ProofCheck::*;
        let mut checks = vec![
            ActivityShape,
            EnvelopeCount,
            ClaimSetAndIntent,
            CommonSubmission,
            OutputKeyRoles,
            ResultBinding,
        ];
        if matches!(acceptance, AcceptanceEvidence::Quorum { .. }) {
            checks.push(ShareRoutingShape);
        }
        Self {
            environment,
            signer_quorum_key,
            checks: checks
                .into_iter()
                .map(|kind| Check {
                    kind,
                    state: CheckState::NotRun,
                })
                .collect(),
            envelopes: (0..count.min(64)).map(|_| EnvelopeReport::new()).collect(),
            acceptance,
            sent_fingerprints,
            matched_fingerprint: None,
        }
    }
    pub(super) fn set(&mut self, kind: ProofCheck, state: CheckState) {
        self.checks
            .iter_mut()
            .find(|check| check.kind == kind)
            .expect("acceptance check catalog")
            .state = state;
    }
    pub fn environment(&self) -> &str {
        self.environment
    }
    pub fn signer_quorum_key(&self) -> &str {
        &self.signer_quorum_key
    }
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }
    pub fn envelopes(&self) -> &[EnvelopeReport] {
        &self.envelopes
    }
    pub fn acceptance(&self) -> &AcceptanceEvidence {
        &self.acceptance
    }
    pub fn sent_fingerprints(&self) -> &[String] {
        &self.sent_fingerprints
    }
    pub fn matched_fingerprint(&self) -> Option<&str> {
        self.matched_fingerprint.as_deref()
    }
}

/// Original typed result alongside diagnostics, without an error wrapper.
pub struct ReportedAcceptance<T> {
    pub(super) result: Result<T, Error>,
    pub(super) report: ProofReport,
}
impl<T> ReportedAcceptance<T> {
    pub fn into_parts(self) -> (Result<T, Error>, ProofReport) {
        (self.result, self.report)
    }
    pub fn into_result(self) -> Result<T, Error> {
        self.result
    }
}
