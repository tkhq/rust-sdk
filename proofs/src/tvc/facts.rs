//! Records what the verifier checked. Only a successful acceptance returns verified keys.

use super::{ClaimEvidence, P256PublicKey, TvcProvenanceError, endorsement::SignerEnvironment};
use crate::EphemeralPublicKey;
use std::{fmt, sync::Arc};
use turnkey_client::RequestFingerprint;
use turnkey_enclave_encrypt::QuorumPublicKey;

/// Outcome of a check actually reached by verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
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

/// AWS-authenticated bytes. Key and commitment shape are checked during boot binding.
#[derive(Debug, PartialEq, Eq)]
pub struct AttestationEvidence {
    pub issued_at_ms: u64,
    pub public_key: Option<Vec<u8>>,
    pub manifest_commitment: Option<Vec<u8>>,
    pub pcrs: Vec<(usize, Vec<u8>)>,
}

/// Supplied manifest operands. Authentication requires the combined boot check.
#[derive(Debug, PartialEq, Eq)]
pub struct BootEvidence {
    pub manifest_hash: Vec<u8>,
    pub envelope_manifest_hash: Vec<u8>,
    pub manifest_quorum_key: Vec<u8>,
    pub ephemeral_key: EphemeralPublicKey,
    pub pcrs: [Vec<u8>; 4],
}

/// Raw returned operator operands, retained even when malformed.
/// The operator ID is an unsigned routing label.
#[derive(Debug)]
pub struct OperatorResultEvidence {
    pub wallet: String,
    pub encrypt: String,
    pub sign: String,
    pub operator_id: String,
}

/// Raw returned quorum operands, retained even when malformed.
/// Share IDs are unsigned routing labels.
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
        recipients: Vec<P256PublicKey>,
        returned: Option<QuorumResultEvidence>,
    },
}

/// A verification check and its zero-based envelope index when it operates on one envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcceptanceStage {
    pub check: ProofCheck,
    pub envelope_index: Option<usize>,
}

