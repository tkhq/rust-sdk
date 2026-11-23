use super::{
    TvcProvenanceError,
    facts::{
        AttestationEvidence, BootObservation, CheckOutcome, CheckTrace, FailedCheck, ProofCheck,
    },
};
use crate::{
    AWS_ROOT_CERT_PEM, EXPECTED_EPHEMERAL_PUBLIC_KEY_LENGTH, EphemeralPublicKey, ParsedBootProof,
    cert_from_pem, parse_and_verify_der_attestation, unsafe_attestation_doc_from_der,
    verify_boot_binding, verify_payload_signature,
};
use aws_nitro_enclaves_nsm_api::api::AttestationDoc;
use p256::ecdsa::{Signature, signature::Verifier};
use turnkey_client::generated::external::data::v1::{AppProof, BootProof, SignatureScheme};
use turnkey_enclave_encrypt::QuorumPublicKey;

/// Domain separation for Signer endorsements of raw boot attestation documents.
pub const ENDORSEMENT_PREFIX: &[u8] = b"turnkey.boot-proof.attestation.v1\0";

/// Original app-proof payload with a verified ephemeral-key signature and boot chain.
/// The chain includes AWS attestation and its endorsement by the pinned Signer quorum key.
/// Its claims still need to match the original request and returned result.
pub(super) struct AuthenticatedPayload {
    payload: String,
}

impl AuthenticatedPayload {
    pub(super) fn into_inner(self) -> String {
        self.payload
    }
}

/// App envelope with its canonical full composite key parsed once at the boundary.
pub(super) struct ParsedAppProof {
    proof: AppProof,
    ephemeral_key: EphemeralPublicKey,
}
impl TryFrom<AppProof> for ParsedAppProof {
    type Error = TvcProvenanceError;
    fn try_from(proof: AppProof) -> Result<Self, Self::Error> {
        // Bound the hex input before decoding. EphemeralPublicKey validates both SEC1 points.
        if proof.public_key.len() != EXPECTED_EPHEMERAL_PUBLIC_KEY_LENGTH * 2 {
            return Err(TvcProvenanceError::MalformedEvidence(
                "composite key must contain two uncompressed 65-byte points".into(),
            ));
        }
        let ephemeral_key = proof
            .public_key
            .parse::<EphemeralPublicKey>()
            .map_err(|e| TvcProvenanceError::MalformedEvidence(e.to_string()))?;
        Ok(Self {
            proof,
            ephemeral_key,
        })
    }
}
impl ParsedAppProof {
    /// Requires the app proof's ephemeral key to contain two valid uncompressed P-256 public keys.
    pub(super) fn check_ephemeral_key(proof: AppProof) -> CheckOutcome<Self> {
        CheckOutcome::new(ProofCheck::EphemeralKeyFormat, Self::try_from(proof))
    }

    pub(super) fn ephemeral_key(&self) -> &EphemeralPublicKey {
        &self.ephemeral_key
    }
    pub(super) fn proof(&self) -> &AppProof {
        &self.proof
    }
}

/// Successful pinned-Signer/AWS binding. Construction is restricted to boot authentication.
pub(super) struct VerifiedBoot {
    key: EphemeralPublicKey,
}
impl VerifiedBoot {
    /// Checks the signature on the exact app-proof payload using the signing key
    /// from the verified boot proof. The full ephemeral key must also match.
    pub(super) fn check_application_signature(
        &self,
        parsed: ParsedAppProof,
    ) -> CheckOutcome<AuthenticatedPayload> {
        CheckOutcome::new(ProofCheck::ApplicationSignature, self.authenticate(parsed))
    }

    pub(super) fn key(&self) -> &EphemeralPublicKey {
        &self.key
    }

