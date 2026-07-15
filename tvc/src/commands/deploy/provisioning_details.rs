//! Deploy provisioning-details command.

use crate::output::StdCtx;
use crate::provisioning::{
    ProvisionBundle, extract_ephemeral_public_key_bytes, verify_provisioning_details,
};
use crate::shell_println;
use crate::util::write_file;
use anyhow::{Context, bail};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::{Approval, VersionedManifestEnvelope};
use qos_nsm::types::NsmDigest;
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

    /// Never use for sensitive applications! Skip attestation, PCR, and approval verification.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_VERIFICATION")]
    pub dangerous_skip_verification: bool,

    /// Write provisioning details to a local json bundle usable during re-encryption.
    #[arg(long, value_name = "PATH", env = "TVC_PROVISION_BUNDLE_OUT")]
    pub provision_bundle_out: Option<PathBuf>,
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

const SUMMARY_PCR_MAX_INDEX: usize = 17;

/// Run the deploy provisioning-details command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<()> {
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

    let manifest_envelope =
        VersionedManifestEnvelope::try_from_slice_compat(&manifest_envelope_bytes)
            .context("failed to parse manifest envelope from provisioning details")?;

    let summary = build_summary_with_optional_verify(
        &attestation_document,
        &manifest_envelope,
        args.dangerous_skip_verification,
        None,
    )?;

    if let Some(path) = args.provision_bundle_out.as_ref() {
        let bundle = ProvisionBundle::new(
            args.deploy_id.clone(),
            &attestation_document,
            manifest_envelope.clone(),
            fetched_at_unix_ms,
            &summary.ephemeral_key,
        );
        write_provision_bundle(ctx, path, &bundle).await?;
        shell_println!(ctx)?;
    }

    let verification_status = if args.dangerous_skip_verification {
        "skipped attestation, PCR, and approval verification (--dangerous-skip-verification)"
    } else {
        "verified (attestation + approvals)"
    };
    print_summary(ctx, &args.deploy_id, verification_status, &summary)?;

    Ok(())
}

async fn write_provision_bundle(
    ctx: &mut StdCtx,
    path: &Path,
    bundle: &ProvisionBundle,
) -> anyhow::Result<()> {
    let contents =
        serde_json::to_vec_pretty(bundle).context("failed to serialize provision bundle")?;
    write_file(path, &contents).await?;
    shell_println!(ctx, "Provision bundle written to: {}", path.display())?;
    Ok(())
}