impl fmt::Display for AcceptanceStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.check {
            ProofCheck::ActivityShape => "activity shape",
            ProofCheck::EnvelopeCount => "envelope count",
            ProofCheck::EnvelopeRouting => "envelope routing",
            ProofCheck::EphemeralKeyFormat => "ephemeral key format",
            ProofCheck::BootProofLookup => "boot proof lookup",
            ProofCheck::DocumentEndorsement => "Signer document endorsement",
            ProofCheck::AwsAttestationAtIssuance => "AWS attestation at issuance",
            ProofCheck::BootBinding => "boot binding",
            ProofCheck::ApplicationSignature => "application signature",
            ProofCheck::ClaimDecode => "claim decoding",
            ProofCheck::ClaimSetAndIntent => "claim set and caller intent",
            ProofCheck::CommonSubmission => "common submission",
            ProofCheck::OutputKeyRoles => "output key roles",
            ProofCheck::ResultBinding => "result binding",
            ProofCheck::ShareRoutingShape => "share routing",
        };
        f.write_str(name)?;
        if let Some(index) = self.envelope_index {
            write!(f, " (proof envelope {index})")?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct FailedCheck {
    stage: AcceptanceStage,
    pub(super) source: TvcProvenanceError,
}

pub(super) struct CheckOutcome<T> {
    kind: ProofCheck,
    result: Result<T, TvcProvenanceError>,
}

impl<T> CheckOutcome<T> {
    pub(super) fn new(kind: ProofCheck, result: Result<T, TvcProvenanceError>) -> Self {
        Self { kind, result }
    }

    pub(super) fn run(
        kind: ProofCheck,
        check: impl FnOnce() -> Result<T, TvcProvenanceError>,
    ) -> Self {
        Self::new(kind, check())
    }
}

/// Checks recorded in execution order.
#[derive(Default, Debug)]
pub(super) struct CheckTrace {
    envelope_index: Option<usize>,
    executed: Vec<Check>,
}
impl CheckTrace {
    pub(super) fn record<T>(&mut self, outcome: CheckOutcome<T>) -> Result<T, FailedCheck> {
        let CheckOutcome { kind, result } = outcome;
        assert!(
            !self.executed.iter().any(|check| check.kind == kind),
            "check recorded twice: {kind:?}"
        );
        self.executed.push(Check {
            kind,
            state: if result.is_ok() {
                CheckState::Pass
            } else {
                CheckState::Fail
            },
        });
        result.map_err(|source| FailedCheck {
            stage: AcceptanceStage {
                check: kind,
                envelope_index: self.envelope_index,
            },
            source,
        })
    }
}

/// Diagnostic operands collected independently of check verdicts.
#[derive(Debug, Default)]
pub(super) struct BootObservation {
    pub(super) attestation: Option<AttestationEvidence>,
    pub(super) boot: Option<BootEvidence>,
}

#[derive(Default)]
pub(super) struct AcceptanceTrace {
    pub(super) checks: CheckTrace,
    pub(super) envelopes: Vec<EnvelopeFacts>,
    pub(super) matched_fingerprint: Option<RequestFingerprint>,
}

/// One visited envelope's executed checks and observations.
/// Policy envelopes remain visible but are excluded from creation acceptance.
#[derive(Debug)]
pub struct EnvelopeFacts {
    pub(super) checks: CheckTrace,
    pub(super) policy_excluded: bool,
    pub(super) ephemeral_key: Option<EphemeralPublicKey>,
    pub(super) boot: Option<Arc<BootObservation>>,
    pub(super) boot_source_envelope: Option<usize>,
    pub(super) claim: Option<ClaimEvidence>,
}
impl EnvelopeFacts {
    pub(super) fn new(index: usize) -> Self {
        Self {
            checks: CheckTrace {
                envelope_index: Some(index),
                executed: Vec::new(),
            },
            policy_excluded: false,
            ephemeral_key: None,
            boot: None,
            boot_source_envelope: None,
            claim: None,
        }
    }
    pub fn checks(&self) -> &[Check] {
        &self.checks.executed
    }
    pub fn policy_excluded(&self) -> bool {
        self.policy_excluded
    }
    /// Parsed key supplied by the envelope. Boot binding authenticates this operand.
    pub fn ephemeral_key(&self) -> Option<&EphemeralPublicKey> {
        self.ephemeral_key.as_ref()
    }
    pub fn attestation(&self) -> Option<&AttestationEvidence> {
        self.boot.as_ref()?.attestation.as_ref()
    }
    pub fn boot(&self) -> Option<&BootEvidence> {
        self.boot.as_ref()?.boot.as_ref()
    }
    pub fn claim(&self) -> Option<&ClaimEvidence> {
        self.claim.as_ref()
    }
    /// Zero-based index into [`AcceptanceFacts::envelopes`] whose verified boot evidence was reused.
    /// Lookup and boot stages are not run again; the application signature is always checked.
    pub fn boot_source_envelope(&self) -> Option<usize> {
        self.boot_source_envelope
    }
}

/// Owned public diagnostics retained on both success and failure.
#[derive(Debug)]
pub struct AcceptanceFacts {
    environment: SignerEnvironment,
    signer_quorum_key: QuorumPublicKey,
    checks: Vec<Check>,
    envelope_count: usize,
    envelopes: Vec<EnvelopeFacts>,
    acceptance: AcceptanceEvidence,
    sent_fingerprints: Vec<RequestFingerprint>,
    matched_fingerprint: Option<RequestFingerprint>,
}
impl AcceptanceFacts {
    pub(super) fn assemble(
        environment: SignerEnvironment,
        signer_quorum_key: QuorumPublicKey,
        acceptance: AcceptanceEvidence,
        sent_fingerprints: Vec<RequestFingerprint>,
        count: usize,
        trace: AcceptanceTrace,
    ) -> Self {
        Self {
            environment,
            signer_quorum_key,
            checks: trace.checks.executed,
            envelope_count: count,
            envelopes: trace.envelopes,
            acceptance,
            sent_fingerprints,
            matched_fingerprint: trace.matched_fingerprint,
        }
    }
    pub fn environment(&self) -> &str {
        self.environment.label()
    }
    pub fn signer_quorum_key(&self) -> &QuorumPublicKey {
        &self.signer_quorum_key
    }
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }
    /// Number of supplied envelopes, including any that were never visited.
    pub fn envelope_count(&self) -> usize {
        self.envelope_count
    }
    /// The visited prefix of the supplied envelopes, in their original order.
    pub fn envelopes(&self) -> &[EnvelopeFacts] {
        &self.envelopes
    }
    pub fn acceptance(&self) -> &AcceptanceEvidence {
        &self.acceptance
    }
    pub fn sent_fingerprints(&self) -> &[RequestFingerprint] {
        &self.sent_fingerprints
    }
    pub fn matched_fingerprint(&self) -> Option<&RequestFingerprint> {
        self.matched_fingerprint.as_ref()
    }

    /// Attaches the recorded facts to the verified value or to the original failure.
    pub(super) fn finish<T>(self, result: Result<T, FailedCheck>) -> Result<Accepted<T>, Rejected> {
        match result {
            Ok(value) => Ok(Accepted { value, facts: self }),
            Err(FailedCheck { stage, source }) => Err(Rejected {
                stage,
                source,
                facts: Box::new(self),
            }),
        }
    }
}

