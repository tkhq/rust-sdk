//! Shared provisioning bundle and attestation verification helpers.

use anyhow::{anyhow, bail, Context};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use qos_core::protocol::services::boot::ManifestEnvelope;
use qos_core::protocol::QosHash;
use qos_p256::P256Public;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ProvisionBundle {
    attestation_document_cose_sign1_base64: String,
    manifest_envelope: ManifestEnvelope,
    fetched_at_unix_ms: u64,
    deployment_id: String,
    ephemeral_public_key_hex: String,
}

impl ProvisionBundle {
    pub(crate) fn new(
        deployment_id: String,
        attestation_document: &[u8],
        manifest_envelope: ManifestEnvelope,
        fetched_at_unix_ms: u64,
        ephemeral_public_key: &[u8],
    ) -> Self {
        Self {
            attestation_document_cose_sign1_base64: BASE64_STANDARD.encode(attestation_document),
            manifest_envelope,
            fetched_at_unix_ms,
            deployment_id,
            ephemeral_public_key_hex: hex::encode(ephemeral_public_key),
        }
    }

    pub(crate) fn manifest_envelope(&self) -> &ManifestEnvelope {
        &self.manifest_envelope
    }

    pub(crate) fn ephemeral_public_key(
        &self,
        dangerous_skip_verification: bool,
    ) -> anyhow::Result<P256Public> {
        self.ephemeral_public_key_with_validation_time(dangerous_skip_verification, None)
    }

    fn ephemeral_public_key_with_validation_time(
        &self,
        dangerous_skip_verification: bool,
        validation_time_override: Option<u64>,
    ) -> anyhow::Result<P256Public> {
        let bundled_public_key = decode_ephemeral_public_key_hex(&self.ephemeral_public_key_hex)?;

        if dangerous_skip_verification {
            return Ok(bundled_public_key);
        }

        let attestation_document = BASE64_STANDARD
            .decode(&self.attestation_document_cose_sign1_base64)
            .context("failed to decode attestation document in provision bundle")?;

        verify_provisioning_details(
            &attestation_document,
            &self.manifest_envelope,
            validation_time_override,
        )?;

        let attestation_doc =
            qos_nsm::nitro::unsafe_attestation_doc_from_der(&attestation_document)
                .context("failed to parse attestation document")?;
        let attested_public_key = extract_ephemeral_public_key_bytes(
            attestation_doc
                .public_key
                .as_ref()
                .map(|public_key| public_key.as_ref()),
        )?;

        if attested_public_key != bundled_public_key.to_bytes() {
            bail!("provision bundle ephemeral public key does not match attestation document");
        }

        Ok(bundled_public_key)
    }
}

pub(crate) fn verify_provisioning_details(
    cose_sign1_der: &[u8],
    manifest_envelope: &ManifestEnvelope,
    validation_time_override: Option<u64>,
) -> anyhow::Result<()> {
    manifest_envelope
        .check_approvals()
        .context("failed to verify manifest approvals")?;

    let attestation_doc = qos_nsm::nitro::attestation_doc_from_der(
        cose_sign1_der,
        &qos_nsm::nitro::cert_from_pem(qos_nsm::nitro::AWS_ROOT_CERT_PEM)
            .context("failed to parse AWS Nitro root certificate")?,
        validation_time_secs(validation_time_override)?,
    )
    .context("failed to parse and verify attestation document")?;

    qos_nsm::nitro::verify_attestation_doc_against_user_input(
        &attestation_doc,
        &manifest_envelope.manifest.qos_hash(),
        &manifest_envelope.manifest.enclave.pcr0,
        &manifest_envelope.manifest.enclave.pcr1,
        &manifest_envelope.manifest.enclave.pcr2,
        &manifest_envelope.manifest.enclave.pcr3,
    )
    .context("attestation document did not match manifest expectations")?;

    Ok(())
}

pub(crate) fn extract_ephemeral_public_key_bytes(
    public_key: Option<&[u8]>,
) -> anyhow::Result<Vec<u8>> {
    let public_key =
        public_key.ok_or_else(|| anyhow!("attestation document missing ephemeral public key"))?;

    P256Public::from_bytes(public_key)
        .map(|ephemeral_key| ephemeral_key.to_bytes())
        .map_err(|err| anyhow!("invalid ephemeral public key: {err:?}"))
}

fn decode_ephemeral_public_key_hex(ephemeral_public_key_hex: &str) -> anyhow::Result<P256Public> {
    let public_key_bytes = hex::decode(ephemeral_public_key_hex.trim())
        .context("failed to decode ephemeral public key from provision bundle")?;

    P256Public::from_bytes(&public_key_bytes)
        .map_err(|err| anyhow!("invalid ephemeral public key in provision bundle: {err:?}"))
}

