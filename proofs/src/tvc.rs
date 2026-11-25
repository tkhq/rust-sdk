//! Request-bound TVC key acceptance.
//!
//! Submission receipts establish which requests this client sent. Only an
//! independently pinned Signer quorum key and AWS boot evidence authenticate
//! the ephemeral key that signs each returned claim.

mod claims;
mod endorsement;
mod facts;
mod keys;

pub use endorsement::ENDORSEMENT_PREFIX;
pub use facts::{
    AcceptanceEvidence, AcceptanceFacts, AcceptanceStage, Accepted, AttestationEvidence,
    BootEvidence, Check, CheckState, ClaimEvidence, EnvelopeFacts, OperatorResultEvidence,
    ProofCheck, QuorumResultEvidence, Rejected,
};
pub use keys::{
    BootProofLookupError, OperatorExpectation, P256PublicKey, QuorumExpectation,
    TvcProvenanceError, VerifiedOperatorKeys, VerifiedOperatorParts, VerifiedQuorumKey,
};

use crate::EphemeralPublicKey;
use claims::{Claim, Context, OperatorClaim, QuorumClaim};
use endorsement::{ParsedAppProof, VerifiedBoot};
use facts::{AcceptanceTrace, BootObservation, CheckOutcome, FailedCheck};
use std::sync::Arc;
use turnkey_api_key_stamper::Stamp;
use turnkey_client::{
    ActivityReceipt, RequestFingerprint, SubmissionCheckpoint, TurnkeyClient,
    generated::{
        Activity, ActivityStatus, ActivityType, CreateTvcOperatorResult, CreateTvcQuorumKeyResult,
        external::data::v1::{AppProof, BootProof},
        immutable::activity::v1::result::Inner as ActivityResult,
    },
};
use uuid::Uuid;

/// Unverified response data and its proof envelopes.
struct RawActivity {
    response: ActivityResponse,
    app_proofs: Vec<AppProof>,
}

/// Coordinator fields retained for comparison and diagnostics after the proofs are consumed.
struct ActivityResponse {
    status: ActivityStatus,
    kind: ActivityType,
    returned_result: Option<ActivityResult>,
}

impl ActivityResponse {
    /// Requires a completed operator creation with an operator result.
    fn completed_operator_result(&self) -> CheckOutcome<&CreateTvcOperatorResult> {
        CheckOutcome::run(ProofCheck::ActivityShape, || {
            if self.status != ActivityStatus::Completed
                || self.kind != ActivityType::CreateTvcOperator
            {
                return Err(TvcProvenanceError::ClaimMismatch(
                    "operator activity type/status",
                ));
            }
            match &self.returned_result {
                Some(ActivityResult::CreateTvcOperatorResult(result)) => Ok(result),
                _ => Err(TvcProvenanceError::ClaimMismatch("operator result variant")),
            }
        })
    }

    /// Requires a completed quorum key creation with a quorum key result.
    fn completed_quorum_result(&self) -> CheckOutcome<&CreateTvcQuorumKeyResult> {
        CheckOutcome::run(ProofCheck::ActivityShape, || {
            if self.status != ActivityStatus::Completed
                || self.kind != ActivityType::CreateTvcQuorumKey
            {
                return Err(TvcProvenanceError::ClaimMismatch(
                    "quorum activity type/status",
                ));
            }
            match &self.returned_result {
                Some(ActivityResult::CreateTvcQuorumKeyResult(result)) => Ok(result),
                _ => Err(TvcProvenanceError::ClaimMismatch("quorum result variant")),
            }
        })
    }
}

impl From<Activity> for RawActivity {
    fn from(activity: Activity) -> Self {
        Self {
            response: ActivityResponse {
                status: activity.status,
                kind: activity.r#type,
                returned_result: activity.result.and_then(|result| result.inner),
            },
            app_proofs: activity.app_proofs,
        }
    }
}

/// Opaque capability for the selected independent TVC endorsement policy.
pub struct TvcProvenance<S: Stamp> {
    client: TurnkeyClient<S>,
    endorsement: endorsement::SelectedEndorsement,
}

impl<S: Stamp> TryFrom<TurnkeyClient<S>> for TvcProvenance<S> {
    type Error = TvcProvenanceError;

    /// Selects the trusted Signer policy from the client's API base URL.
    fn try_from(client: TurnkeyClient<S>) -> Result<Self, Self::Error> {
        let endorsement = endorsement::SelectedEndorsement::try_from(client.base_url())?;
        Ok(Self {
            client,
            endorsement,
        })
    }
}

impl<S: Stamp> TryFrom<(TurnkeyClient<S>, &str)> for TvcProvenance<S> {
    type Error = TvcProvenanceError;

    /// Binds a custom transport client to an independently selected built-in API environment.
    fn try_from((client, api_url): (TurnkeyClient<S>, &str)) -> Result<Self, Self::Error> {
        let endorsement = endorsement::SelectedEndorsement::try_from(api_url)?;
        Ok(Self {
            client,
            endorsement,
        })
    }
}

impl<S: Stamp> TvcProvenance<S> {
    /// The owned client, borrowed for creation or activity lookups before acceptance.
    pub fn client(&self) -> &TurnkeyClient<S> {
        &self.client
    }

    /// Authenticates both operator roles against the caller's intent and submitted request.
    /// Both acceptance and rejection retain the facts collected during verification.
    pub async fn accept_operator(
        self,
        expected: OperatorExpectation,
        receipt: ActivityReceipt,
    ) -> Result<Accepted<VerifiedOperatorKeys>, Rejected> {
        let (activity, checkpoint) = receipt.into_parts();
        let RawActivity {
            response,
            app_proofs,
        } = activity.into();
        let count = app_proofs.len();
        let mut trace = AcceptanceTrace::default();
        let verification = self
            .verify_operator(&expected, &response, app_proofs, &checkpoint, &mut trace)
            .await;
        let returned = match response.returned_result {
            Some(ActivityResult::CreateTvcOperatorResult(result)) => Some(OperatorResultEvidence {
                wallet: result.wallet_id,
                encrypt: result.encrypt_public_key,
                sign: result.sign_public_key,
                operator_id: result.operator_id,
            }),
            _ => None,
        };
        AcceptanceFacts::assemble(
            self.endorsement.environment,
            self.endorsement.quorum_key,
            AcceptanceEvidence::Operator {
                organization: expected.organization,
                wallet: expected.wallet,
                base_path: expected.base_path,
                returned,
            },
            checkpoint.sent_fingerprints().copied().collect(),
            count,
            trace,
        )
        .finish(verification)
    }

    /// Verifies both operator roles against the requested wallet and derivation paths.
    /// Both claims must match one local submission and the returned wallet and keys.
    async fn verify_operator(
        &self,
        expected: &OperatorExpectation,
        response: &ActivityResponse,
        proofs: Vec<AppProof>,
        checkpoint: &SubmissionCheckpoint,
        trace: &mut AcceptanceTrace,
    ) -> Result<VerifiedOperatorKeys, FailedCheck> {
        let returned = trace.checks.record(response.completed_operator_result())?;
        let claims = self.claims(&expected.organization, proofs, trace).await?;
        let roles = trace
            .checks
            .record(OperatorRoleClaims::from_claims(claims, expected))?;
        let fingerprint = trace.checks.record(match_common_submission(
            checkpoint,
            &[&roles.encrypt.context, &roles.sign.context],
        ))?;
        trace.matched_fingerprint = Some(fingerprint);
        trace.checks.record(roles.check_distinct_keys())?;
        trace.checks.record(roles.check_returned_result(returned))?;
        Ok(VerifiedOperatorKeys {
            wallet: roles.encrypt.body.wallet_id,
            base_path: expected.base_path.clone(),
            encrypt_key: roles.encrypt.body.address,
            signing_key: roles.sign.body.address,
            untrusted_operator_id: returned.operator_id.clone(),
        })
    }

    /// Authenticates the quorum key against the caller's intent and submitted request.
    /// Both acceptance and rejection retain the facts collected during verification.
    pub async fn accept_quorum(
        self,
        expected: QuorumExpectation,
        receipt: ActivityReceipt,
    ) -> Result<Accepted<VerifiedQuorumKey>, Rejected> {
        let (activity, checkpoint) = receipt.into_parts();
        let RawActivity {
            response,
            app_proofs,
        } = activity.into();
        let count = app_proofs.len();
        let mut trace = AcceptanceTrace::default();
        let verification = self
            .verify_quorum(&expected, &response, app_proofs, &checkpoint, &mut trace)
            .await;
        let returned = match response.returned_result {
            Some(ActivityResult::CreateTvcQuorumKeyResult(result)) => Some(QuorumResultEvidence {
                id: result.quorum_key_id,
                public_key: result.quorum_public_key,
                share_ids: result.share_ids,
            }),
            _ => None,
        };
        AcceptanceFacts::assemble(
            self.endorsement.environment,
            self.endorsement.quorum_key,
            AcceptanceEvidence::Quorum {
                organization: expected.organization,
                threshold: expected.threshold,
                recipients: expected.recipients,
                returned,
            },
            checkpoint.sent_fingerprints().copied().collect(),
            count,
            trace,
        )
        .finish(verification)
    }

    async fn verify_quorum(
        &self,
        expected: &QuorumExpectation,
        response: &ActivityResponse,
        proofs: Vec<AppProof>,
        checkpoint: &SubmissionCheckpoint,
        trace: &mut AcceptanceTrace,
    ) -> Result<VerifiedQuorumKey, FailedCheck> {
        let returned = trace.checks.record(response.completed_quorum_result())?;
        let claims = self.claims(&expected.organization, proofs, trace).await?;
        let claim = trace.checks.record(check_quorum_claim(claims, expected))?;
        let fingerprint = trace
            .checks
            .record(match_common_submission(checkpoint, &[&claim.context]))?;
        trace.matched_fingerprint = Some(fingerprint);
        let id = trace
            .checks
            .record(check_quorum_result_binding(returned, &claim))?;
        let untrusted_share_ids = trace.checks.record(parse_share_routing_labels(
            &returned.share_ids,
            expected.recipients.len(),
        ))?;
        Ok(VerifiedQuorumKey {
            id,
            public_key: claim.body.quorum_public_key,
            untrusted_share_ids,
        })
    }

    /// Authenticates and decodes key claims, skipping policy outcomes.
    /// Reuses verified boot evidence within this call and checks every app signature.
    async fn claims(
        &self,
        organization: &str,
        proofs: Vec<AppProof>,
        trace: &mut AcceptanceTrace,
    ) -> Result<Vec<Claim>, FailedCheck> {
        trace.checks.record(check_envelope_count(proofs.len()))?;
        // Entries live only for this acceptance's organization and selected endorsement policy.
        let mut boots: Vec<(VerifiedBoot, usize)> = Vec::new();
        let mut claims = Vec::new();
        for (index, proof) in proofs.into_iter().enumerate() {
            trace.envelopes.push(EnvelopeFacts::new(index));
            let (entry, previous) = trace
                .envelopes
                .split_last_mut()
                .expect("current proof envelope");
            let policy = entry
                .checks
                .record(claims::policy_envelope(&proof.proof_payload))?;
            if policy {
                entry.policy_excluded = true;
                continue;
            }
            let parsed = entry
                .checks
                .record(ParsedAppProof::check_ephemeral_key(proof))?;
            entry.ephemeral_key = Some(parsed.key().clone());
            let boot = match boots.iter().find(|(boot, _)| boot.key() == parsed.key()) {
                Some((boot, source)) => {
                    entry.boot = previous[*source].boot.clone();
                    entry.boot_source_envelope = Some(*source);
                    boot
                }
                None => {
                    let lookup = self.lookup_boot_proof(organization, parsed.key()).await;
                    let raw = entry.checks.record(lookup)?;
                    let mut evidence = BootObservation::default();
                    let verified = self.endorsement.authenticate_boot(
                        parsed.key(),
                        raw,
                        &mut entry.checks,
                        &mut evidence,
                    );
                    entry.boot = Some(Arc::new(evidence));
                    boots.push((verified?, index));
                    &boots.last().expect("verified boot entry").0
                }
            };
            let authenticated = entry
                .checks
                .record(boot.check_application_signature(parsed))?;
            let claim = entry.checks.record(Claim::decode(authenticated))?;
            entry.claim = Some(ClaimEvidence::from(&claim));
            claims.push(claim);
        }
        Ok(claims)
    }

