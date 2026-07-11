//! Deploy provisioning-details command.

use crate::outcome::Outcome;
use crate::output::{Message, StdCtx};
use crate::provisioning::{
    ProvisionBundle, extract_ephemeral_public_key_bytes, verify_provisioning_details,
};
use crate::util::write_file;
use anyhow::{Context, bail};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::{Approval, VersionedManifestEnvelope};
use qos_nsm::types::NsmDigest;
use serde::Serialize;
use std::fmt::Write;
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
pub async fn run(_ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
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

    let bundle_path = match args.provision_bundle_out.as_ref() {
        Some(path) => {
            let bundle = ProvisionBundle::new(
                args.deploy_id.clone(),
                &attestation_document,
                manifest_envelope.clone(),
                fetched_at_unix_ms,
                &summary.ephemeral_key,
            );
            write_provision_bundle(path, &bundle).await?;
            Some(path.display().to_string())
        }
        None => None,
    };

    let verification_status = if args.dangerous_skip_verification {
        "skipped attestation, PCR, and approval verification (--dangerous-skip-verification)"
    } else {
        "verified (attestation + approvals)"
    };

    Ok(Outcome::DeployProvisioningDetails(
        ProvisioningDetails::from_summary(
            args.deploy_id,
            verification_status,
            bundle_path,
            &summary,
        ),
    ))
}

