use super::{
    Error,
    report::{
        AttestationEvidence,
        CheckState::{Fail, Pass},
        EnvelopeReport,
        ProofCheck::*,
    },
};
use crate::{
    AWS_ROOT_CERT_PEM, AppProofError, EphemeralPublicKey, ParsedBootProof, cert_from_pem,
    parse_and_verify_der_attestation, unsafe_attestation_doc_from_der, verify_app_proof_signature,
    verify_boot_binding,
};
use p256::ecdsa::{Signature, signature::Verifier};
use turnkey_client::generated::external::data::v1::{AppProof, BootProof};
use turnkey_enclave_encrypt::QuorumPublicKey;

/// Domain separation for Signer endorsements of raw boot attestation documents.
pub const ENDORSEMENT_PREFIX: &[u8] = b"turnkey.boot-proof.attestation.v1\0";

pub(super) struct AuthenticatedPayload<'a> {
    bytes: &'a str,
}
impl AuthenticatedPayload<'_> {
    pub(super) fn bytes(&self) -> &str {
        self.bytes
    }
}

pub(super) struct SelectedEndorsement {
    quorum_key: QuorumPublicKey,
    environment: &'static str,
    attestation_root: Vec<u8>,
}
impl SelectedEndorsement {
    pub(super) fn for_api_url(api_url: &str) -> Result<Self, Error> {
        let (quorum_key, environment) = match api_url {
            "https://api.turnkey.com" => (
                QuorumPublicKey::production_signer(),
                "production (https://api.turnkey.com)",
            ),
            "https://api.preprod.turnkey.engineering" => (
                QuorumPublicKey::preprod_signer(),
                "preproduction (https://api.preprod.turnkey.engineering)",
            ),
            "https://api.dev.turnkey.engineering" => (
                QuorumPublicKey::dev_signer(),
                "development (https://api.dev.turnkey.engineering)",
            ),
            _ => return Err(Error::UnsupportedEnvironment),
        };
        quorum_key
            .encrypt_public_key()
            .map_err(|_| Error::EndorsementFailure)?;
        quorum_key
            .verifying_key()
            .map_err(|_| Error::EndorsementFailure)?;
        Ok(Self {
            quorum_key,
            environment,
            attestation_root: cert_from_pem(AWS_ROOT_CERT_PEM)
                .map_err(|_| Error::EndorsementFailure)?,
        })
    }

    pub(super) fn environment(&self) -> &'static str {
        self.environment
    }
    pub(super) fn quorum_key(&self) -> String {
        hex::encode(self.quorum_key.as_ref())
    }

    #[cfg(test)]
    pub(super) fn authenticate<'a>(
        &self,
        proof: &'a AppProof,
        boot: &BootProof,
    ) -> Result<AuthenticatedPayload<'a>, Error> {
        self.authenticate_reported(proof, boot, &mut EnvelopeReport::new())
    }

    pub(super) fn authenticate_reported<'a>(
        &self,
        proof: &'a AppProof,
        boot: &BootProof,
        report: &mut EnvelopeReport,
    ) -> Result<AuthenticatedPayload<'a>, Error> {
        report.set(DocumentEndorsement, Fail);
        let signature = boot
            .quorum_key_attestation_signature_hex
            .as_deref()
            .ok_or(Error::ProofUnavailable)?;
        let document =
            base64::decode(&boot.aws_attestation_doc_b64).map_err(|_| Error::EndorsementFailure)?;
        self.verify_document_signature(&document, signature)?;
        report.set(DocumentEndorsement, Pass);
        report.set(AwsAttestationAtIssuance, Fail);
        let issued_at = unsafe_attestation_doc_from_der(&document)
            .map_err(|_| Error::EndorsementFailure)?
            .timestamp;
        let attestation =
            parse_and_verify_der_attestation(&document, &self.attestation_root, issued_at / 1000)
                .map_err(|_| Error::EndorsementFailure)?;
        report.attestation = Some(AttestationEvidence {
            issued_at_ms: attestation.timestamp,
            public_key: attestation.public_key.as_ref().map(hex::encode),
            manifest_commitment: attestation.user_data.as_ref().map(hex::encode),
            pcrs: attestation
                .pcrs
                .iter()
                .map(|(index, value)| (*index, hex::encode(value)))
                .collect(),
        });
        report.set(AwsAttestationAtIssuance, Pass);
        report.set(BootBinding, Fail);
        let app_ephemeral_pub_key = EphemeralPublicKey::try_from(proof.public_key.as_str())
            .map_err(|e| AppProofError::InvalidPublicKey(e.to_string()))
            .map_err(|e| Error::MalformedEvidence(e.to_string()))?;
        let parsed = ParsedBootProof::parse(boot).map_err(|_| Error::EndorsementFailure)?;
        report.boot = Some(parsed.evidence());
        verify_boot_binding(
            app_ephemeral_pub_key,
            parsed,
            attestation,
            Some(&self.quorum_key),
        )
        .map_err(|_| Error::EndorsementFailure)?;
        report.set(BootBinding, Pass);
        report.set(ApplicationSignature, Fail);
        verify_app_proof_signature(proof).map_err(|_| Error::EndorsementFailure)?;
        report.set(ApplicationSignature, Pass);
        Ok(AuthenticatedPayload {
            bytes: &proof.proof_payload,
        })
    }

    pub(super) fn verify_document_signature(
        &self,
        document: &[u8],
        signature: &str,
    ) -> Result<(), Error> {
        if signature.len() != 128
            || !signature
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
        {
            return Err(Error::EndorsementFailure);
        }
        let signature = hex::decode(signature).map_err(|_| Error::EndorsementFailure)?;
        let signature = Signature::from_slice(&signature).map_err(|_| Error::EndorsementFailure)?;
        let key = self
            .quorum_key
            .verifying_key()
            .map_err(|_| Error::EndorsementFailure)?;
        let mut message = Vec::with_capacity(ENDORSEMENT_PREFIX.len() + document.len());
        message.extend_from_slice(ENDORSEMENT_PREFIX);
        message.extend_from_slice(document);
        key.verify(&message, &signature)
            .map_err(|_| Error::EndorsementFailure)?;

        Ok(())
    }

    #[cfg(test)]
    pub(super) fn with_test_roots(quorum_key: [u8; 130], attestation_root: Vec<u8>) -> Self {
        Self {
            quorum_key: QuorumPublicKey::from_bytes(quorum_key).unwrap(),
            environment: "synthetic test policy",
            attestation_root,
        }
    }
}