    async fn lookup_boot_proof(
        &self,
        organization: &str,
        key: &EphemeralPublicKey,
    ) -> CheckOutcome<BootProof> {
        let result =
            crate::get_boot_proof_for_key(&self.client, organization.to_owned(), key.to_string())
                .await
                .map_err(|error| TvcProvenanceError::BootProofLookup(error.into()))
                .and_then(|response| {
                    response
                        .boot_proof
                        .ok_or(TvcProvenanceError::ProofUnavailable)
                });
        CheckOutcome::new(ProofCheck::BootProofLookup, result)
    }
}

struct OperatorRoleClaims {
    encrypt: OperatorClaim,
    sign: OperatorClaim,
}

impl OperatorRoleClaims {
    /// Requires one claim for each requested key path. Both must be for the
    /// requested organization and the same wallet, which must match any wallet the caller selected.
    fn from_claims(claims: Vec<Claim>, expected: &OperatorExpectation) -> CheckOutcome<Self> {
        CheckOutcome::run(ProofCheck::ClaimSetAndIntent, || {
            let encrypt_path = format!("{}/0", expected.base_path);
            let sign_path = format!("{}/1", expected.base_path);
            let mut encrypt = None;
            let mut sign = None;
            for claim in claims {
                let Claim::Operator(claim) = claim else {
                    return Err(TvcProvenanceError::ClaimMismatch("unexpected quorum claim"));
                };
                if claim.body.organization_id != expected.organization {
                    return Err(TvcProvenanceError::ClaimMismatch("operator organization"));
                }
                let slot = if claim.body.derivation_path == encrypt_path {
                    &mut encrypt
                } else if claim.body.derivation_path == sign_path {
                    &mut sign
                } else {
                    return Err(TvcProvenanceError::ClaimMismatch("operator role path"));
                };
                if slot.replace(claim).is_some() {
                    return Err(TvcProvenanceError::MalformedEvidence(
                        "duplicate operator role claim".into(),
                    ));
                }
            }
            let (encrypt, sign) = encrypt
                .zip(sign)
                .ok_or(TvcProvenanceError::ProofUnavailable)?;
            if encrypt.body.wallet_id != sign.body.wallet_id
                || expected
                    .wallet
                    .as_ref()
                    .is_some_and(|wallet| wallet != &encrypt.body.wallet_id)
            {
                return Err(TvcProvenanceError::ClaimMismatch("operator wallet"));
            }
            Ok(Self { encrypt, sign })
        })
    }

    /// Requires different public keys for encryption and signing.
    fn check_distinct_keys(&self) -> CheckOutcome<()> {
        CheckOutcome::run(ProofCheck::OutputKeyRoles, || {
            if self.encrypt.body.address == self.sign.body.address {
                return Err(TvcProvenanceError::ClaimMismatch(
                    "operator role keys must differ",
                ));
            }
            Ok(())
        })
    }

    /// Checks that the returned wallet and both public keys match the signed claims.
    fn check_returned_result(&self, returned: &CreateTvcOperatorResult) -> CheckOutcome<()> {
        CheckOutcome::run(ProofCheck::ResultBinding, || {
            if returned.wallet_id != self.encrypt.body.wallet_id
                || returned.encrypt_public_key.parse().ok() != Some(self.encrypt.body.address)
                || returned.sign_public_key.parse().ok() != Some(self.sign.body.address)
            {
                return Err(TvcProvenanceError::ClaimMismatch(
                    "operator result wallet/keys",
                ));
            }
            Ok(())
        })
    }
}

/// Requires one quorum key claim for the requested organization, threshold, and
/// recipient keys in their original order.
fn check_quorum_claim(
    claims: Vec<Claim>,
    expected: &QuorumExpectation,
) -> CheckOutcome<QuorumClaim> {
    CheckOutcome::run(ProofCheck::ClaimSetAndIntent, || {
        let mut claims = claims.into_iter();
        let first = claims.next().ok_or(TvcProvenanceError::ProofUnavailable)?;
        if claims.next().is_some() {
            return Err(TvcProvenanceError::MalformedEvidence(
                "duplicate/conflicting quorum claim".into(),
            ));
        }
        let Claim::Quorum(claim) = first else {
            return Err(TvcProvenanceError::ClaimMismatch(
                "unexpected operator claim",
            ));
        };
        if claim.body.organization_id != expected.organization
            || claim.body.threshold != expected.threshold
            || claim.body.operator_encrypt_keys != expected.recipients
        {
            return Err(TvcProvenanceError::ClaimMismatch(
                "quorum organization/threshold/ordered recipients",
            ));
        }
        Ok(claim)
    })
}

/// Checks that the response matches the signed key ID and full public key.
/// Returns the signed key ID as a UUID.
fn check_quorum_result_binding(
    returned: &CreateTvcQuorumKeyResult,
    claim: &QuorumClaim,
) -> CheckOutcome<Uuid> {
    CheckOutcome::run(ProofCheck::ResultBinding, || {
        let signed_key = claim.body.quorum_public_key.as_ref();
        if returned.quorum_key_id != claim.body.quorum_key_id
            || returned.quorum_public_key.len() != 2 * signed_key.len()
            || hex::decode(&returned.quorum_public_key).as_deref() != Ok(signed_key)
        {
            return Err(TvcProvenanceError::ClaimMismatch("quorum result ID/key"));
        }
        Uuid::parse_str(&claim.body.quorum_key_id).map_err(|_| {
            TvcProvenanceError::MalformedEvidence("quorum key ID must be a UUID".into())
        })
    })
}

/// Requires one UUID label per recipient. The Signer proof does not cover
/// these share IDs, so callers can use them only for routing.
fn parse_share_routing_labels(
    share_ids: &[String],
    recipient_count: usize,
) -> CheckOutcome<Vec<Uuid>> {
    CheckOutcome::run(ProofCheck::ShareRoutingShape, || {
        if share_ids.len() != recipient_count || share_ids.iter().any(String::is_empty) {
            return Err(TvcProvenanceError::ClaimMismatch(
                "quorum share routing labels",
            ));
        }
        share_ids
            .iter()
            .map(|id| {
                Uuid::parse_str(id).map_err(|_| {
                    TvcProvenanceError::MalformedEvidence("share ID must be a UUID".into())
                })
            })
            .collect()
    })
}

/// Caps the number of proofs before inspecting any of them.
fn check_envelope_count(count: usize) -> CheckOutcome<()> {
    CheckOutcome::run(ProofCheck::EnvelopeCount, || {
        if count > 64 {
            return Err(TvcProvenanceError::MalformedEvidence(
                "more than 64 proof envelopes".into(),
            ));
        }
        Ok(())
    })
}

/// Finds one fingerprint that the client sent and every signed claim includes.
fn match_common_submission(
    checkpoint: &SubmissionCheckpoint,
    contexts: &[&Context],
) -> CheckOutcome<RequestFingerprint> {
    CheckOutcome::run(ProofCheck::CommonSubmission, || {
        checkpoint
            .sent_fingerprints()
            .find(|sent| {
                contexts
                    .iter()
                    .all(|context| context.digests.contains(sent))
            })
            .copied()
            .ok_or(TvcProvenanceError::ReceiptMismatch)
    })
}

#[cfg(test)]
mod fixture {
    use super::{
        endorsement::{
            AuthenticatedPayload, ParsedAppProof, SelectedEndorsement, SignerEnvironment,
        },
        facts::{BootObservation, CheckTrace},
    };
    use turnkey_enclave_encrypt::QuorumPublicKey;

    use x509_cert::{
        Certificate,
        der::{Decode, Encode},
    };

    pub(super) fn with_test_roots(
        quorum_key: QuorumPublicKey,
        attestation_root: Certificate,
    ) -> SelectedEndorsement {
        SelectedEndorsement {
            quorum_key,
            environment: SignerEnvironment::Synthetic,
            attestation_root: attestation_root.to_der().unwrap(),
        }
    }

    use crate::{CoseError, CoseSign1, MessageDigest, Sha2, SignatureAlgorithm, SigningPublicKey};
    use aws_nitro_enclaves_cose::{crypto::SigningPrivateKey, header_map::HeaderMap};
    use aws_nitro_enclaves_nsm_api::api::{AttestationDoc, Digest};
    use p256::ecdsa::{Signature, SigningKey, signature::Signer};
    use p384::ecdsa::signature::hazmat::PrehashSigner;
    use qos_core::protocol::{
        QosHash,
        services::boot::{
            Approval, Manifest, ManifestEnvelope, ManifestSet, MemberPubKey, Namespace,
            NitroConfig, PatchSet, PivotConfig, QuorumMember, RestartPolicy, ShareSet,
        },
    };
    use serde_bytes::ByteBuf;
    use std::collections::BTreeMap;
    use turnkey_client::generated::external::data::v1::{AppProof, BootProof, SignatureScheme};

    pub(super) fn signer() -> SelectedEndorsement {
        with_test_roots(
            QuorumPublicKey::try_from(quorum_key().as_slice()).unwrap(),
            Certificate::from_der(ROOT).unwrap(),
        )
    }

    pub(super) fn proof(payload: String) -> AppProof {
        let signature: Signature = SigningKey::from_slice(&[9; 32])
            .unwrap()
            .sign(payload.as_bytes());
        AppProof {
            scheme: SignatureScheme::EphemeralKeyP256,
            public_key: hex::encode(pin()),
            proof_payload: payload,
            signature: hex::encode(signature.to_bytes()),
        }
    }

    /// Sign fixture bytes and authenticate them using the synthetic boot proof.
    pub(super) fn authenticate_payload(payload: String) -> AuthenticatedPayload {
        let parsed = ParsedAppProof::try_from(proof(payload)).unwrap();
        let verified_boot = signer()
            .authenticate_boot(
                parsed.key(),
                boot(),
                &mut CheckTrace::default(),
                &mut BootObservation::default(),
            )
            .unwrap();
        verified_boot.authenticate(parsed).unwrap()
    }

    pub(super) fn point(seed: u8) -> String {
        hex::encode(
            SigningKey::from_slice(&[seed; 32])
                .unwrap()
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes(),
        )
    }

    pub(super) fn mixed_case_hex(hex: &str) -> String {
        let (first, second) = hex.split_at(hex.len() / 2);
        format!("{}{second}", first.to_ascii_uppercase())
    }

    pub(super) fn pin() -> [u8; 130] {
        hex::decode(format!("{}{}", point(8), point(9)))
            .unwrap()
            .try_into()
            .unwrap()
    }

