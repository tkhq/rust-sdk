//! Deploy provisioning-details command.

use crate::util::write_file;
use anyhow::{anyhow, bail, Context};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::{Approval, ManifestEnvelope};
use qos_core::protocol::QosHash;
use qos_nsm::types::NsmDigest;
use qos_p256::P256Public;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{
    GetTvcDeploymentProvisioningDetailsRequest, GetTvcDeploymentProvisioningDetailsResponse,
};

/// Get provisioning details for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short = 'd', long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,

    /// Never use for sensitive applications! Skip attestation and approval verification.
    #[arg(long)]
    pub dangerous_skip_verification: bool,

    /// Write provisioning details to a local json bundle usable during re-encryption.
    #[arg(long, value_name = "PATH")]
    pub provision_bundle_out: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProvisionBundle {
    attestation_document_cose_sign1_base64: String,
    manifest_envelope: ManifestEnvelope,
    fetched_at_unix_ms: u64,
    deployment_id: String,
    ephemeral_public_key_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApprovalSummary {
    alias: String,
    public_key: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttestationSummary {
    ephemeral_key: Vec<u8>,
    module_id: String,
    digest: NsmDigest,
    timestamp_ms: u64,
    user_data: Option<Vec<u8>>,
    nonce: Option<Vec<u8>>,
    pcrs: Vec<(usize, Vec<u8>)>,
    certificate_len: usize,
    ca_bundle_cert_count: usize,
    manifest_set_threshold: u32,
    manifest_set_approvals: Vec<ApprovalSummary>,
    share_set_approvals: Vec<ApprovalSummary>,
}

const SUMMARY_PCR_MAX_INDEX: usize = 3;

/// Run the deploy provisioning-details command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let request = GetTvcDeploymentProvisioningDetailsRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id.clone(),
    };
    let fetched_at_unix_ms = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system time before unix epoch")?
            .as_millis(),
    )
    .context("system time exceeded u64 milliseconds")?;

    let GetTvcDeploymentProvisioningDetailsResponse {
        attestation_document,
        manifest_envelope: manifest_envelope_bytes,
    } = auth
        .client
        .get_tvc_deployment_provisioning_details(request)
        .await
        .context("failed to fetch deployment provisioning details")?;

    if attestation_document.is_empty() {
        bail!("attestation document missing in provisioning details response");
    }
    if manifest_envelope_bytes.is_empty() {
        bail!("manifest envelope missing in provisioning details response");
    }

    let manifest_envelope: ManifestEnvelope = serde_json::from_slice(&manifest_envelope_bytes)
        .context("failed to parse manifest envelope from provisioning details")?;

    let summary = build_summary_with_optional_verify(
        &attestation_document,
        &manifest_envelope,
        args.dangerous_skip_verification,
        None,
    )?;

    if let Some(path) = args.provision_bundle_out.as_ref() {
        let bundle = build_provision_bundle(
            args.deploy_id.clone(),
            &attestation_document,
            manifest_envelope.clone(),
            fetched_at_unix_ms,
            &summary.ephemeral_key,
        );
        write_provision_bundle(path, &bundle).await?;
        println!();
    }

    let verification_status = if args.dangerous_skip_verification {
        "skipped attestation and approval verification (--dangerous-skip-verification)"
    } else {
        "verified (attestation + approvals)"
    };
    print_summary(&args.deploy_id, verification_status, &summary);

    Ok(())
}

fn build_provision_bundle(
    deployment_id: String,
    attestation_document: &[u8],
    manifest_envelope: ManifestEnvelope,
    fetched_at_unix_ms: u64,
    ephemeral_public_key: &[u8],
) -> ProvisionBundle {
    ProvisionBundle {
        attestation_document_cose_sign1_base64: BASE64_STANDARD.encode(attestation_document),
        manifest_envelope,
        fetched_at_unix_ms,
        deployment_id,
        ephemeral_public_key_hex: hex::encode(ephemeral_public_key),
    }
}

async fn write_provision_bundle(path: &Path, bundle: &ProvisionBundle) -> anyhow::Result<()> {
    let contents =
        serde_json::to_vec_pretty(bundle).context("failed to serialize provision bundle")?;
    write_file(path, &contents).await?;
    println!("Provision bundle written to: {}", path.display());
    Ok(())
}

