use super::Error;
use crate::{
    AWS_ROOT_CERT_PEM, AppProofError, EphemeralPublicKey, ParsedBootProof, cert_from_pem,
    parse_and_verify_der_attestation, unsafe_attestation_doc_from_der, verify_app_proof_signature,
    verify_boot_binding,
};
use p256::ecdsa::{Signature, signature::Verifier};
use turnkey_client::generated::external::data::v1::{AppProof, BootProof};
use turnkey_enclave_encrypt::QuorumPublicKey;

pub(super) const ENDORSEMENT_PREFIX: &[u8] = b"turnkey.boot-proof.attestation.v1\0";

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
    attestation_root: Vec<u8>,
}
impl SelectedEndorsement {
    pub(super) fn for_api_url(api_url: &str) -> Result<Self, Error> {
        let quorum_key = match api_url {
            "https://api.turnkey.com" => QuorumPublicKey::production_signer(),
            "https://api.preprod.turnkey.engineering" => QuorumPublicKey::preprod_signer(),
            "https://api.dev.turnkey.engineering" => QuorumPublicKey::dev_signer(),
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
            attestation_root: cert_from_pem(AWS_ROOT_CERT_PEM)
                .map_err(|_| Error::EndorsementFailure)?,
        })
    }

    pub(super) fn authenticate<'a>(
        &self,
        proof: &'a AppProof,
        boot: &BootProof,
    ) -> Result<AuthenticatedPayload<'a>, Error> {
        let signature = boot
            .quorum_key_attestation_signature_hex
            .as_deref()
            .ok_or(Error::ProofUnavailable)?;
        let document =
            base64::decode(&boot.aws_attestation_doc_b64).map_err(|_| Error::EndorsementFailure)?;
        self.verify_document_signature(&document, signature)?;

        let issued_at = unsafe_attestation_doc_from_der(&document)
            .map_err(|_| Error::EndorsementFailure)?
            .timestamp;
        let attestation =
            parse_and_verify_der_attestation(&document, &self.attestation_root, issued_at / 1000)
                .map_err(|_| Error::EndorsementFailure)?;
        let app_ephemeral_pub_key = EphemeralPublicKey::try_from(proof.public_key.as_str())
            .map_err(|e| AppProofError::InvalidPublicKey(e.to_string()))
            .map_err(|e| Error::MalformedEvidence(e.to_string()))?;
        let parsed_boot_proof =
            ParsedBootProof::parse(boot).map_err(|_| Error::EndorsementFailure)?;
        verify_boot_binding(
            app_ephemeral_pub_key,
            parsed_boot_proof,
            attestation,
            Some(&self.quorum_key),
        )
        .map_err(|_| Error::EndorsementFailure)?;
        verify_app_proof_signature(proof).map_err(|_| Error::EndorsementFailure)?;
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
            attestation_root,
        }
    }
}