fn validation_time_secs(validation_time_override: Option<u64>) -> anyhow::Result<u64> {
    match validation_time_override {
        Some(time) => Ok(time),
        None => Ok(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system time before unix epoch")?
            .as_secs()),
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_ephemeral_public_key_bytes, verify_provisioning_details, ProvisionBundle};
    use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
    use qos_core::protocol::services::boot::ManifestEnvelope;
    use qos_p256::P256Pair;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize)]
    struct ValidProvisioningDetailsFixture {
        validation_time_secs: u64,
        attestation_document_cose_sign1_base64: String,
        manifest_envelope: ManifestEnvelope,
    }

    fn valid_provisioning_details_fixture() -> ValidProvisioningDetailsFixture {
        serde_json::from_str(include_str!("../fixtures/valid_provisioning_details.json")).unwrap()
    }

    fn sample_manifest_envelope() -> ManifestEnvelope {
        serde_json::from_value(json!({
            "manifest": {
                "namespace": {
                    "name": "test-namespace",
                    "nonce": 7,
                    "quorumKey": "0102"
                },
                "pivot": {
                    "hash": "0000000000000000000000000000000000000000000000000000000000000000",
                    "restart": "Never",
                    "bridgeConfig": [],
                    "debugMode": false,
                    "args": ["--serve"]
                },
                "manifestSet": {
                    "threshold": 1,
                    "members": [{
                        "alias": "member-1",
                        "pubKey": "aabbcc"
                    }]
                },
                "shareSet": {
                    "threshold": 0,
                    "members": []
                },
                "enclave": {
                    "pcr0": "00",
                    "pcr1": "11",
                    "pcr2": "22",
                    "pcr3": "33",
                    "awsRootCertificate": "44",
                    "qosCommit": "test-commit"
                },
                "patchSet": {
                    "threshold": 0,
                    "members": []
                }
            },
            "manifestSetApprovals": [{
                "signature": "beef",
                "member": {
                    "alias": "member-1",
                    "pubKey": "aabbcc"
                }
            }],
            "shareSetApprovals": []
        }))
        .unwrap()
    }

    #[test]
    fn extract_ephemeral_public_key_bytes_requires_key() {
        let err = extract_ephemeral_public_key_bytes(None).unwrap_err();

        assert!(err
            .to_string()
            .contains("attestation document missing ephemeral public key"));
    }

    #[test]
    fn extract_ephemeral_public_key_bytes_rejects_malformed_key() {
        let err = extract_ephemeral_public_key_bytes(Some(&[1, 2, 3])).unwrap_err();

        assert!(err.to_string().contains("invalid ephemeral public key"));
    }

    #[test]
    fn provision_bundle_serializes_expected_fields() {
        let manifest_envelope = sample_manifest_envelope();
        let bundle = ProvisionBundle::new(
            "deploy-123".to_string(),
            &[1, 2, 3, 4],
            manifest_envelope.clone(),
            1_712_345_678_901,
            &[0x04, 0xab, 0xcd],
        );

        let value = serde_json::to_value(&bundle).unwrap();

        assert_eq!(
            value["attestation_document_cose_sign1_base64"],
            json!(BASE64_STANDARD.encode([1, 2, 3, 4])),
        );
        assert_eq!(value["fetched_at_unix_ms"], json!(1_712_345_678_901_u64));
        assert_eq!(value["deployment_id"], json!("deploy-123"));
        assert_eq!(value["ephemeral_public_key_hex"], json!("04abcd"));
        assert_eq!(
            value["manifest_envelope"],
            serde_json::to_value(&manifest_envelope).unwrap()
        );
    }

    #[test]
    fn verify_provisioning_details_accepts_real_fixture() {
        let fixture = valid_provisioning_details_fixture();
        let attestation_document = BASE64_STANDARD
            .decode(&fixture.attestation_document_cose_sign1_base64)
            .unwrap();

        verify_provisioning_details(
            &attestation_document,
            &fixture.manifest_envelope,
            Some(fixture.validation_time_secs),
        )
        .unwrap();
    }

    #[test]
    fn verify_provisioning_details_rejects_real_fixture_with_missing_manifest_approval() {
        let fixture = valid_provisioning_details_fixture();
        let attestation_document = BASE64_STANDARD
            .decode(&fixture.attestation_document_cose_sign1_base64)
            .unwrap();
        let mut manifest_envelope = fixture.manifest_envelope;
        manifest_envelope.manifest_set_approvals.clear();

        assert!(verify_provisioning_details(
            &attestation_document,
            &manifest_envelope,
            Some(fixture.validation_time_secs),
        )
        .is_err());
    }

    #[test]
    fn safe_bundle_ephemeral_key_extraction_rejects_mismatched_bundle_key() {
        let fixture = valid_provisioning_details_fixture();
        let attestation_document = BASE64_STANDARD
            .decode(&fixture.attestation_document_cose_sign1_base64)
            .unwrap();
        let attestation_doc =
            qos_nsm::nitro::unsafe_attestation_doc_from_der(&attestation_document).unwrap();
        let valid_ephemeral_key = extract_ephemeral_public_key_bytes(
            attestation_doc
                .public_key
                .as_ref()
                .map(|public_key| public_key.as_ref()),
        )
        .unwrap();
        let mut bundle = ProvisionBundle::new(
            "deploy-123".to_string(),
            &attestation_document,
            fixture.manifest_envelope,
            1_712_345_678_901,
            &valid_ephemeral_key,
        );
        bundle.ephemeral_public_key_hex =
            hex::encode(P256Pair::generate().unwrap().public_key().to_bytes());

        let err = match bundle
            .ephemeral_public_key_with_validation_time(false, Some(fixture.validation_time_secs))
        {
            Ok(_) => panic!("mismatched bundle key should be rejected"),
            Err(err) => err,
        };

        assert!(err
            .to_string()
            .contains("does not match attestation document"));
    }

    #[test]
    fn skip_bundle_ephemeral_key_extraction_accepts_valid_bundle_key_without_attestation() {
        let expected_public_key = P256Pair::generate().unwrap().public_key();
        let bundle = ProvisionBundle {
            attestation_document_cose_sign1_base64: "not base64".to_string(),
            manifest_envelope: sample_manifest_envelope(),
            fetched_at_unix_ms: 1_712_345_678_901,
            deployment_id: "deploy-123".to_string(),
            ephemeral_public_key_hex: hex::encode(expected_public_key.to_bytes()),
        };

        let public_key = bundle.ephemeral_public_key(true).unwrap();

        assert_eq!(public_key.to_bytes(), expected_public_key.to_bytes());
    }
}