fn verify_provisioning_details(
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

fn build_summary_with_optional_verify(
    cose_sign1_der: &[u8],
    manifest_envelope: &ManifestEnvelope,
    dangerous_skip_verification: bool,
    validation_time_override: Option<u64>,
) -> anyhow::Result<AttestationSummary> {
    if !dangerous_skip_verification {
        verify_provisioning_details(cose_sign1_der, manifest_envelope, validation_time_override)?;
    }

    let mut attestation_doc = qos_nsm::nitro::unsafe_attestation_doc_from_der(cose_sign1_der)
        .context("failed to parse attestation document")?;

    Ok(AttestationSummary {
        ephemeral_key: extract_ephemeral_key(
            attestation_doc
                .public_key
                .as_ref()
                .map(|public_key| public_key.as_ref()),
        )?,
        user_data: attestation_doc
            .user_data
            .take()
            .map(|user_data| user_data.into_vec()),
        nonce: attestation_doc.nonce.take().map(|nonce| nonce.into_vec()),
        pcrs: std::mem::take(&mut attestation_doc.pcrs)
            .into_iter()
            .filter(|(index, _)| *index <= SUMMARY_PCR_MAX_INDEX)
            .map(|(index, pcr)| (index, pcr.into_vec()))
            .collect(),
        certificate_len: attestation_doc.certificate.len(),
        ca_bundle_cert_count: attestation_doc.cabundle.len(),
        manifest_set_threshold: manifest_envelope.manifest.manifest_set.threshold,
        manifest_set_approvals: approval_summaries(&manifest_envelope.manifest_set_approvals),
        share_set_approvals: approval_summaries(&manifest_envelope.share_set_approvals),
        module_id: attestation_doc.module_id,
        digest: attestation_doc.digest.into(),
        timestamp_ms: attestation_doc.timestamp,
    })
}

fn approval_summaries(approvals: &[Approval]) -> Vec<ApprovalSummary> {
    approvals
        .iter()
        .map(|approval| ApprovalSummary {
            alias: approval.member.alias.clone(),
            public_key: approval.member.pub_key.clone(),
        })
        .collect()
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

fn extract_ephemeral_key(public_key: Option<&[u8]>) -> anyhow::Result<Vec<u8>> {
    let public_key =
        public_key.ok_or_else(|| anyhow!("attestation document missing ephemeral public key"))?;

    P256Public::from_bytes(public_key)
        .map(|ephemeral_key| ephemeral_key.to_bytes())
        .map_err(|err| anyhow!("invalid ephemeral public key: {err:?}"))
}

fn print_summary(deploy_id: &str, verification_status: &str, summary: &AttestationSummary) {
    println!("Deployment: {deploy_id}");
    println!("Verification: {verification_status}");
    println!("Ephemeral Key: {}", hex::encode(&summary.ephemeral_key));
    println!("Module ID: {}", summary.module_id);
    println!("Digest: {:?}", summary.digest);
    println!("Timestamp (ms): {}", summary.timestamp_ms);
    println!(
        "User Data: {}",
        summary
            .user_data
            .as_ref()
            .map(hex::encode)
            .unwrap_or_else(|| "(none)".to_string())
    );
    println!(
        "Nonce: {}",
        summary
            .nonce
            .as_ref()
            .map(hex::encode)
            .unwrap_or_else(|| "(none)".to_string())
    );
    println!("PCRs:");
    for (index, pcr) in &summary.pcrs {
        println!("  PCR{index}: {}", hex::encode(pcr));
    }
    println!("Certificate Length: {} bytes", summary.certificate_len);
    println!("CA Bundle Certificates: {}", summary.ca_bundle_cert_count);
    println!(
        "Manifest Set Approvals: {}/{}",
        summary.manifest_set_approvals.len(),
        summary.manifest_set_threshold
    );
    print_approval_summary_entries(&summary.manifest_set_approvals);
    if summary.share_set_approvals.is_empty() {
        println!("Share Set Approvals: (none)");
    } else {
        println!("Share Set Approvals: {}", summary.share_set_approvals.len());
        print_approval_summary_entries(&summary.share_set_approvals);
    }
}

fn print_approval_summary_entries(approvals: &[ApprovalSummary]) {
    for approval in approvals {
        println!(
            "  {}: {}",
            approval.alias,
            hex::encode(&approval.public_key)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{build_provision_bundle, extract_ephemeral_key, verify_provisioning_details};
    use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
    use qos_core::protocol::services::boot::ManifestEnvelope;
    use serde::Deserialize;
    use serde_json::json;

    #[derive(Debug, Deserialize)]
    struct ValidProvisioningDetailsFixture {
        validation_time_secs: u64,
        attestation_document_cose_sign1_base64: String,
        manifest_envelope: ManifestEnvelope,
    }

    fn valid_provisioning_details_fixture() -> ValidProvisioningDetailsFixture {
        serde_json::from_str(include_str!(
            "../../../fixtures/valid_provisioning_details.json"
        ))
        .unwrap()
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
    fn extract_ephemeral_key_requires_key() {
        let err = extract_ephemeral_key(None).unwrap_err();

        assert!(err
            .to_string()
            .contains("attestation document missing ephemeral public key"));
    }

    #[test]
    fn extract_ephemeral_key_rejects_malformed_key() {
        let err = extract_ephemeral_key(Some(&[1, 2, 3])).unwrap_err();

        assert!(err.to_string().contains("invalid ephemeral public key"));
    }

    #[test]
    fn provision_bundle_serializes_expected_fields() {
        let manifest_envelope = sample_manifest_envelope();
        let bundle = build_provision_bundle(
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
}