fn build_summary_with_optional_verify(
    cose_sign1_der: &[u8],
    manifest_envelope: &VersionedManifestEnvelope,
    dangerous_skip_verification: bool,
    validation_time_override: Option<u64>,
) -> anyhow::Result<AttestationSummary> {
    let mut attestation_doc = if dangerous_skip_verification {
        qos_nsm::nitro::unsafe_attestation_doc_from_der(cose_sign1_der)
            .context("failed to parse attestation document")?
    } else {
        verify_provisioning_details(cose_sign1_der, manifest_envelope, validation_time_override)?
    };

    let manifest = manifest_envelope.clone().manifest();

    Ok(AttestationSummary {
        ephemeral_key: extract_ephemeral_public_key_bytes(
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
        manifest_set_threshold: manifest.manifest_set().threshold,
        manifest_set_approvals: approval_summaries(manifest_envelope.manifest_set_approvals()),
        share_set_approvals: approval_summaries(manifest_envelope.share_set_approvals()),
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

fn print_summary(
    ctx: &mut StdCtx,
    deploy_id: &str,
    verification_status: &str,
    summary: &AttestationSummary,
) -> anyhow::Result<()> {
    shell_println!(ctx, "Deployment: {deploy_id}")?;
    shell_println!(ctx, "Verification: {verification_status}")?;
    shell_println!(
        ctx,
        "Ephemeral Key: {}",
        hex::encode(&summary.ephemeral_key)
    )?;
    shell_println!(ctx, "Module ID: {}", summary.module_id)?;
    shell_println!(ctx, "Digest: {:?}", summary.digest)?;
    shell_println!(ctx, "Timestamp (ms): {}", summary.timestamp_ms)?;
    shell_println!(
        ctx,
        "User Data: {}",
        summary
            .user_data
            .as_ref()
            .map(hex::encode)
            .unwrap_or_else(|| "(none)".to_string())
    )?;
    shell_println!(
        ctx,
        "Nonce: {}",
        summary
            .nonce
            .as_ref()
            .map(hex::encode)
            .unwrap_or_else(|| "(none)".to_string())
    )?;
    shell_println!(ctx, "PCRs:")?;
    for (index, pcr) in &summary.pcrs {
        let label = match *index {
            16 => " (setup manifest/key commitment)",
            17 => " (live manifest/key commitment)",
            _ => "",
        };
        shell_println!(ctx, "  PCR{index}{label}: {}", hex::encode(pcr))?;
    }
    shell_println!(ctx, "Certificate Length: {} bytes", summary.certificate_len)?;
    shell_println!(
        ctx,
        "CA Bundle Certificates: {}",
        summary.ca_bundle_cert_count
    )?;
    shell_println!(
        ctx,
        "Manifest Set Approvals: {}/{}",
        summary.manifest_set_approvals.len(),
        summary.manifest_set_threshold
    )?;
    print_approval_summary_entries(ctx, &summary.manifest_set_approvals)?;
    if summary.share_set_approvals.is_empty() {
        shell_println!(ctx, "Share Set Approvals: (none)")?;
    } else {
        shell_println!(
            ctx,
            "Share Set Approvals: {}",
            summary.share_set_approvals.len()
        )?;
        print_approval_summary_entries(ctx, &summary.share_set_approvals)?;
    }
    Ok(())
}

fn print_approval_summary_entries(
    ctx: &mut StdCtx,
    approvals: &[ApprovalSummary],
) -> anyhow::Result<()> {
    for approval in approvals {
        shell_println!(
            ctx,
            "  {}: {}",
            approval.alias,
            hex::encode(&approval.public_key)
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::build_summary_with_optional_verify;
    use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
    use qos_core::protocol::services::boot::VersionedManifestEnvelope;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct ValidProvisioningDetailsFixture {
        validation_time_secs: u64,
        attestation_document_cose_sign1_base64: String,
        manifest_envelope: VersionedManifestEnvelope,
    }

    fn valid_provisioning_details_fixture() -> ValidProvisioningDetailsFixture {
        serde_json::from_str(include_str!(
            "../../../fixtures/valid_provisioning_details.json"
        ))
        .unwrap()
    }

    #[test]
    fn build_summary_accepts_real_fixture() {
        let fixture = valid_provisioning_details_fixture();
        let attestation_document = BASE64_STANDARD
            .decode(&fixture.attestation_document_cose_sign1_base64)
            .unwrap();

        let summary = build_summary_with_optional_verify(
            &attestation_document,
            &fixture.manifest_envelope,
            false,
            Some(fixture.validation_time_secs),
        )
        .unwrap();

        let manifest = fixture.manifest_envelope.clone().manifest();
        assert!(!summary.ephemeral_key.is_empty());
        assert_eq!(
            summary.manifest_set_threshold,
            manifest.manifest_set().threshold
        );
        assert_eq!(
            summary.manifest_set_approvals.len(),
            fixture.manifest_envelope.manifest_set_approvals().len()
        );
    }

    #[test]
    fn build_summary_rejects_real_fixture_with_missing_manifest_approval() {
        let fixture = valid_provisioning_details_fixture();
        let attestation_document = BASE64_STANDARD
            .decode(&fixture.attestation_document_cose_sign1_base64)
            .unwrap();
        let mut manifest_envelope = fixture.manifest_envelope;
        match &mut manifest_envelope {
            VersionedManifestEnvelope::V2(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V1(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V0(envelope) => envelope.manifest_set_approvals.clear(),
        }

        assert!(
            build_summary_with_optional_verify(
                &attestation_document,
                &manifest_envelope,
                false,
                Some(fixture.validation_time_secs),
            )
            .is_err()
        );
    }
}