/// The verified output of an acceptance and its public diagnostic facts.
///
/// ```compile_fail,E0451
/// use turnkey_proofs::tvc::{Accepted, AcceptanceFacts};
/// fn fabricate<T>(value: T, facts: AcceptanceFacts) -> Accepted<T> {
///     Accepted { value, facts }
/// }
/// ```
#[derive(Debug)]
pub struct Accepted<T> {
    value: T,
    facts: AcceptanceFacts,
}
impl<T> Accepted<T> {
    pub fn facts(&self) -> &AcceptanceFacts {
        &self.facts
    }
    pub fn into_value(self) -> T {
        self.value
    }
    pub fn into_parts(self) -> (T, AcceptanceFacts) {
        (self.value, self.facts)
    }
}

/// A failed acceptance with the original cause and the facts collected up to that check.
#[derive(Debug, thiserror::Error)]
#[error("TVC proof rejected at {stage}")]
pub struct Rejected {
    stage: AcceptanceStage,
    #[source]
    source: TvcProvenanceError,
    facts: Box<AcceptanceFacts>,
}
impl Rejected {
    pub fn stage(&self) -> AcceptanceStage {
        self.stage
    }
    pub fn facts(&self) -> &AcceptanceFacts {
        &self.facts
    }
}

impl From<&crate::ParsedBootProof> for BootEvidence {
    fn from(parsed: &crate::ParsedBootProof) -> Self {
        // QOS's accessor consumes the envelope. Retain its supplied PCRs even on binding failure.
        let supplied_manifest = parsed.manifest_envelope.clone().manifest();
        let enclave = supplied_manifest.enclave();
        Self {
            manifest_hash: parsed.manifest.manifest_hash().to_vec(),
            envelope_manifest_hash: parsed.manifest_envelope.manifest_hash().to_vec(),
            manifest_quorum_key: parsed.manifest.namespace().quorum_key.clone(),
            ephemeral_key: parsed.ephemeral_key.clone(),
            pcrs: [&enclave.pcr0, &enclave.pcr1, &enclave.pcr2, &enclave.pcr3].map(Clone::clone),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ProofCheck::*;

    #[test]
    fn recorder_returns_the_original_value_or_a_located_cause() {
        let mut trace = CheckTrace::default();
        assert_eq!(
            trace
                .record(CheckOutcome::new(ActivityShape, Ok(42)))
                .unwrap(),
            42
        );
        let failure = trace
            .record(CheckOutcome::<()>::new(
                EnvelopeCount,
                Err(TvcProvenanceError::ProofUnavailable),
            ))
            .unwrap_err();
        assert_eq!(
            failure.stage,
            AcceptanceStage {
                check: EnvelopeCount,
                envelope_index: None
            }
        );
        assert!(matches!(
            failure.source,
            TvcProvenanceError::ProofUnavailable
        ));
        assert_eq!(
            trace.executed,
            [
                Check {
                    kind: ActivityShape,
                    state: CheckState::Pass
                },
                Check {
                    kind: EnvelopeCount,
                    state: CheckState::Fail
                },
            ]
        );
    }

    #[test]
    #[should_panic(expected = "check recorded twice")]
    fn duplicate_stages_are_detected() {
        let mut trace = CheckTrace::default();
        trace
            .record(CheckOutcome::new(ActivityShape, Ok(())))
            .unwrap();
        let _ = trace.record(CheckOutcome::new(ActivityShape, Ok(())));
    }
}