    pub(super) const ROOT: &[u8] =
        include_bytes!("../tests/fixtures/synthetic-attestation/root.der");
    pub(super) const LEAF: &[u8] =
        include_bytes!("../tests/fixtures/synthetic-attestation/leaf.der");
    pub(super) const ISSUED_AT: u64 = 1_717_200_000_000;

    pub(super) fn quorum_key() -> [u8; 130] {
        hex::decode(format!("{}{}", point(10), point(11)))
            .unwrap()
            .try_into()
            .unwrap()
    }
    pub(super) fn sign_document(document: &[u8], prefix: &[u8]) -> String {
        let mut message = prefix.to_vec();
        message.extend_from_slice(document);
        let signature: Signature = SigningKey::from_slice(&[11; 32]).unwrap().sign(&message);
        hex::encode(signature.to_bytes())
    }
    pub(super) fn manifest() -> Manifest {
        let member = QuorumMember {
            alias: "fixture-approver".into(),
            pub_key: pin().to_vec(),
        };
        Manifest {
            namespace: Namespace {
                name: "fixture".into(),
                nonce: 1,
                quorum_key: quorum_key().to_vec(),
            },
            pivot: PivotConfig {
                hash: [9; 32],
                restart: RestartPolicy::Never,
                bridge_config: vec![],
                debug_mode: false,
                args: vec![],
            },
            manifest_set: ManifestSet {
                threshold: 1,
                members: vec![member.clone()],
            },
            share_set: ShareSet {
                threshold: 1,
                members: vec![member.clone()],
            },
            enclave: NitroConfig {
                pcr0: vec![1; 48],
                pcr1: vec![2; 48],
                pcr2: vec![3; 48],
                pcr3: vec![4; 48],
                aws_root_certificate: ROOT.to_vec(),
                qos_commit: "fixture".into(),
            },
            patch_set: PatchSet {
                threshold: 1,
                members: vec![MemberPubKey {
                    pub_key: member.pub_key,
                }],
            },
        }
    }
    pub(super) fn envelope(manifest: Manifest) -> ManifestEnvelope {
        let signature: Signature = SigningKey::from_slice(&[9; 32])
            .unwrap()
            .sign(&manifest.qos_hash());
        ManifestEnvelope {
            manifest_set_approvals: vec![Approval {
                signature: signature.to_bytes().to_vec(),
                member: manifest.manifest_set.members[0].clone(),
            }],
            share_set_approvals: vec![],
            manifest,
        }
    }
    pub(super) fn attestation(manifest: &Manifest) -> AttestationDoc {
        let hash = manifest.qos_hash();
        let pcr17 =
            qos_nsm::nitro::expected_manifest_commitment_pcr::<qos_nsm::nitro::Live>(&hash, &pin())
                .unwrap();
        let mut pcrs = (0..32)
            .map(|index| (index, ByteBuf::from(vec![0; 48])))
            .collect::<BTreeMap<_, _>>();
        pcrs.extend(BTreeMap::from([
            (0, ByteBuf::from(manifest.enclave.pcr0.clone())),
            (1, ByteBuf::from(manifest.enclave.pcr1.clone())),
            (2, ByteBuf::from(manifest.enclave.pcr2.clone())),
            (3, ByteBuf::from(manifest.enclave.pcr3.clone())),
            (17, ByteBuf::from(pcr17.to_vec())),
        ]));
        AttestationDoc {
            module_id: "fixture-nsm".into(),
            digest: Digest::SHA384,
            timestamp: ISSUED_AT,
            pcrs,
            certificate: ByteBuf::from(LEAF.to_vec()),
            cabundle: vec![ByteBuf::from(ROOT.to_vec())],
            public_key: Some(ByteBuf::from(pin().to_vec())),
            user_data: Some(ByteBuf::from(hash.to_vec())),
            nonce: None,
        }
    }
    struct AttestationKey(p384::ecdsa::SigningKey);
    impl SigningPublicKey for AttestationKey {
        fn get_parameters(&self) -> Result<(SignatureAlgorithm, MessageDigest), CoseError> {
            Ok((SignatureAlgorithm::ES384, MessageDigest::Sha384))
        }
        fn verify(&self, digest: &[u8], signature: &[u8]) -> Result<bool, CoseError> {
            SigningPublicKey::verify(
                &crate::P384PubKey(p384::PublicKey::from(self.0.verifying_key())),
                digest,
                signature,
            )
        }
    }
    impl SigningPrivateKey for AttestationKey {
        fn sign(&self, digest: &[u8]) -> Result<Vec<u8>, CoseError> {
            let signature: p384::ecdsa::Signature = self.0.sign_prehash(digest).unwrap();
            Ok(signature.to_bytes().to_vec())
        }
    }
    pub(super) fn document(doc: &AttestationDoc) -> Vec<u8> {
        CoseSign1::new::<Sha2>(
            &doc.to_binary(),
            &HeaderMap::new(),
            &AttestationKey(p384::ecdsa::SigningKey::from_slice(&[42; 48]).unwrap()),
        )
        .unwrap()
        .as_bytes(true)
        .unwrap()
    }
    pub(super) fn boot_from(manifest: Manifest, doc: &AttestationDoc) -> BootProof {
        let bytes = document(doc);
        BootProof {
            ephemeral_public_key_hex: hex::encode(pin()),
            aws_attestation_doc_b64: base64::encode(&bytes),
            qos_manifest_b64: base64::encode(borsh::to_vec(&manifest).unwrap()),
            qos_manifest_envelope_b64: base64::encode(borsh::to_vec(&envelope(manifest)).unwrap()),
            deployment_label: "synthetic-test".into(),
            enclave_app: "signer".into(),
            owner: String::new(),
            created_at: None,
            qos_manifest_version: Some("1".into()),
            quorum_key_attestation_signature_hex: Some(sign_document(
                &bytes,
                super::endorsement::ENDORSEMENT_PREFIX,
            )),
        }
    }
    pub(super) fn boot() -> BootProof {
        let manifest = manifest();
        let doc = attestation(&manifest);
        boot_from(manifest, &doc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use facts::{
        CheckState::{Fail, Pass},
        CheckTrace,
        ProofCheck::*,
    };
    use fixture::{authenticate_payload, mixed_case_hex, pin, point, proof};
    use p256::ecdsa::{Signature, SigningKey, signature::Signer};
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    use std::collections::HashSet;
    use std::{sync::Mutex, time::Duration};
    use turnkey_client::{
        RetryConfig, TurnkeyClient, TurnkeyP256ApiKey,
        generated::{
            CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent,
            external::data::v1::{AppProof, BootProof, SignatureScheme},
        },
    };

    use turnkey_enclave_encrypt::QuorumPublicKey;
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate, matchers::method};
    use x509_cert::{Certificate, der::Decode};

    const ORGANIZATION_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";

    fn completed<T>(outcome: turnkey_client::SubmissionOutcome<T>) -> T {
        match outcome {
            turnkey_client::SubmissionOutcome::Completed(value) => value,
            turnkey_client::SubmissionOutcome::Pending { .. } => {
                panic!("expected completed proof fixture")
            }
        }
    }

    fn verifier(client: TurnkeyClient<TurnkeyP256ApiKey>) -> TvcProvenance<TurnkeyP256ApiKey> {
        TvcProvenance {
            client,
            endorsement: fixture::signer(),
        }
    }
    fn operator_payload(role: usize, digests: Vec<String>) -> Value {
        json!({"type":"APP_PROOF_TYPE_ADDRESS_DERIVATION", "timestampMs":"1793845997611", "requestContext":{"version":1,"verifiedSignatureDigests":digests}, "addressDerivationProof":{"organizationId":ORGANIZATION_ID,"walletId":"wallet-1","derivationPath":format!("m/0/{role}"),"address":point(role as u8 + 1)}})
    }
    fn quorum_payload(digests: Vec<String>) -> Value {
        json!({"type":"APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE", "timestampMs":"1793845997611", "requestContext":{"version":1,"verifiedSignatureDigests":digests}, "tvcQuorumKeyCreateProof":{"organizationId":ORGANIZATION_ID,"quorumKeyId":"11111111-1111-4111-8111-111111111111","quorumPublicKey":format!("{}{}",point(3),point(4)),"threshold":2,"operatorEncryptKeys":[point(1),point(2)]}})
    }
    fn digest(request: &Request) -> String {
        let bytes = base64::decode_config(
            request.headers.get("x-stamp").unwrap().to_str().unwrap(),
            base64::URL_SAFE_NO_PAD,
        )
        .unwrap();
        let stamp: Value = serde_json::from_slice(&bytes).unwrap();
        let mut hash = Sha256::new();
        hash.update(1i32.to_le_bytes());
        for value in [
            stamp["publicKey"].as_str().unwrap(),
            std::str::from_utf8(&request.body).unwrap(),
            stamp["signature"].as_str().unwrap(),
        ] {
            hash.update((value.len() as u32).to_le_bytes());
            hash.update(value.as_bytes());
        }
        hex::encode(hash.finalize())
    }

    #[test]
    fn selects_expected_signer_pin_for_each_supported_environment() {
        for (url, expected) in [
            (
                "https://api.turnkey.com",
                QuorumPublicKey::production_signer(),
            ),
            (
                "https://api.preprod.turnkey.engineering",
                QuorumPublicKey::preprod_signer(),
            ),
            (
                "https://api.dev.turnkey.engineering",
                QuorumPublicKey::dev_signer(),
            ),
        ] {
            let verifier = TvcProvenance::try_from(capture_client(url.into())).unwrap();
            assert_eq!(verifier.client().base_url(), url);
            assert_eq!(verifier.endorsement.quorum_key, expected, "{url}");
        }
        for url in [
            "http://127.0.0.1:1",
            "https://evil.test",
            "https://api.turnkey.com.evil.test",
        ] {
            assert!(matches!(
                TvcProvenance::try_from(capture_client(url.to_owned())),
                Err(TvcProvenanceError::UnsupportedEnvironment)
            ));
        }
    }

    #[test]
    fn custom_transport_requires_a_known_policy() {
        let client = capture_client("http://127.0.0.1:1".into());
        let verifier = TvcProvenance::try_from((client, "https://api.turnkey.com")).unwrap();
        assert_eq!(verifier.client().base_url(), "http://127.0.0.1:1");
        assert_eq!(
            &verifier.endorsement.quorum_key,
            &QuorumPublicKey::production_signer()
        );
        let client = capture_client("http://127.0.0.1:1".into());
        assert!(matches!(
            TvcProvenance::try_from((client, "https://evil.test")),
            Err(TvcProvenanceError::UnsupportedEnvironment)
        ));
    }

    #[test]
    fn actual_tkms_claim_bytes_decode_and_verify_under_their_fixture_ephemeral_key() {
        let proofs: Vec<AppProof> = serde_json::from_str(include_str!(
            "../tests/fixtures/tvc_request_bound_proofs.json"
        ))
        .unwrap();
        assert_eq!(proofs.len(), 3);
        for p in &proofs {
            crate::verify_app_proof_signature(p).unwrap();
            // Use the synthetic boot chain to decode the same payload bytes.
            assert!(Claim::try_from(authenticate_payload(p.proof_payload.clone())).is_ok());
        }
        let mut changed = proofs[0].clone();
        changed.proof_payload.push(' ');
        assert!(crate::verify_app_proof_signature(&changed).is_err());
    }

    #[test]
    fn application_signature_covers_original_payload_and_both_ephemeral_keys() {
        let payload =
            serde_json::to_string_pretty(&operator_payload(0, vec!["ab".repeat(32)])).unwrap();
        let original = proof(payload.clone());
        let parsed = ParsedAppProof::try_from(original.clone()).unwrap();
        let verifier = verifier(capture_client("http://127.0.0.1:1".into()));
        let boot = verifier
            .endorsement
            .authenticate_boot(
                parsed.key(),
                fixture::boot(),
                &mut CheckTrace::default(),
                &mut BootObservation::default(),
            )
            .unwrap();
        let check = |parsed| CheckTrace::default().record(boot.check_application_signature(parsed));
        assert_eq!(check(parsed).unwrap().into_inner(), payload);

        let reformatted =
            serde_json::to_string(&serde_json::from_str::<Value>(&payload).unwrap()).unwrap();
        for changed in [
            AppProof {
                proof_payload: format!("{payload} "),
                ..original.clone()
            },
            AppProof {
                proof_payload: reformatted,
                ..original.clone()
            },
            AppProof {
                signature: "00".repeat(64),
                ..original.clone()
            },
            AppProof {
                scheme: SignatureScheme::Unspecified,
                ..original.clone()
            },
            AppProof {
                public_key: format!("{}{}", point(7), &original.public_key[130..]),
                ..original.clone()
            },
            AppProof {
                public_key: format!("{}{}", &original.public_key[..130], point(7)),
                ..original
            },
        ] {
            assert!(matches!(
                check(ParsedAppProof::try_from(changed).unwrap()),
                Err(FailedCheck {
                    source: TvcProvenanceError::EndorsementFailure,
                    ..
                })
            ));
        }
    }

    #[test]
    fn app_envelope_requires_two_valid_ephemeral_points() {
        let valid = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
        let check =
            |proof| CheckTrace::default().record(ParsedAppProof::check_ephemeral_key(proof));
        assert_eq!(
            check(valid.clone()).unwrap().key().to_string(),
            valid.public_key
        );
        for key in [
            point(9),
            format!("{}{}", "00".repeat(65), &valid.public_key[130..]),
            format!("{}{}", &valid.public_key[..130], "00".repeat(65)),
        ] {
            assert!(matches!(
                check(AppProof {
                    public_key: key,
                    ..valid.clone()
                }),
                Err(FailedCheck {
                    source: TvcProvenanceError::MalformedEvidence(_),
                    ..
                })
            ));
        }
    }

    #[test]
    fn unsupported_request_context_version_has_a_specific_error() {
        let mut payload = operator_payload(0, vec!["ab".repeat(32)]);
        payload["requestContext"]["version"] = json!(2);

        assert!(matches!(
            Claim::try_from(authenticate_payload(payload.to_string())),
            Err(TvcProvenanceError::UnsupportedContext)
        ));
    }

    #[test]
    fn a_string_request_context_version_is_malformed() {
        let mut payload = operator_payload(0, vec!["ab".repeat(32)]);
        payload["requestContext"]["version"] = json!("1");

        assert!(matches!(
            Claim::try_from(authenticate_payload(payload.to_string())),
            Err(TvcProvenanceError::MalformedEvidence(_))
        ));
    }

    #[test]
    fn v1_claim_rejects_mixed_case_request_fingerprint() {
        let digest = mixed_case_hex(&"ab".repeat(32));
        let payload = operator_payload(0, vec![digest]);
        assert!(matches!(
            Claim::try_from(authenticate_payload(payload.to_string())),
            Err(TvcProvenanceError::MalformedEvidence(message)) if message.contains("lowercase")
        ));
    }

    #[test]
    fn strict_wire_mutation_matrix() {
        let baseline = operator_payload(0, vec!["ab".repeat(32)]);
        let mut cases = Vec::new();
        for field in [
            "type",
            "timestampMs",
            "requestContext",
            "addressDerivationProof",
        ] {
            let mut v = baseline.clone();
            v.as_object_mut().unwrap().remove(field);
            cases.push(v);
            let mut v = baseline.clone();
            v[field] = Value::Null;
            cases.push(v);
        }
        for field in ["organizationId", "walletId", "derivationPath", "address"] {
            for wrong in [Value::Null, json!(""), json!(2), json!([])] {
                let mut v = baseline.clone();
                v["addressDerivationProof"][field] = wrong;
                cases.push(v);
            }
            let mut v = baseline.clone();
            v["addressDerivationProof"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            cases.push(v);
        }
        for wrong in [
            json!(1),
            json!(""),
            json!("+1"),
            json!("1e2"),
            json!("18446744073709551616"),
        ] {
            let mut v = baseline.clone();
            v["timestampMs"] = wrong;
            cases.push(v);
        }
        for wrong in [json!(1.0), json!(-1), Value::Null] {
            let mut v = baseline.clone();
            v["requestContext"]["version"] = wrong;
            cases.push(v);
        }
        for wrong in [
            json!([]),
            json!([null]),
            json!(["AB".repeat(32)]),
            json!(["00"]),
            json!(["gg".repeat(32)]),
            json!(vec!["ab".repeat(32); 1025]),
        ] {
            let mut v = baseline.clone();
            v["requestContext"]["verifiedSignatureDigests"] = wrong;
            cases.push(v);
        }
        let mut v = baseline.clone();
        v["request_context"] = json!({});
        cases.push(v);
        let mut v = baseline.clone();
        v["requestContext"]["unknown"] = json!(true);
        cases.push(v);
        let mut v = baseline.clone();
        v["addressDerivationProof"]["wallet_id"] = json!("wallet-1");
        cases.push(v);
        let mut v = baseline.clone();
        v["tvcQuorumKeyCreateProof"] = json!({});
        cases.push(v);
        let mut v = baseline.clone();
        v["type"] = json!("APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE");
        cases.push(v);
        for (index, v) in cases.into_iter().enumerate() {
            assert!(
                Claim::try_from(authenticate_payload(v.to_string())).is_err(),
                "accepted mutation {index}: {v}"
            );
        }
        let raw = baseline.to_string();
        for invalid in [
            format!("{raw} true"),
            raw.replacen(
                "\"timestampMs\":",
                "\"timestampMs\":\"1\",\"timestampMs\":",
                1,
            ),
            raw.replacen("\"version\":1", "\"version\":1,\"version\":1", 1),
            raw.replacen("\"walletId\":", "\"walletId\":\"other\",\"walletId\":", 1),
            raw.replacen("\"version\":1", "\"version\":1e0", 1),
        ] {
            assert!(
                Claim::try_from(authenticate_payload(invalid.clone())).is_err(),
                "accepted {invalid}"
            );
        }
    }

    fn signed_quorum_claim_with_key(key: &str) -> Result<Claim, TvcProvenanceError> {
        let mut payload = quorum_payload(vec!["ab".repeat(32)]);
        payload["tvcQuorumKeyCreateProof"]["quorumPublicKey"] = json!(key);
        Claim::try_from(authenticate_payload(payload.to_string()))
    }

    #[test]
    fn signed_quorum_claim_accepts_mixed_case_key_hex() {
        let key = format!("{}{}", point(3), point(4));
        let Claim::Quorum(claim) = signed_quorum_claim_with_key(&mixed_case_hex(&key)).unwrap()
        else {
            panic!("quorum fixture");
        };
        assert_eq!(
            claim.body.quorum_public_key.as_ref(),
            hex::decode(&key).unwrap()
        );
        let returned = CreateTvcQuorumKeyResult {
            quorum_key_id: claim.body.quorum_key_id.clone(),
            quorum_public_key: format!("{}{}", point(3), point(4).to_ascii_uppercase()),
            share_ids: vec![],
        };
        assert!(
            CheckTrace::default()
                .record(check_quorum_result_binding(&returned, &claim))
                .is_ok()
        );
    }

    #[test]
    fn signed_quorum_claim_rejects_malformed_key_hex() {
        let key = format!("g{}{}", &point(3)[1..], point(4));
        assert!(matches!(
            signed_quorum_claim_with_key(&key),
            Err(TvcProvenanceError::MalformedEvidence(_))
        ));
    }

    #[test]
    fn signed_quorum_claim_rejects_invalid_encryption_point() {
        let key = format!("04{}{}", "00".repeat(64), point(4));
        assert!(matches!(
            signed_quorum_claim_with_key(&key),
            Err(TvcProvenanceError::MalformedEvidence(_))
        ));
    }

    #[test]
    fn signed_quorum_claim_rejects_invalid_signing_point() {
        let key = format!("{}04{}", point(3), "00".repeat(64));
        assert!(matches!(
            signed_quorum_claim_with_key(&key),
            Err(TvcProvenanceError::MalformedEvidence(_))
        ));
    }

    #[test]
    fn application_signature_rejects_case_change_inside_quorum_claim_after_signing() {
        let original = proof(quorum_payload(vec!["ab".repeat(32)]).to_string());
        let key = format!("{}{}", point(3), point(4));
        let parsed = ParsedAppProof::try_from(AppProof {
            proof_payload: original.proof_payload.replace(&key, &mixed_case_hex(&key)),
            ..original
        })
        .unwrap();
        let boot = fixture::signer()
            .authenticate_boot(
                parsed.key(),
                fixture::boot(),
                &mut CheckTrace::default(),
                &mut BootObservation::default(),
            )
            .unwrap();
        assert!(matches!(
            CheckTrace::default().record(boot.check_application_signature(parsed)),
            Err(FailedCheck {
                source: TvcProvenanceError::EndorsementFailure,
                ..
            })
        ));
    }

    #[test]
    fn role_key_requires_an_uncompressed_p256_point() {
        for invalid in [
            format!(" {}", point(1)),
            format!("0x{}", point(1)),
            format!("02{}", &point(1)[2..66]),
            "00".repeat(65),
            format!("04{}", "ff".repeat(64)),
            format!("{}00", point(1)),
        ] {
            assert!(invalid.parse::<P256PublicKey>().is_err());
        }
    }

    #[test]
    fn maximum_quorum_claim_boundaries() {
        let mut largest = quorum_payload(vec!["ab".repeat(32); 1024]);
        let recipients: Vec<String> = (1u32..=255)
            .map(|i| {
                let mut scalar = [0; 32];
                scalar[28..].copy_from_slice(&i.to_be_bytes());
                hex::encode(
                    SigningKey::from_slice(&scalar)
                        .unwrap()
                        .verifying_key()
                        .to_encoded_point(false)
                        .as_bytes(),
                )
            })
            .collect();
        largest["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] = json!(recipients);
        largest["tvcQuorumKeyCreateProof"]["threshold"] = json!(255);
        assert!(Claim::try_from(authenticate_payload(largest.to_string())).is_ok());
        assert!(largest.to_string().len() < 256 * 1024);
        largest["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"]
            .as_array_mut()
            .unwrap()
            .push(json!(point(7)));
        assert!(Claim::try_from(authenticate_payload(largest.to_string())).is_err());
    }

    #[derive(Clone, Copy)]
    enum Mode {
        Operator,
        Quorum,
        Split,
        Common,
        SubstituteKey,
        QuorumSubstitutedKey,
        EarlierClaim,
    }

    impl Mode {
        fn is_quorum(self) -> bool {
            matches!(self, Self::Quorum | Self::QuorumSubstitutedKey)
        }
    }

    struct Producer {
        mode: Mode,
        first: Mutex<Option<String>>,
    }

    impl Respond for Producer {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let sent = digest(request);
            let mut first = self.first.lock().unwrap();
            let pending = matches!(self.mode, Mode::Split | Mode::Common) && first.is_none();
            let first_digest = first.get_or_insert_with(|| sent.clone()).clone();
            let (kind, mut proofs, result) = if self.mode.is_quorum() {
                let public_key = if matches!(self.mode, Mode::QuorumSubstitutedKey) {
                    format!("{}{}", point(5), point(6))
                } else {
                    format!("{}{}", point(3), point(4))
                };
                (
                    "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY",
                    vec![proof(quorum_payload(vec![sent]).to_string())],
                    json!({"createTvcQuorumKeyResult": {
                        "quorumKeyId": "11111111-1111-4111-8111-111111111111",
                        "quorumPublicKey": public_key,
                        "shareIds": [
                            "22222222-2222-4222-8222-222222222222",
                            "33333333-3333-4333-8333-333333333333"
                        ],
                    }}),
                )
            } else {
                let proofs = (0..2)
                    .map(|role| {
                        let digests = match self.mode {
                            Mode::Split if role == 0 => vec![first_digest.clone()],
                            Mode::Common => vec![first_digest.clone(), sent.clone()],
                            Mode::EarlierClaim => vec![first_digest.clone()],
                            _ => vec![sent.clone()],
                        };
                        proof(operator_payload(role, digests).to_string())
                    })
                    .collect();
                let encrypt_key = if matches!(self.mode, Mode::SubstituteKey) {
                    point(7)
                } else {
                    point(1)
                };
                (
                    "ACTIVITY_TYPE_CREATE_TVC_OPERATOR",
                    proofs,
                    json!({"createTvcOperatorResult": {
                        "walletId": "wallet-1",
                        "operatorId": "routing-label",
                        "encryptPublicKey": encrypt_key,
                        "signPublicKey": point(2),
                    }}),
                )
            };
            proofs.push(AppProof {
                scheme: SignatureScheme::Unspecified,
                public_key: "unrelated-ump-key".into(),
                proof_payload: r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#.into(),
                signature: String::new(),
            });
            let status = if pending {
                "ACTIVITY_STATUS_PENDING"
            } else {
                "ACTIVITY_STATUS_COMPLETED"
            };
            ResponseTemplate::new(200).set_body_json(json!({"activity": {
                "id": "activity-1",
                "organizationId": ORGANIZATION_ID,
                "type": kind,
                "status": status,
                "fingerprint": "untrusted",
                "appProofs": proofs,
                "result": result,
            }}))
        }
    }

    async fn receipt(mode: Mode) -> ActivityReceipt {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(Producer {
                mode,
                first: Mutex::new(None),
            })
            .mount(&server)
            .await;
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::from_bytes([6; 32], None).unwrap())
            .base_url(server.uri())
            .retry_config(RetryConfig {
                initial_delay: Duration::from_millis(1),
                multiplier: 1.0,
                max_delay: Duration::from_millis(1),
                max_retries: 3,
            })
            .build()
            .unwrap()
            .with_app_proofs();
        let outcome = if mode.is_quorum() {
            client
                .create_tvc_quorum_key_with_receipt(
                    ORGANIZATION_ID.parse().unwrap(),
                    100,
                    CreateTvcQuorumKeyIntent {
                        threshold: 2,
                        operator_encrypt_keys: vec![point(1), point(2)],
                    },
                    async |_| Ok(()),
                )
                .await
                .map(|outcome| completed(outcome).into_parts().1)
        } else {
            client
                .create_tvc_operator_with_receipt(
                    ORGANIZATION_ID.parse().unwrap(),
                    100,
                    CreateTvcOperatorIntent {
                        wallet_name: Some("wallet".into()),
                        wallet_id: None,
                        path: "m/0".into(),
                        operator_name: "synthetic-predicate-test".into(),
                    },
                    async |_| Ok(()),
                )
                .await
                .map(|outcome| completed(outcome).into_parts().1)
        };
        outcome.unwrap()
    }

    async fn boot_server() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::path("/public/v1/query/get_boot_proof"))
            .and(wiremock::matchers::body_json(
                json!({"organizationId":ORGANIZATION_ID, "ephemeralKey":hex::encode(pin())}),
            ))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"bootProof":fixture::boot()})),
            )
            .mount(&server)
            .await;
        server
    }
    async fn accept_operator(
        expected: OperatorExpectation,
        receipt: ActivityReceipt,
    ) -> Result<VerifiedOperatorKeys, Rejected> {
        let server = boot_server().await;
        verifier(capture_client(server.uri()))
            .accept_operator(expected, receipt)
            .await
            .map(Accepted::into_value)
    }
    fn expected() -> OperatorExpectation {
        OperatorExpectation::parse(
            ORGANIZATION_ID.into(),
            Some("wallet-1".into()),
            "m/0".into(),
        )
        .unwrap()
    }

    #[test]
    fn activity_shape_requires_completion_and_the_right_activity_and_result_types() {
        let operator = ActivityResult::CreateTvcOperatorResult(CreateTvcOperatorResult {
            wallet_id: "wallet-1".into(),
            operator_id: "operator-1".into(),
            encrypt_public_key: point(1),
            sign_public_key: point(2),
        });
        let quorum = ActivityResult::CreateTvcQuorumKeyResult(CreateTvcQuorumKeyResult {
            quorum_key_id: "11111111-1111-4111-8111-111111111111".into(),
            quorum_public_key: format!("{}{}", point(3), point(4)),
            share_ids: vec![],
        });
        for (expected_kind, matching, other) in [
            (ActivityType::CreateTvcOperator, &operator, &quorum),
            (ActivityType::CreateTvcQuorumKey, &quorum, &operator),
        ] {
            for (status, kind, result, accepted) in [
                (
                    ActivityStatus::Completed,
                    expected_kind,
                    Some(matching),
                    true,
                ),
                (
                    ActivityStatus::Pending,
                    expected_kind,
                    Some(matching),
                    false,
                ),
                (
                    ActivityStatus::Completed,
                    ActivityType::CreateWallet,
                    Some(matching),
                    false,
                ),
                (ActivityStatus::Completed, expected_kind, Some(other), false),
                (ActivityStatus::Completed, expected_kind, None, false),
            ] {
                let response = ActivityResponse {
                    status,
                    kind,
                    returned_result: result.cloned(),
                };
                let mut trace = CheckTrace::default();
                let verdict = if expected_kind == ActivityType::CreateTvcOperator {
                    trace
                        .record(response.completed_operator_result())
                        .map(|_| ())
                } else {
                    trace.record(response.completed_quorum_result()).map(|_| ())
                };
                if accepted {
                    verdict.unwrap();
                } else {
                    assert!(
                        matches!(
                            verdict,
                            Err(FailedCheck {
                                source: TvcProvenanceError::ClaimMismatch(_),
                                ..
                            })
                        ),
                        "{expected_kind:?} accepted {status:?} {kind:?} with {result:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn share_routing_requires_one_uuid_per_recipient() {
        let valid = "22222222-2222-4222-8222-222222222222".to_owned();
        let second = "33333333-3333-4333-8333-333333333333".to_owned();
        let parsed = CheckTrace::default()
            .record(parse_share_routing_labels(
                &[valid.clone(), second.clone()],
                2,
            ))
            .unwrap();
        assert_eq!(
            parsed,
            [
                Uuid::parse_str(&valid).unwrap(),
                Uuid::parse_str(&second).unwrap()
            ]
        );
        for labels in [vec![valid.clone()], vec![valid.clone(), String::new()]] {
            let verdict = CheckTrace::default().record(parse_share_routing_labels(&labels, 2));
            assert!(matches!(
                verdict,
                Err(FailedCheck {
                    source: TvcProvenanceError::ClaimMismatch(_),
                    ..
                })
            ));
        }
        assert!(matches!(
            CheckTrace::default()
                .record(parse_share_routing_labels(&[valid, "not-a-uuid".into()], 2)),
            Err(FailedCheck {
                source: TvcProvenanceError::MalformedEvidence(_),
                ..
            })
        ));
    }

    fn operator_claim(role: usize) -> OperatorClaim {
        let payload = operator_payload(role, vec!["ab".repeat(32)]);
        let Claim::Operator(claim) =
            Claim::try_from(authenticate_payload(payload.to_string())).unwrap()
        else {
            panic!("operator fixture");
        };
        claim
    }

    fn quorum_claim() -> QuorumClaim {
        let payload = quorum_payload(vec!["ab".repeat(32)]);
        let Claim::Quorum(claim) =
            Claim::try_from(authenticate_payload(payload.to_string())).unwrap()
        else {
            panic!("quorum fixture");
        };
        claim
    }

    #[test]
    fn operator_role_claims_require_one_claim_per_path_in_either_order() {
        let expected = expected();
        let encryption = || Claim::Operator(operator_claim(0));
        let signing = || Claim::Operator(operator_claim(1));
        let check = |claims| {
            CheckTrace::default().record(OperatorRoleClaims::from_claims(claims, &expected))
        };
        let roles = check(vec![signing(), encryption()]).unwrap();
        assert_eq!(roles.encrypt.body.address.to_string(), point(1));
        assert_eq!(roles.sign.body.address.to_string(), point(2));

        assert!(matches!(
            check(vec![encryption(), encryption()]),
            Err(FailedCheck {
                source: TvcProvenanceError::MalformedEvidence(_),
                ..
            })
        ));
        assert!(matches!(
            check(vec![signing()]),
            Err(FailedCheck {
                source: TvcProvenanceError::ProofUnavailable,
                ..
            })
        ));

        let mutate_signing: [fn(&mut claims::OperatorBody); 3] = [
            |body| body.organization_id = "other-org".into(),
            |body| body.wallet_id = "other-wallet".into(),
            |body| body.derivation_path = "m/0/2".into(),
        ];
        for mutate in mutate_signing {
            let mut claim = operator_claim(1);
            mutate(&mut claim.body);
            assert!(matches!(
                check(vec![encryption(), Claim::Operator(claim)]),
                Err(FailedCheck {
                    source: TvcProvenanceError::ClaimMismatch(_),
                    ..
                })
            ));
        }
        assert!(matches!(
            check(vec![encryption(), Claim::Quorum(quorum_claim())]),
            Err(FailedCheck {
                source: TvcProvenanceError::ClaimMismatch(_),
                ..
            })
        ));
        let other_wallet = OperatorExpectation::parse(
            ORGANIZATION_ID.into(),
            Some("other-wallet".into()),
            "m/0".into(),
        )
        .unwrap();
        assert!(matches!(
            CheckTrace::default().record(OperatorRoleClaims::from_claims(
                vec![encryption(), signing()],
                &other_wallet,
            )),
            Err(FailedCheck {
                source: TvcProvenanceError::ClaimMismatch("operator wallet"),
                ..
            })
        ));
    }

    #[test]
    fn operator_encryption_and_signing_keys_must_differ() {
        let roles = OperatorRoleClaims {
            encrypt: operator_claim(0),
            sign: operator_claim(1),
        };
        CheckTrace::default()
            .record(roles.check_distinct_keys())
            .unwrap();
        let reused_key = OperatorRoleClaims {
            sign: OperatorClaim {
                body: claims::OperatorBody {
                    address: roles.encrypt.body.address,
                    ..roles.sign.body
                },
                ..roles.sign
            },
            encrypt: roles.encrypt,
        };
        assert!(matches!(
            CheckTrace::default().record(reused_key.check_distinct_keys()),
            Err(FailedCheck {
                source: TvcProvenanceError::ClaimMismatch(_),
                ..
            })
        ));
    }

    #[test]
    fn operator_result_must_match_each_signed_field() {
        let roles = OperatorRoleClaims {
            encrypt: operator_claim(0),
            sign: operator_claim(1),
        };
        let returned = CreateTvcOperatorResult {
            wallet_id: "wallet-1".into(),
            operator_id: "routing-label".into(),
            encrypt_public_key: point(1),
            sign_public_key: point(2),
        };
        let check = |returned: &CreateTvcOperatorResult| {
            CheckTrace::default().record(roles.check_returned_result(returned))
        };
        check(&returned).unwrap();
        for changed in [
            CreateTvcOperatorResult {
                wallet_id: "other-wallet".into(),
                ..returned.clone()
            },
            CreateTvcOperatorResult {
                encrypt_public_key: point(7),
                ..returned.clone()
            },
            CreateTvcOperatorResult {
                sign_public_key: point(7),
                ..returned
            },
        ] {
            assert!(matches!(
                check(&changed),
                Err(FailedCheck {
                    source: TvcProvenanceError::ClaimMismatch(_),
                    ..
                })
            ));
        }
    }

    #[test]
    fn operator_result_accepts_equivalent_key_hex_for_each_role() {
        let mut payload = operator_payload(0, vec!["ab".repeat(32)]);
        payload["addressDerivationProof"]["address"] = json!(mixed_case_hex(&point(1)));
        let Claim::Operator(encrypt) =
            Claim::try_from(authenticate_payload(payload.to_string())).unwrap()
        else {
            panic!("operator fixture");
        };
        let roles = OperatorRoleClaims {
            encrypt,
            sign: operator_claim(1),
        };
        let returned = CreateTvcOperatorResult {
            wallet_id: "wallet-1".into(),
            operator_id: "routing-label".into(),
            encrypt_public_key: point(1).to_ascii_uppercase(),
            sign_public_key: mixed_case_hex(&point(2)),
        };
        assert!(
            CheckTrace::default()
                .record(roles.check_returned_result(&returned))
                .is_ok()
        );
    }

    #[test]
    fn operator_result_rejects_swapped_key_roles() {
        let roles = OperatorRoleClaims {
            encrypt: operator_claim(0),
            sign: operator_claim(1),
        };
        let returned = CreateTvcOperatorResult {
            wallet_id: "wallet-1".into(),
            operator_id: "routing-label".into(),
            encrypt_public_key: mixed_case_hex(&point(2)),
            sign_public_key: mixed_case_hex(&point(1)),
        };
        assert!(matches!(
            CheckTrace::default().record(roles.check_returned_result(&returned)),
            Err(FailedCheck {
                source: TvcProvenanceError::ClaimMismatch("operator result wallet/keys"),
                ..
            })
        ));
    }

    #[test]
    fn quorum_check_requires_exactly_one_claim_matching_the_request() {
        let expected = QuorumExpectation::new(
            ORGANIZATION_ID.into(),
            2,
            vec![
                point(1).parse().unwrap(),
                point(2).parse().unwrap(),
                point(3).parse().unwrap(),
            ],
        )
        .unwrap();
        let claim_with_three_recipients = || {
            let mut claim = quorum_claim();
            claim
                .body
                .operator_encrypt_keys
                .push(point(3).parse::<P256PublicKey>().unwrap());
            claim
        };
        let quorum = || Claim::Quorum(claim_with_three_recipients());
        let check = |claims| CheckTrace::default().record(check_quorum_claim(claims, &expected));
        assert!(check(vec![quorum()]).is_ok());
        assert!(matches!(
            check(vec![]),
            Err(FailedCheck {
                source: TvcProvenanceError::ProofUnavailable,
                ..
            })
        ));
        let operator = || Claim::Operator(operator_claim(0));
        assert!(matches!(
            check(vec![operator()]),
            Err(FailedCheck {
                source: TvcProvenanceError::ClaimMismatch(_),
                ..
            })
        ));
        for claims in [
            vec![quorum(), quorum()],
            vec![operator(), quorum()],
            vec![quorum(), operator()],
        ] {
            assert!(matches!(
                check(claims),
                Err(FailedCheck {
                    source: TvcProvenanceError::MalformedEvidence(_),
                    ..
                })
            ));
        }
        let mutate_claim: [fn(&mut claims::QuorumBody); 4] = [
            |body| body.organization_id = "other-org".into(),
            |body| body.threshold = 3,
            |body| body.operator_encrypt_keys.swap(0, 1),
            |body| body.operator_encrypt_keys[2] = point(4).parse::<P256PublicKey>().unwrap(),
        ];
        for mutate in mutate_claim {
            let mut claim = claim_with_three_recipients();
            mutate(&mut claim.body);
            assert!(matches!(
                check(vec![Claim::Quorum(claim)]),
                Err(FailedCheck {
                    source: TvcProvenanceError::ClaimMismatch(_),
                    ..
                })
            ));
        }
    }

    #[test]
    fn quorum_result_must_match_the_signed_uuid_and_full_public_key() {
        let mut claim = quorum_claim();
        let returned = CreateTvcQuorumKeyResult {
            quorum_key_id: "11111111-1111-4111-8111-111111111111".into(),
            quorum_public_key: format!("{}{}", point(3), point(4)).to_ascii_uppercase(),
            share_ids: vec![],
        };
        let check = |returned: &CreateTvcQuorumKeyResult, claim: &QuorumClaim| {
            CheckTrace::default().record(check_quorum_result_binding(returned, claim))
        };
        assert_eq!(
            check(&returned, &claim).unwrap(),
            Uuid::parse_str(&returned.quorum_key_id).unwrap()
        );
        for changed in [
            CreateTvcQuorumKeyResult {
                quorum_key_id: "22222222-2222-4222-8222-222222222222".into(),
                ..returned.clone()
            },
            CreateTvcQuorumKeyResult {
                quorum_public_key: format!("{}{}", point(7), point(4)),
                ..returned.clone()
            },
            CreateTvcQuorumKeyResult {
                quorum_public_key: format!("{}{}", point(3), point(7)),
                ..returned.clone()
            },
        ] {
            assert!(matches!(
                check(&changed, &claim),
                Err(FailedCheck {
                    source: TvcProvenanceError::ClaimMismatch(_),
                    ..
                })
            ));
        }
        claim.body.quorum_key_id = "not-a-uuid".into();
        let changed = CreateTvcQuorumKeyResult {
            quorum_key_id: "not-a-uuid".into(),
            ..returned
        };
        assert!(matches!(
            check(&changed, &claim),
            Err(FailedCheck {
                source: TvcProvenanceError::MalformedEvidence(_),
                ..
            })
        ));
    }

    #[test]
    fn proof_count_accepts_64_and_rejects_65() {
        CheckTrace::default()
            .record(check_envelope_count(64))
            .unwrap();
        assert!(matches!(
            CheckTrace::default().record(check_envelope_count(65)),
            Err(FailedCheck {
                source: TvcProvenanceError::MalformedEvidence(_),
                ..
            })
        ));
    }

    #[test]
    fn policy_routing_reads_the_type_and_caps_the_payload_at_256_kib() {
        let check = |payload: &str| CheckTrace::default().record(claims::policy_envelope(payload));
        let policy = r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#;
        let limit = 256 * 1024;
        let max_payload = format!("{policy}{}", " ".repeat(limit - policy.len()));
        assert!(check(&max_payload).unwrap());
        assert!(!check(r#"{"type":"APP_PROOF_TYPE_ADDRESS_DERIVATION"}"#).unwrap());
        for malformed in ["{".to_owned(), format!("{max_payload} ")] {
            assert!(matches!(
                check(&malformed),
                Err(FailedCheck {
                    source: TvcProvenanceError::MalformedEvidence(_),
                    ..
                })
            ));
        }
    }

    #[test]
    fn common_submission_must_be_local_and_appear_in_every_claim() {
        let first = RequestFingerprint::try_from("ab".repeat(32)).unwrap();
        let second = RequestFingerprint::try_from("cd".repeat(32)).unwrap();
        let unknown = RequestFingerprint::try_from("ef".repeat(32)).unwrap();
        let checkpoint: SubmissionCheckpoint = serde_json::from_value(json!({
            "version": 1,
            "organization_id": ORGANIZATION_ID,
            "kind": "CreateTvcOperator",
            "fingerprints": [first, second],
        }))
        .unwrap();
        let encrypt = Context {
            digests: vec![first, second, unknown],
        };
        let sign = Context {
            digests: vec![second, unknown],
        };
        let check = |contexts: &[&Context]| {
            CheckTrace::default().record(match_common_submission(&checkpoint, contexts))
        };
        assert_eq!(check(&[&encrypt, &sign]).unwrap(), second);
        let only_first = Context {
            digests: vec![first, unknown],
        };
        assert!(matches!(
            check(&[&only_first, &sign]),
            Err(FailedCheck {
                source: TvcProvenanceError::ReceiptMismatch,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn freshly_signed_capture_accepts_common_intersection_and_rejects_split() {
        let accepted = accept_operator(expected(), receipt(Mode::Common).await)
            .await
            .unwrap();
        assert_eq!(accepted.encryption_key().to_string(), point(1));
        assert_eq!(accepted.signing_key().to_string(), point(2));
        assert!(matches!(
            accept_operator(expected(), receipt(Mode::Split).await).await,
            Err(ref rejection) if matches!(cause(rejection), TvcProvenanceError::ReceiptMismatch)
        ));
    }
    fn capture_client(url: String) -> TurnkeyClient<TurnkeyP256ApiKey> {
        TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::from_bytes([6; 32], None).unwrap())
            .base_url(url)
            .build()
            .unwrap()
    }
    async fn same_body_receipt(url: String) -> ActivityReceipt {
        capture_client(url)
            .with_app_proofs()
            .create_tvc_operator_with_receipt(
                ORGANIZATION_ID.parse().unwrap(),
                100,
                CreateTvcOperatorIntent {
                    wallet_name: Some("wallet".into()),
                    wallet_id: None,
                    path: "m/0".into(),
                    operator_name: "same-body-with-clock-rollback".into(),
                },
                async |_| Ok(()),
            )
            .await
            .map(completed)
            .unwrap()
            .into_parts()
            .1
    }
    #[tokio::test]
    async fn earlier_signed_claim_rejects_a_new_submission_of_the_same_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(Producer {
                mode: Mode::EarlierClaim,
                first: Mutex::new(None),
            })
            .mount(&server)
            .await;
        let first = same_body_receipt(server.uri()).await;
        assert!(accept_operator(expected(), first).await.is_ok());
        assert!(matches!(
            accept_operator(expected(), same_body_receipt(server.uri()).await).await,
            Err(ref rejection) if matches!(cause(rejection), TvcProvenanceError::ReceiptMismatch)
        ));
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let digests = requests.iter().map(digest).collect::<HashSet<_>>();
        assert_eq!(digests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|request| request.body == requests[0].body)
        );
    }
    #[test]
    fn strict_quorum_required_fields_numeric_forms_aliases_and_duplicates() {
        let baseline = quorum_payload(vec!["ab".repeat(32)]);
        let mut cases = Vec::new();
        for field in [
            "organizationId",
            "quorumKeyId",
            "quorumPublicKey",
            "threshold",
            "operatorEncryptKeys",
        ] {
            let mut value = baseline.clone();
            value["tvcQuorumKeyCreateProof"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            cases.push(value);
            let mut value = baseline.clone();
            value["tvcQuorumKeyCreateProof"][field] = Value::Null;
            cases.push(value);
        }
        for wrong in [json!("2"), json!(2.0), json!(1), json!(256), json!(-2)] {
            let mut value = baseline.clone();
            value["tvcQuorumKeyCreateProof"]["threshold"] = wrong;
            cases.push(value);
        }
        for wrong in [
            json!([]),
            json!([null]),
            json!([point(1), point(1)]),
            json!("keys"),
        ] {
            let mut value = baseline.clone();
            value["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] = wrong;
            cases.push(value);
        }
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"]["quorum_key_id"] = json!("alias");
        cases.push(value);
        let mut value = baseline.clone();
        value["addressDerivationProof"] =
            operator_payload(0, vec!["ab".repeat(32)])["addressDerivationProof"].clone();
        cases.push(value);
        let mut value = baseline.clone();
        value["tvcQuorumKeyCreateProof"]["quorumKeyId"] = json!("");
        cases.push(value);
        for value in cases {
            assert!(
                Claim::try_from(authenticate_payload(value.to_string())).is_err(),
                "accepted {value}"
            );
        }
        let raw = baseline.to_string();
        for invalid in [
            raw.replacen("\"threshold\":2", "\"threshold\":2e0", 1),
            raw.replacen("\"threshold\":2", "\"threshold\":2,\"threshold\":2", 1),
            raw.replacen(
                "\"type\":",
                "\"type\":\"APP_PROOF_TYPE_POLICY_OUTCOME\",\"type\":",
                1,
            ),
        ] {
            assert!(Claim::try_from(authenticate_payload(invalid)).is_err());
        }
    }

    #[test]
    fn signed_quorum_recipients_keep_their_order_across_hex_spellings() {
        let mut payload = quorum_payload(vec!["ab".repeat(32)]);
        payload["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] =
            json!([mixed_case_hex(&point(1)), point(2).to_ascii_uppercase()]);
        let expected = QuorumExpectation::new(
            ORGANIZATION_ID.into(),
            2,
            vec![point(1).parse().unwrap(), point(2).parse().unwrap()],
        )
        .unwrap();
        let claim = Claim::try_from(authenticate_payload(payload.to_string())).unwrap();
        assert!(
            CheckTrace::default()
                .record(check_quorum_claim(vec![claim], &expected))
                .is_ok()
        );
    }

    #[test]
    fn signed_quorum_recipients_reject_duplicates_across_hex_spellings() {
        let mut payload = quorum_payload(vec!["ab".repeat(32)]);
        payload["tvcQuorumKeyCreateProof"]["operatorEncryptKeys"] =
            json!([point(1), mixed_case_hex(&point(1))]);
        assert!(matches!(
            Claim::try_from(authenticate_payload(payload.to_string())),
            Err(TvcProvenanceError::MalformedEvidence(message)) if message == "duplicate quorum recipient"
        ));
    }

    #[test]
    fn real_signer_endorsement_vector_fixes_purpose_bytes_and_raw_signature_encoding() {
        let v: Value =
            serde_json::from_str(include_str!("../tests/fixtures/boot_endorsement_v1.json"))
                .unwrap();
        let q = v["quorumPublicKeyHex"].as_str().unwrap().parse().unwrap();
        let verifier = fixture::with_test_roots(q, Certificate::from_der(fixture::ROOT).unwrap());
        let document = hex::decode(v["documentHex"].as_str().unwrap()).unwrap();
        assert_eq!(hex::encode(endorsement::ENDORSEMENT_PREFIX), v["prefixHex"]);
        let mut message = endorsement::ENDORSEMENT_PREFIX.to_vec();
        message.extend_from_slice(&document);
        assert_eq!(hex::encode(message), v["messageHex"]);
        verifier
            .verify_document_signature(&document, v["signatureHex"].as_str().unwrap())
            .unwrap();
        for (_, signature) in v["negativeSignatures"].as_object().unwrap() {
            assert!(
                verifier
                    .verify_document_signature(&document, signature.as_str().unwrap())
                    .is_err()
            );
        }
        let mut changed = document.clone();
        changed[10] ^= 1;
        assert!(
            verifier
                .verify_document_signature(&changed, v["signatureHex"].as_str().unwrap())
                .is_err()
        );
    }

    #[tokio::test]
    async fn boot_lookup_checks_the_requested_key_and_requires_evidence() {
        let proof = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
        let key = proof.public_key.parse().unwrap();
        for (response, expected_status) in [
            (ResponseTemplate::new(200).set_body_json(json!({})), None),
            (
                ResponseTemplate::new(503).set_body_string("private-backend-sentinel"),
                Some(503),
            ),
        ] {
            let server = MockServer::start().await;
            let request = wiremock::matchers::body_json(json!({
                "organizationId": ORGANIZATION_ID,
                "ephemeralKey": hex::encode(pin()),
            }));
            Mock::given(method("POST"))
                .and(wiremock::matchers::path("/public/v1/query/get_boot_proof"))
                .and(request)
                .respond_with(response)
                .expect(1)
                .mount(&server)
                .await;
            let verifier = verifier(capture_client(server.uri()));
            let lookup = verifier.lookup_boot_proof(ORGANIZATION_ID, &key).await;
            let failure = CheckTrace::default().record(lookup).unwrap_err();
            let observed = server.received_requests().await.unwrap();
            assert_eq!(observed.len(), 1);
            assert!(!format!("{failure:?}").contains("private-backend-sentinel"));
            match (expected_status, failure.source) {
                (None, TvcProvenanceError::ProofUnavailable) => {}
                (
                    Some(expected),
                    TvcProvenanceError::BootProofLookup(BootProofLookupError::HttpStatus(actual)),
                ) => {
                    assert_eq!(actual, expected);
                }
                (expected, actual) => panic!("unexpected lookup error for {expected:?}: {actual}"),
            }
        }
        let server = boot_server().await;
        let verifier = verifier(capture_client(server.uri()));
        let lookup = verifier.lookup_boot_proof(ORGANIZATION_ID, &key).await;
        assert_eq!(
            CheckTrace::default()
                .record(lookup)
                .unwrap()
                .ephemeral_public_key_hex,
            proof.public_key
        );
    }

    #[tokio::test]
    async fn boot_lookup_uses_the_same_key_for_lowercase_and_mixed_case_app_proofs() {
        let server = boot_server().await;
        let client = capture_client(server.uri());
        let lowercase = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
        let mixed = AppProof {
            public_key: mixed_case_hex(&lowercase.public_key),
            ..lowercase.clone()
        };
        assert_ne!(lowercase.public_key, mixed.public_key);
        let lower_response =
            crate::get_boot_proof_for_app_proof(&client, ORGANIZATION_ID.to_owned(), &lowercase)
                .await
                .unwrap();
        let mixed_response =
            crate::get_boot_proof_for_app_proof(&client, ORGANIZATION_ID.to_owned(), &mixed)
                .await
                .unwrap();
        assert_eq!(lower_response.boot_proof, Some(fixture::boot()));
        assert_eq!(mixed_response.boot_proof, lower_response.boot_proof);

        let accepted = verifier(client)
            .claims_for_test(ORGANIZATION_ID, vec![mixed])
            .await
            .unwrap();
        assert_eq!(
            accepted.facts().envelopes()[0].ephemeral_key(),
            Some(&lowercase.public_key.parse().unwrap())
        );
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    fn cause(rejection: &Rejected) -> &TvcProvenanceError {
        std::error::Error::source(rejection)
            .unwrap()
            .downcast_ref::<TvcProvenanceError>()
            .unwrap()
    }

    impl<S: Stamp> TvcProvenance<S> {
        async fn claims_for_test(
            &self,
            organization: &str,
            proofs: Vec<AppProof>,
        ) -> Result<Accepted<()>, Rejected> {
            let count = proofs.len();
            let mut trace = AcceptanceTrace::default();
            let result = self
                .claims(organization, proofs, &mut trace)
                .await
                .map(drop);
            AcceptanceFacts::assemble(
                self.endorsement.environment,
                self.endorsement.quorum_key.clone(),
                AcceptanceEvidence::Operator {
                    organization: organization.into(),
                    wallet: None,
                    base_path: "m/0".into(),
                    returned: None,
                },
                Vec::new(),
                count,
                trace,
            )
            .finish(result)
        }
    }

    fn states(checks: &[Check]) -> Vec<(ProofCheck, CheckState)> {
        checks
            .iter()
            .map(|check| (check.kind, check.state))
            .collect()
    }

    #[tokio::test]
    async fn reports_actual_operator_and_quorum_acceptance_with_full_public_operands() {
        let server = boot_server().await;
        for existing_wallet in [None, Some("wallet-1".to_owned())] {
            let expected =
                OperatorExpectation::parse(ORGANIZATION_ID.into(), existing_wallet, "m/0".into())
                    .unwrap();
            let (accepted, facts) = verifier(capture_client(server.uri()))
                .accept_operator(expected, receipt(Mode::Operator).await)
                .await
                .unwrap()
                .into_parts();
            assert_eq!(accepted.encryption_key().to_string(), point(1));
            assert_eq!(accepted.signing_key().to_string(), point(2));
            assert_eq!(
                states(facts.checks()),
                [
                    (ActivityShape, Pass),
                    (EnvelopeCount, Pass),
                    (ClaimSetAndIntent, Pass),
                    (CommonSubmission, Pass),
                    (OutputKeyRoles, Pass),
                    (ResultBinding, Pass),
                ]
            );
            assert_eq!(
                facts.signer_quorum_key(),
                &QuorumPublicKey::try_from(fixture::quorum_key().as_slice()).unwrap()
            );
            assert_eq!(
                facts.matched_fingerprint(),
                Some(&facts.sent_fingerprints()[0])
            );
            for (index, envelope) in facts.envelopes()[..2].iter().enumerate() {
                assert_eq!(envelope.boot_source_envelope(), (index == 1).then_some(0));
                let expected: &[ProofCheck] = if index == 0 {
                    &[
                        EnvelopeRouting,
                        EphemeralKeyFormat,
                        BootProofLookup,
                        DocumentEndorsement,
                        AwsAttestationAtIssuance,
                        BootBinding,
                        ApplicationSignature,
                        ClaimDecode,
                    ]
                } else {
                    &[
                        EnvelopeRouting,
                        EphemeralKeyFormat,
                        ApplicationSignature,
                        ClaimDecode,
                    ]
                };
                assert_eq!(
                    states(envelope.checks()),
                    expected
                        .iter()
                        .map(|&kind| (kind, Pass))
                        .collect::<Vec<_>>()
                );
                assert_eq!(
                    envelope.ephemeral_key(),
                    Some(&hex::encode(pin()).parse::<EphemeralPublicKey>().unwrap())
                );
                let attestation = envelope.attestation().unwrap();
                assert_eq!(attestation.issued_at_ms, fixture::ISSUED_AT);
                assert_eq!(attestation.public_key, Some(pin().to_vec()));
                assert!(
                    attestation
                        .pcrs
                        .iter()
                        .any(|(index, pcr)| *index == 17 && pcr.len() == 48)
                );
                assert_eq!(
                    envelope.boot().unwrap().manifest_hash,
                    envelope.boot().unwrap().envelope_manifest_hash
                );
                let Some(ClaimEvidence::Operator {
                    public_key,
                    digests,
                    ..
                }) = envelope.claim()
                else {
                    panic!("operator claim evidence");
                };
                assert!(
                    [
                        point(1).parse::<P256PublicKey>().unwrap(),
                        point(2).parse::<P256PublicKey>().unwrap()
                    ]
                    .contains(public_key)
                );
                assert_eq!(digests, facts.sent_fingerprints());
            }
            let policy = &facts.envelopes()[2];
            assert!(policy.policy_excluded());
            assert_eq!(states(policy.checks()), [(EnvelopeRouting, Pass)]);
            assert!(policy.claim().is_none());
        }
        let expected = QuorumExpectation::new(
            ORGANIZATION_ID.into(),
            2,
            vec![point(1).parse().unwrap(), point(2).parse().unwrap()],
        )
        .unwrap();
        let (accepted, facts) = verifier(capture_client(server.uri()))
            .accept_quorum(expected, receipt(Mode::Quorum).await)
            .await
            .unwrap()
            .into_parts();
        assert_eq!(
            states(facts.checks()),
            [
                (ActivityShape, Pass),
                (EnvelopeCount, Pass),
                (ClaimSetAndIntent, Pass),
                (CommonSubmission, Pass),
                (ResultBinding, Pass),
                (ShareRoutingShape, Pass),
            ]
        );
        assert_eq!(
            accepted.into_parts(),
            (
                Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
                format!("{}{}", point(3), point(4))
                    .parse::<QuorumPublicKey>()
                    .unwrap(),
                vec![
                    Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
                    Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap(),
                ],
            )
        );
        let Some(ClaimEvidence::Quorum {
            public_key,
            recipients,
            ..
        }) = facts.envelopes()[0].claim()
        else {
            panic!("quorum claim evidence");
        };
        assert_eq!(
            public_key,
            &format!("{}{}", point(3), point(4))
                .parse::<QuorumPublicKey>()
                .unwrap()
        );
        assert_eq!(
            recipients,
            &vec![
                point(1).parse::<P256PublicKey>().unwrap(),
                point(2).parse::<P256PublicKey>().unwrap()
            ]
        );
        let debug = format!("{facts:?}");
        assert!(!debug.contains("unrelated-ump-key"));
        assert!(!debug.contains("signature:"));
        assert!(!debug.contains("synthetic-predicate-test"));
    }

    #[tokio::test]
    async fn boot_binding_failure_retains_partial_evidence_and_stops_acceptance() {
        let boot = BootProof {
            ephemeral_public_key_hex: format!("{}{}", point(7), point(9)),
            ..fixture::boot()
        };
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::path("/public/v1/query/get_boot_proof"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"bootProof":boot})))
            .expect(1)
            .mount(&server)
            .await;
        let rejection = verifier(capture_client(server.uri()))
            .accept_operator(expected(), receipt(Mode::Operator).await)
            .await
            .unwrap_err();
        assert_eq!(
            rejection.stage(),
            AcceptanceStage {
                check: BootBinding,
                envelope_index: Some(0)
            }
        );
        assert!(matches!(
            cause(&rejection),
            TvcProvenanceError::EndorsementFailure
        ));
        let facts = rejection.facts();
        assert_eq!(facts.envelope_count(), 3);
        assert_eq!(
            states(facts.checks()),
            [(ActivityShape, Pass), (EnvelopeCount, Pass)]
        );
        let [envelope] = facts.envelopes() else {
            panic!("only the first envelope should have been visited");
        };
        assert_eq!(
            states(envelope.checks()),
            [
                (EnvelopeRouting, Pass),
                (EphemeralKeyFormat, Pass),
                (BootProofLookup, Pass),
                (DocumentEndorsement, Pass),
                (AwsAttestationAtIssuance, Pass),
                (BootBinding, Fail),
            ]
        );
        assert!(envelope.attestation().is_some());
        assert!(envelope.boot().is_some());
        assert!(envelope.claim().is_none());
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn public_acceptance_rejects_substituted_operator_and_quorum_keys() {
        let operator = accept_operator(expected(), receipt(Mode::SubstituteKey).await)
            .await
            .unwrap_err();
        let server = boot_server().await;
        let expected = QuorumExpectation::new(
            ORGANIZATION_ID.into(),
            2,
            vec![point(1).parse().unwrap(), point(2).parse().unwrap()],
        )
        .unwrap();
        let quorum = verifier(capture_client(server.uri()))
            .accept_quorum(expected, receipt(Mode::QuorumSubstitutedKey).await)
            .await
            .unwrap_err();
        for rejection in [&operator, &quorum] {
            assert_eq!(
                rejection.stage(),
                AcceptanceStage {
                    check: ResultBinding,
                    envelope_index: None
                }
            );
            assert!(matches!(
                cause(rejection),
                TvcProvenanceError::ClaimMismatch(_)
            ));
            assert_eq!(
                states(rejection.facts().checks()).last(),
                Some(&(ResultBinding, Fail))
            );
        }
        let facts = quorum.facts();
        let AcceptanceEvidence::Quorum {
            returned: Some(returned),
            ..
        } = facts.acceptance()
        else {
            panic!("returned quorum evidence");
        };
        assert_eq!(returned.public_key, format!("{}{}", point(5), point(6)));
        let Some(ClaimEvidence::Quorum { public_key, .. }) = facts.envelopes()[0].claim() else {
            panic!("signed quorum evidence");
        };
        assert_eq!(
            public_key,
            &format!("{}{}", point(3), point(4))
                .parse::<QuorumPublicKey>()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn too_many_envelopes_stops_verification_before_any_are_visited() {
        let valid = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
        let rejection = verifier(capture_client("http://127.0.0.1:1".into()))
            .claims_for_test(ORGANIZATION_ID, vec![valid; 65])
            .await
            .unwrap_err();
        let facts = rejection.facts();
        assert_eq!(facts.envelope_count(), 65);
        assert!(facts.envelopes().is_empty());
        assert_eq!(states(facts.checks()), [(EnvelopeCount, Fail)]);
    }

    #[tokio::test]
    async fn distinct_ephemeral_keys_stay_scoped_to_each_authenticated_envelope() {
        use qos_core::protocol::QosHash;
        let second_key = format!("{}{}", point(12), point(13));
        let manifest = fixture::manifest();
        let mut doc = fixture::attestation(&manifest);
        let key_bytes = hex::decode(&second_key).unwrap();
        doc.public_key = Some(serde_bytes::ByteBuf::from(key_bytes.clone()));
        let pcr17 = qos_nsm::nitro::expected_manifest_commitment_pcr::<qos_nsm::nitro::Live>(
            &manifest.qos_hash(),
            &key_bytes,
        )
        .unwrap();
        doc.pcrs
            .insert(17, serde_bytes::ByteBuf::from(pcr17.to_vec()));
        let mut second_boot = fixture::boot_from(manifest, &doc);
        second_boot.ephemeral_public_key_hex = second_key.clone();
        let server = boot_server().await;
        Mock::given(wiremock::matchers::body_json(
            json!({"organizationId":ORGANIZATION_ID, "ephemeralKey":second_key}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"bootProof":second_boot})))
        .mount(&server)
        .await;
        let first = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
        let mut second = proof(operator_payload(1, vec!["ab".repeat(32)]).to_string());
        second.public_key = second_key.clone();
        let signature: Signature = SigningKey::from_slice(&[13; 32])
            .unwrap()
            .sign(second.proof_payload.as_bytes());
        second.signature = hex::encode(signature.to_bytes());
        let (_, facts) = verifier(capture_client(server.uri()))
            .claims_for_test(ORGANIZATION_ID, vec![first, second])
            .await
            .unwrap()
            .into_parts();
        assert_eq!(
            facts.envelopes()[0].ephemeral_key(),
            Some(&hex::encode(pin()).parse::<EphemeralPublicKey>().unwrap())
        );
        assert_eq!(
            facts.envelopes()[1].ephemeral_key(),
            Some(&second_key.parse::<EphemeralPublicKey>().unwrap())
        );
        assert!(
            facts
                .envelopes()
                .iter()
                .all(|entry| entry.checks().iter().all(|check| check.state == Pass))
        );
    }

    #[tokio::test]
    async fn boot_reuse_is_local_and_never_skips_an_application_signature() {
        let server = boot_server().await;
        let verifier = verifier(capture_client(server.uri()));
        for invalid_signature in [false, true] {
            let first = proof(operator_payload(0, vec!["ab".repeat(32)]).to_string());
            let mut second = proof(operator_payload(1, vec!["ab".repeat(32)]).to_string());
            if invalid_signature {
                second.signature = "00".repeat(64);
            }
            let result = verifier
                .claims_for_test(ORGANIZATION_ID, vec![first, second])
                .await;
            let facts = if invalid_signature {
                let rejection = result.as_ref().unwrap_err();
                assert_eq!(
                    rejection.stage(),
                    AcceptanceStage {
                        check: ApplicationSignature,
                        envelope_index: Some(1)
                    }
                );
                rejection.facts()
            } else {
                result.as_ref().unwrap().facts()
            };
            let reused = &facts.envelopes()[1];
            assert_eq!(reused.boot_source_envelope(), Some(0));
            assert!(reused.boot().is_some());
            assert!(reused.attestation().is_some());
            for kind in [
                BootProofLookup,
                DocumentEndorsement,
                AwsAttestationAtIssuance,
                BootBinding,
            ] {
                assert!(!reused.checks().iter().any(|check| check.kind == kind));
            }
            assert_eq!(
                reused
                    .checks()
                    .iter()
                    .find(|check| check.kind == ApplicationSignature)
                    .unwrap()
                    .state,
                if invalid_signature { Fail } else { Pass }
            );
            assert_eq!(
                server.received_requests().await.unwrap().len(),
                if invalid_signature { 2 } else { 1 }
            );
        }
        // A different organization still performs its own lookup, even using the same verifier.
        assert!(matches!(
            verifier
                .claims_for_test(
                    "different-org",
                    vec![proof(
                        operator_payload(0, vec!["ab".repeat(32)]).to_string()
                    )],
                )
                .await,
            Err(ref rejection) if matches!(cause(rejection), TvcProvenanceError::BootProofLookup(_))
        ));
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
    }

    #[test]
    fn quorum_expectation_rejects_too_many_recipients() {
        let key = point(1).parse().unwrap();
        assert!(matches!(
            QuorumExpectation::new(ORGANIZATION_ID.into(), 2, vec![key; 256]),
            Err(TvcProvenanceError::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255"
            ))
        ));
    }

    #[test]
    fn quorum_expectation_rejects_an_empty_organization() {
        let recipients = vec![point(1).parse().unwrap(), point(2).parse().unwrap()];
        assert!(matches!(
            QuorumExpectation::new(String::new(), 2, recipients),
            Err(TvcProvenanceError::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255"
            ))
        ));
    }

    #[test]
    fn quorum_expectation_rejects_a_threshold_above_the_recipient_count() {
        let recipients = vec![point(1).parse().unwrap()];
        assert!(matches!(
            QuorumExpectation::new(ORGANIZATION_ID.into(), 2, recipients),
            Err(TvcProvenanceError::InvalidExpectation(
                "require 2 <= threshold <= recipients <= 255"
            ))
        ));
    }
}