    pub(super) fn authenticate(
        &self,
        parsed: ParsedAppProof,
    ) -> Result<AuthenticatedPayload, TvcProvenanceError> {
        let proof = parsed.proof;
        if self.key != parsed.ephemeral_key || proof.scheme != SignatureScheme::EphemeralKeyP256 {
            return Err(TvcProvenanceError::EndorsementFailure);
        }
        verify_payload_signature(
            proof.proof_payload.as_bytes(),
            &proof.signature,
            self.key.0.signing_key(),
        )
        .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        Ok(AuthenticatedPayload {
            payload: proof.proof_payload,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum SignerEnvironment {
    Production,
    Preproduction,
    Development,
    #[cfg(test)]
    Synthetic,
}

impl SignerEnvironment {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Preproduction => "preproduction",
            Self::Development => "development",
            #[cfg(test)]
            Self::Synthetic => "synthetic test policy",
        }
    }
}

pub(super) struct SelectedEndorsement {
    pub(super) quorum_key: QuorumPublicKey,
    pub(super) environment: SignerEnvironment,
    pub(super) attestation_root: Vec<u8>,
}
impl TryFrom<&str> for SelectedEndorsement {
    type Error = TvcProvenanceError;

    fn try_from(api_url: &str) -> Result<Self, Self::Error> {
        let (quorum_key, environment) = match api_url {
            "https://api.turnkey.com" => (
                QuorumPublicKey::production_signer(),
                SignerEnvironment::Production,
            ),
            "https://api.preprod.turnkey.engineering" => (
                QuorumPublicKey::preprod_signer(),
                SignerEnvironment::Preproduction,
            ),
            "https://api.dev.turnkey.engineering" => (
                QuorumPublicKey::dev_signer(),
                SignerEnvironment::Development,
            ),
            _ => return Err(TvcProvenanceError::UnsupportedEnvironment),
        };
        quorum_key
            .encrypt_public_key()
            .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        quorum_key
            .verifying_key()
            .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        Ok(Self {
            quorum_key,
            environment,
            attestation_root: cert_from_pem(AWS_ROOT_CERT_PEM)
                .map_err(|_| TvcProvenanceError::EndorsementFailure)?,
        })
    }
}

impl SelectedEndorsement {
    /// Authenticates a boot document and binds its full ephemeral key to the pinned Signer.
    /// Partial observations survive errors; only success produces a reusable verified binding.
    pub(super) fn authenticate_boot(
        &self,
        key: &EphemeralPublicKey,
        boot: BootProof,
        trace: &mut CheckTrace,
        observed: &mut BootObservation,
    ) -> Result<VerifiedBoot, FailedCheck> {
        let document = trace.record(self.verify_document_endorsement(&boot))?;
        let attestation = trace.record(self.verify_aws_attestation_at_issuance(&document))?;
        observed.attestation = Some(AttestationEvidence {
            issued_at_ms: attestation.timestamp,
            public_key: attestation.public_key.as_ref().map(|value| value.to_vec()),
            manifest_commitment: attestation.user_data.as_ref().map(|value| value.to_vec()),
            pcrs: attestation
                .pcrs
                .iter()
                .map(|(index, value)| (*index, value.to_vec()))
                .collect(),
        });
        trace.record(self.check_boot_binding(key, &boot, attestation, &mut observed.boot))?;
        Ok(VerifiedBoot { key: key.clone() })
    }

    /// Checks the manifest approvals and matches the manifest, enclave measurements,
    /// and ephemeral key to what AWS attested. Any Signer quorum key in the
    /// manifest must match the pinned key.
    fn check_boot_binding(
        &self,
        key: &EphemeralPublicKey,
        boot: &BootProof,
        attestation: AttestationDoc,
        observed: &mut Option<super::BootEvidence>,
    ) -> CheckOutcome<()> {
        CheckOutcome::run(ProofCheck::BootBinding, || {
            let parsed =
                ParsedBootProof::parse(boot).map_err(|_| TvcProvenanceError::EndorsementFailure)?;
            *observed = Some(super::BootEvidence::from(&parsed));
            verify_boot_binding(key, parsed, attestation, Some(&self.quorum_key))
                .map_err(|_| TvcProvenanceError::EndorsementFailure)
        })
    }

    /// Checks that the pinned Signer quorum key signed these exact attestation bytes.
    fn verify_document_endorsement(&self, boot: &BootProof) -> CheckOutcome<Vec<u8>> {
        CheckOutcome::run(ProofCheck::DocumentEndorsement, || {
            let signature = boot
                .quorum_key_attestation_signature_hex
                .as_deref()
                .ok_or(TvcProvenanceError::ProofUnavailable)?;
            let document = base64::decode(&boot.aws_attestation_doc_b64)
                .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
            self.verify_document_signature(&document, signature)?;
            Ok(document)
        })
    }

    /// Verifies the attestation signature and AWS certificate chain at the time
    /// in the signed document.
    fn verify_aws_attestation_at_issuance(&self, document: &[u8]) -> CheckOutcome<AttestationDoc> {
        CheckOutcome::run(ProofCheck::AwsAttestationAtIssuance, || {
            let unverified = unsafe_attestation_doc_from_der(document)
                .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
            parse_and_verify_der_attestation(
                document,
                &self.attestation_root,
                unverified.timestamp / 1000,
            )
            .map_err(|_| TvcProvenanceError::EndorsementFailure)
        })
    }

    /// Verifies the pinned Signer signature over [`ENDORSEMENT_PREFIX`] and the exact document bytes.
    /// AWS certificate and attestation validation are separate checks.
    pub(super) fn verify_document_signature(
        &self,
        document: &[u8],
        signature: &str,
    ) -> Result<(), TvcProvenanceError> {
        if signature.len() != 128
            || !signature
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(TvcProvenanceError::EndorsementFailure);
        }
        let signature =
            hex::decode(signature).map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        let signature = Signature::from_slice(&signature)
            .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        let key = self
            .quorum_key
            .verifying_key()
            .map_err(|_| TvcProvenanceError::EndorsementFailure)?;
        let message = [ENDORSEMENT_PREFIX, document].concat();
        key.verify(&message, &signature)
            .map_err(|_| TvcProvenanceError::EndorsementFailure)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tvc::{
        ClaimEvidence,
        fixture::{self, signer},
    };
    use x509_cert::{Certificate, der::Decode};

    #[test]
    fn document_endorsement_requires_the_pinned_signer_and_exact_document_bytes() {
        let signer = signer();
        let boot = fixture::boot();
        let check = |boot: &BootProof| {
            CheckTrace::default().record(signer.verify_document_endorsement(boot))
        };
        let document = check(&boot).unwrap();
        assert_eq!(
            document,
            base64::decode(&boot.aws_attestation_doc_b64).unwrap()
        );

        let missing = BootProof {
            quorum_key_attestation_signature_hex: None,
            ..boot.clone()
        };
        assert!(matches!(
            check(&missing),
            Err(FailedCheck {
                source: TvcProvenanceError::ProofUnavailable,
                ..
            })
        ));
        for signature in [
            String::new(),
            "00".repeat(64),
            "AA".repeat(64),
            "00".repeat(65),
        ] {
            let changed = BootProof {
                quorum_key_attestation_signature_hex: Some(signature),
                ..boot.clone()
            };
            assert!(matches!(
                check(&changed),
                Err(FailedCheck {
                    source: TvcProvenanceError::EndorsementFailure,
                    ..
                })
            ));
        }
        let mut changed_document = document;
        changed_document[10] ^= 1;
        let changed = BootProof {
            aws_attestation_doc_b64: base64::encode(changed_document),
            ..boot.clone()
        };
        assert!(check(&changed).is_err());
        let wrong_signer = fixture::with_test_roots(
            QuorumPublicKey::from_bytes(fixture::pin()).unwrap(),
            Certificate::from_der(fixture::ROOT).unwrap(),
        );
        assert!(
            CheckTrace::default()
                .record(wrong_signer.verify_document_endorsement(&boot))
                .is_err()
        );
    }

    #[test]
    fn aws_attestation_checks_signature_and_certificate_at_signed_issuance_time() {
        let signer = signer();
        let manifest = fixture::manifest();
        let attestation = fixture::attestation(&manifest);
        let document = fixture::document(&attestation);
        let check = |document: &[u8]| {
            CheckTrace::default().record(signer.verify_aws_attestation_at_issuance(document))
        };
        assert_eq!(check(&document).unwrap().timestamp, fixture::ISSUED_AT);
        assert!(parse_and_verify_der_attestation(&document, fixture::ROOT, 1_793_904_000).is_err());

        let changed_time = AttestationDoc {
            timestamp: 1_735_776_000_000,
            ..attestation
        };
        assert!(check(&fixture::document(&changed_time)).is_err());
        let mut bad_signature = document.clone();
        *bad_signature.last_mut().unwrap() ^= 1;
        assert!(check(&bad_signature).is_err());
        let wrong_root = fixture::with_test_roots(
            QuorumPublicKey::from_bytes(fixture::quorum_key()).unwrap(),
            Certificate::from_der(fixture::LEAF).unwrap(),
        );
        assert!(
            CheckTrace::default()
                .record(wrong_root.verify_aws_attestation_at_issuance(&document))
                .is_err()
        );

        let boot = BootProof {
            created_at: Some(turnkey_client::generated::external::data::v1::Timestamp {
                seconds: "0".into(),
                nanos: "999999999999".into(),
            }),
            ..fixture::boot()
        };
        let key = boot
            .ephemeral_public_key_hex
            .parse::<EphemeralPublicKey>()
            .unwrap();
        let mut observed = BootObservation::default();
        assert!(
            signer
                .authenticate_boot(&key, boot, &mut CheckTrace::default(), &mut observed)
                .is_ok()
        );
        assert_eq!(
            observed.attestation.unwrap().issued_at_ms,
            fixture::ISSUED_AT
        );
    }

    #[test]
    fn boot_binding_matches_the_manifest_measurements_and_ephemeral_key() {
        let signer = signer();
        let manifest = fixture::manifest();
        let boot = fixture::boot();
        let key = boot
            .ephemeral_public_key_hex
            .parse::<EphemeralPublicKey>()
            .unwrap();
        let check = |boot: &BootProof, attestation, evidence: &mut Option<_>| {
            CheckTrace::default().record(signer.check_boot_binding(
                &key,
                boot,
                attestation,
                evidence,
            ))
        };
        let mut evidence = None;
        check(&boot, fixture::attestation(&manifest), &mut evidence).unwrap();
        assert!(evidence.is_some());

        let boot_key_changed = BootProof {
            ephemeral_public_key_hex: format!("{}{}", fixture::point(12), fixture::point(9)),
            ..boot.clone()
        };
        let mut evidence = None;
        assert!(
            check(
                &boot_key_changed,
                fixture::attestation(&manifest),
                &mut evidence
            )
            .is_err()
        );
        assert!(evidence.is_some());

        let invalid_manifest = BootProof {
            qos_manifest_b64: "not-base64".into(),
            ..boot.clone()
        };
        let invalid_envelope = BootProof {
            qos_manifest_envelope_b64: base64::encode([0]),
            ..boot.clone()
        };
        let invalid_key = BootProof {
            ephemeral_public_key_hex: "00".repeat(130),
            ..boot.clone()
        };
        for invalid in [invalid_manifest, invalid_envelope, invalid_key] {
            let mut evidence = None;
            assert!(check(&invalid, fixture::attestation(&manifest), &mut evidence).is_err());
            assert!(evidence.is_none());
        }

        let mut replaced_manifest = fixture::manifest();
        replaced_manifest.namespace.name = "substituted".into();
        let replaced = BootProof {
            qos_manifest_b64: base64::encode(borsh::to_vec(&replaced_manifest).unwrap()),
            ..boot.clone()
        };
        let mut envelope = fixture::envelope(fixture::manifest());
        envelope.manifest_set_approvals[0].signature[0] ^= 1;
        let invalid_approval = BootProof {
            qos_manifest_envelope_b64: base64::encode(borsh::to_vec(&envelope).unwrap()),
            ..boot.clone()
        };
        for invalid in [replaced, invalid_approval] {
            assert!(check(&invalid, fixture::attestation(&manifest), &mut None).is_err());
        }

        let mutate_attestation: [fn(&mut AttestationDoc); 6] = [
            |doc| {
                doc.public_key = Some(
                    hex::decode(format!("{}{}", fixture::point(12), fixture::point(9)))
                        .unwrap()
                        .into(),
                )
            },
            |doc| doc.pcrs.get_mut(&17).unwrap()[0] ^= 1,
            |doc| doc.pcrs.get_mut(&0).unwrap()[0] ^= 1,
            |doc| doc.user_data.as_mut().unwrap()[0] ^= 1,
            |doc| {
                doc.pcrs.remove(&31);
            },
            |doc| doc.nonce = Some(vec![1].into()),
        ];
        for mutate in mutate_attestation {
            let mut doc = fixture::attestation(&manifest);
            mutate(&mut doc);
            assert!(check(&boot, doc, &mut None).is_err());
        }

        let mut wrong_signer_manifest = fixture::manifest();
        wrong_signer_manifest.namespace.quorum_key =
            hex::decode(format!("{}{}", fixture::point(12), fixture::point(11))).unwrap();
        let doc = fixture::attestation(&wrong_signer_manifest);
        let boot = fixture::boot_from(wrong_signer_manifest, &doc);
        assert!(check(&boot, doc, &mut None).is_err());
    }

    #[test]
    fn claim_decode_requires_a_supported_claim_and_request_context() {
        let proofs: Vec<AppProof> = serde_json::from_str(include_str!(
            "../../tests/fixtures/tvc_request_bound_proofs.json"
        ))
        .unwrap();
        let check = |bytes| {
            CheckTrace::default()
                .record(ClaimEvidence::decode(fixture::authenticate_payload(bytes)))
        };
        assert!(matches!(
            check(proofs[0].proof_payload.clone()),
            Ok(ClaimEvidence::Operator { .. })
        ));
        assert!(matches!(
            check(proofs[2].proof_payload.clone()),
            Ok(ClaimEvidence::Quorum { .. })
        ));
        let mut missing_context: serde_json::Value =
            serde_json::from_str(&proofs[0].proof_payload).unwrap();
        missing_context
            .as_object_mut()
            .unwrap()
            .remove("requestContext");
        for payload in [
            missing_context.to_string(),
            r#"{"type":"APP_PROOF_TYPE_POLICY_OUTCOME"}"#.into(),
        ] {
            assert!(matches!(
                check(payload),
                Err(FailedCheck {
                    source: TvcProvenanceError::MalformedEvidence(_),
                    ..
                })
            ));
        }
    }
}