async fn write_provision_bundle(path: &Path, bundle: &ProvisionBundle) -> anyhow::Result<()> {
    let contents =
        serde_json::to_vec_pretty(bundle).context("failed to serialize provision bundle")?;
    write_file(path, &contents).await?;
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

/// Wide terminal outcome for `deploy provisioning-details`. Byte fields are
/// hex-encoded; `digest` carries the Debug rendering of the NSM digest so the
/// same string serves both the payload and the human line.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvisioningDetails {
    deployment_id: String,
    verification: String,
    ephemeral_key: String,
    module_id: String,
    digest: String,
    timestamp_ms: u64,
    user_data: Option<String>,
    nonce: Option<String>,
    pcrs: Vec<PcrEntry>,
    certificate_length: usize,
    ca_bundle_certificates: usize,
    manifest_set_threshold: u32,
    manifest_set_approvals: Vec<ApprovalEntry>,
    share_set_approvals: Vec<ApprovalEntry>,
    /// Present when `--provision-bundle-out` wrote a bundle file.
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PcrEntry {
    index: usize,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApprovalEntry {
    alias: String,
    public_key: String,
}

impl ProvisioningDetails {
    fn from_summary(
        deployment_id: String,
        verification: &str,
        bundle_path: Option<String>,
        summary: &AttestationSummary,
    ) -> Self {
        Self {
            deployment_id,
            verification: verification.to_string(),
            ephemeral_key: hex::encode(&summary.ephemeral_key),
            module_id: summary.module_id.clone(),
            digest: format!("{:?}", summary.digest),
            timestamp_ms: summary.timestamp_ms,
            user_data: summary.user_data.as_ref().map(hex::encode),
            nonce: summary.nonce.as_ref().map(hex::encode),
            pcrs: summary
                .pcrs
                .iter()
                .map(|(index, pcr)| PcrEntry {
                    index: *index,
                    value: hex::encode(pcr),
                })
                .collect(),
            certificate_length: summary.certificate_len,
            ca_bundle_certificates: summary.ca_bundle_cert_count,
            manifest_set_threshold: summary.manifest_set_threshold,
            manifest_set_approvals: approval_entries(&summary.manifest_set_approvals),
            share_set_approvals: approval_entries(&summary.share_set_approvals),
            bundle_path,
        }
    }
}

fn approval_entries(approvals: &[ApprovalSummary]) -> Vec<ApprovalEntry> {
    approvals
        .iter()
        .map(|approval| ApprovalEntry {
            alias: approval.alias.clone(),
            public_key: hex::encode(&approval.public_key),
        })
        .collect()
}

impl Message for ProvisioningDetails {
    fn reason(&self) -> &'static str {
        "provisioning-details"
    }

    fn human_message(&self) -> String {
        let mut message = String::new();

        if let Some(path) = &self.bundle_path {
            let _ = write!(message, "Provision bundle written to: {path}\n\n");
        }

        // Fixed attestation summary; `PCRs:` deliberately has no trailing
        // newline so the PCR loop below appends its own leading-newline lines
        // (and the section reads correctly even when there are no PCRs).
        let _ = write!(
            message,
            r#"Deployment: {}
Verification: {}
Ephemeral Key: {}
Module ID: {}
Digest: {}
Timestamp (ms): {}
User Data: {}
Nonce: {}
PCRs:"#,
            self.deployment_id,
            self.verification,
            self.ephemeral_key,
            self.module_id,
            self.digest,
            self.timestamp_ms,
            self.user_data.as_deref().unwrap_or("(none)"),
            self.nonce.as_deref().unwrap_or("(none)"),
        );

        for pcr in &self.pcrs {
            let label = match pcr.index {
                16 => " (setup manifest/key commitment)",
                17 => " (live manifest/key commitment)",
                _ => "",
            };
            let _ = write!(message, "\n  PCR{}{label}: {}", pcr.index, pcr.value);
        }

        let _ = write!(
            message,
            r#"
Certificate Length: {} bytes
CA Bundle Certificates: {}
Manifest Set Approvals: {}/{}"#,
            self.certificate_length,
            self.ca_bundle_certificates,
            self.manifest_set_approvals.len(),
            self.manifest_set_threshold,
        );
        write_approval_entries(&mut message, &self.manifest_set_approvals);

        if self.share_set_approvals.is_empty() {
            let _ = write!(message, "\nShare Set Approvals: (none)");
        } else {
            let _ = write!(
                message,
                "\nShare Set Approvals: {}",
                self.share_set_approvals.len()
            );
            write_approval_entries(&mut message, &self.share_set_approvals);
        }

        message
    }
}

fn write_approval_entries(message: &mut String, approvals: &[ApprovalEntry]) {
    for approval in approvals {
        let _ = write!(message, "\n  {}: {}", approval.alias, approval.public_key);
    }
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

    use super::{ApprovalEntry, PcrEntry, ProvisioningDetails};
    use crate::output::Message;

    fn full_details() -> ProvisioningDetails {
        ProvisioningDetails {
            deployment_id: "dep_123".to_string(),
            verification: "verified".to_string(),
            ephemeral_key: "abcd".to_string(),
            module_id: "mod-1".to_string(),
            digest: "SHA384".to_string(),
            timestamp_ms: 1_700_000_000_000,
            user_data: Some("aa".to_string()),
            nonce: Some("bb".to_string()),
            pcrs: vec![
                PcrEntry { index: 0, value: "00".to_string() },
                PcrEntry { index: 16, value: "1616".to_string() },
                PcrEntry { index: 17, value: "1717".to_string() },
            ],
            certificate_length: 1234,
            ca_bundle_certificates: 3,
            manifest_set_threshold: 2,
            manifest_set_approvals: vec![
                ApprovalEntry { alias: "alice".to_string(), public_key: "aaaa".to_string() },
                ApprovalEntry { alias: "bob".to_string(), public_key: "bbbb".to_string() },
            ],
            share_set_approvals: vec![ApprovalEntry {
                alias: "carol".to_string(),
                public_key: "cccc".to_string(),
            }],
            bundle_path: Some("/tmp/bundle.json".to_string()),
        }
    }

    #[test]
    fn human_message_full_golden() {
        assert_eq!(
            full_details().human_message(),
            r#"Provision bundle written to: /tmp/bundle.json

Deployment: dep_123
Verification: verified
Ephemeral Key: abcd
Module ID: mod-1
Digest: SHA384
Timestamp (ms): 1700000000000
User Data: aa
Nonce: bb
PCRs:
  PCR0: 00
  PCR16 (setup manifest/key commitment): 1616
  PCR17 (live manifest/key commitment): 1717
Certificate Length: 1234 bytes
CA Bundle Certificates: 3
Manifest Set Approvals: 2/2
  alice: aaaa
  bob: bbbb
Share Set Approvals: 1
  carol: cccc"#
        );
    }

    #[test]
    fn human_message_minimal_golden() {
        let details = ProvisioningDetails::default();
        // NOTE: the first five lines have empty values, so they end in a
        // significant trailing space — do not strip trailing whitespace here.
        assert_eq!(
            details.human_message(),
            r#"Deployment: 
Verification: 
Ephemeral Key: 
Module ID: 
Digest: 
Timestamp (ms): 0
User Data: (none)
Nonce: (none)
PCRs:
Certificate Length: 0 bytes
CA Bundle Certificates: 0
Manifest Set Approvals: 0/0
Share Set Approvals: (none)"#
        );
    }
}
