//! Provision one hosted quorum-key share for a deployment.

use crate::{
    client::{build_client, fetch_tvc_deployment},
    config::turnkey::Config,
    operator::{
        ResolvedHostedOperator, ensure_authenticated_org, hosted_activity_error,
        resolve_hosted_operator, timestamp_ms,
    },
    outcome::Outcome,
    output::StdCtx,
    provisioning::{
        FetchedProvisioningDetails, fetch_provisioning_details, verify_provisioning_details,
    },
    shell_eprintln,
};
use anyhow::{Context, Result, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::VersionedManifest;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use tracing::instrument;
use turnkey_client::generated::{
    ReEncryptTvcQuorumKeyShareIntent, ReEncryptTvcQuorumKeyShareResult,
    external::data::v1::TvcManifest,
};
use uuid::Uuid;

/// Provision one hosted quorum-key share for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to provision.
    #[arg(short = 'd', long, env = "TVC_DEPLOY_ID")]
    deploy_id: Uuid,

    /// Hosted operator UUID whose quorum-key share should be provisioned.
    #[arg(long, env = "TVC_OPERATOR_ID")]
    operator_id: Uuid,

    /// Never use for sensitive applications! Skip attestation, PCR, and approval verification.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_VERIFICATION")]
    dangerous_skip_verification: bool,
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq, Eq))]
#[serde(rename_all = "camelCase")]
pub struct ProvisioningShareCreated {
    provisioning_share_id: String,
}

impl Display for ProvisioningShareCreated {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Provisioning Share ID: {}",
            self.provisioning_share_id
        )
    }
}

/// Run the hosted deploy provision command.
#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let Args {
        deploy_id,
        operator_id,
        dangerous_skip_verification,
    } = args;

    if dangerous_skip_verification {
        shell_eprintln!(
            ctx,
            "WARNING: Skipping attestation, PCR, and manifest approval verification! This is dangerous and should not be used for sensitive applications."
        )?;
    }

    let config = Config::load().await?;
    let operator = resolve_hosted_operator(&config, &operator_id)?;
    let auth = build_client().await?;
    ensure_authenticated_org(&auth.org_id, operator.organization_id())?;

    let details = fetch_provisioning_details(&auth, &deploy_id).await?;
    let deployment =
        fetch_tvc_deployment(&auth, auth.org_id.clone(), deploy_id.to_string()).await?;
    let TvcManifest {
        id: _,
        manifest: deployment_manifest,
        created_at: _,
        updated_at: _,
    } = deployment
        .manifest
        .context("deployment response missing manifest")?;
    ensure!(
        !deployment_manifest.is_empty(),
        "deployment response contained an empty manifest"
    );
    let intent = build_re_encrypt_intent(
        details,
        deployment_manifest,
        &operator,
        dangerous_skip_verification,
        None,
    )?;
    let result = auth
        .client
        .re_encrypt_tvc_quorum_key_share(auth.org_id, timestamp_ms()?, intent)
        .await
        .map_err(|error| hosted_activity_error("re-encrypt hosted TVC quorum-key share", error))?;
    let output = validate_result(result.result)?;

    Ok(Outcome::ProvisioningShareCreated(output))
}

fn build_re_encrypt_intent(
    details: FetchedProvisioningDetails,
    deployment_manifest_bytes: Vec<u8>,
    operator: &ResolvedHostedOperator,
    dangerous_skip_verification: bool,
    validation_time_override: Option<u64>,
) -> Result<ReEncryptTvcQuorumKeyShareIntent> {
    let (deployment_id, attestation_document, manifest_envelope, _) = details.into_parts();
    if dangerous_skip_verification {
        qos_nsm::nitro::unsafe_attestation_doc_from_der(&attestation_document)
            .context("failed to parse attestation document")?;
    } else {
        verify_provisioning_details(
            &attestation_document,
            &manifest_envelope,
            validation_time_override,
        )?;
    }

    let verified_manifest = manifest_envelope.manifest();
    let deployment_manifest = VersionedManifest::try_from_slice_compat(&deployment_manifest_bytes)
        .context("failed to parse deployment manifest")?;
    ensure!(
        deployment_manifest == verified_manifest,
        "deployment manifest does not match provisioning manifest envelope"
    );
    let composite_public_key = operator.composite_public_key();
    ensure!(
        deployment_manifest
            .share_set()
            .members
            .iter()
            .any(|member| member.pub_key == composite_public_key),
        "hosted operator '{}' ({}) is not part of the manifest share set",
        operator.name(),
        operator.operator_id(),
    );

    let app_quorum_key = hex::encode(&deployment_manifest.namespace().quorum_key);

    Ok(ReEncryptTvcQuorumKeyShareIntent {
        attestation_doc_b64: BASE64_STANDARD.encode(attestation_document),
        manifest_b64: BASE64_STANDARD.encode(deployment_manifest_bytes),
        operator_encrypt_key: operator.encrypt_public_key().to_string(),
        operator_sign_key: operator.sign_public_key().to_string(),
        deployment_id: deployment_id.to_string(),
        app_quorum_key,
    })
}

fn validate_result(result: ReEncryptTvcQuorumKeyShareResult) -> Result<ProvisioningShareCreated> {
    let ReEncryptTvcQuorumKeyShareResult {
        provisioning_share_id,
    } = result;
    ensure!(
        !provisioning_share_id.trim().is_empty(),
        "re-encrypt TVC quorum-key share response contained an empty provisioning share ID"
    );

    Ok(ProvisioningShareCreated {
        provisioning_share_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
    };
    use indexmap::IndexMap;
    use qos_core::protocol::services::boot::VersionedManifestEnvelope;
    use serde::Deserialize;
    use std::path::PathBuf;

    const DEPLOYMENT_ID: &str = "33333333-3333-4333-8333-333333333333";
    const OPERATOR_ID: &str = "11111111-1111-4111-8111-111111111111";
    const WALLET_ID: &str = "22222222-2222-4222-8222-222222222222";
    const FETCHED_AT_UNIX_MS: u64 = 1_712_345_678_901;

    #[derive(Deserialize)]
    struct ValidProvisioningDetailsFixture {
        validation_time_secs: u64,
        attestation_document_cose_sign1_base64: String,
        manifest_envelope: VersionedManifestEnvelope,
    }

    fn fixture() -> ValidProvisioningDetailsFixture {
        serde_json::from_str(include_str!(
            "../../../fixtures/valid_provisioning_details.json"
        ))
        .unwrap()
    }

    fn details(
        attestation_document_cose_sign1_base64: &str,
        manifest_envelope: VersionedManifestEnvelope,
    ) -> FetchedProvisioningDetails {
        FetchedProvisioningDetails::new(
            Uuid::parse_str(DEPLOYMENT_ID).unwrap(),
            BASE64_STANDARD
                .decode(attestation_document_cose_sign1_base64)
                .unwrap(),
            manifest_envelope,
            FETCHED_AT_UNIX_MS,
        )
    }

    fn config_for_member(fixture: &ValidProvisioningDetailsFixture) -> Config {
        let composite_key = match &fixture.manifest_envelope {
            VersionedManifestEnvelope::V2(envelope) => {
                &envelope.manifest.share_set.members[0].pub_key
            }
            VersionedManifestEnvelope::V1(envelope) => {
                &envelope.manifest.share_set.members[0].pub_key
            }
            VersionedManifestEnvelope::V0(envelope) => {
                &envelope.manifest.share_set.members[0].pub_key
            }
        };
        let encrypt_public_key = hex::encode(&composite_key[..65]);
        let sign_public_key = hex::encode(&composite_key[65..]);
        let record = OperatorRecord {
            name: "hosted-operator".to_string(),
            kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                operator_id: Uuid::parse_str(OPERATOR_ID).unwrap(),
                wallet_id: Uuid::parse_str(WALLET_ID).unwrap(),
                path: "m/5527107'/0'/0'".to_string(),
                encrypt_public_key,
                sign_public_key,
                extra: toml::Table::new(),
            }),
        };

        Config {
            active_org: Some("active".to_string()),
            orgs: IndexMap::from([(
                "active".to_string(),
                OrgConfig {
                    id: Uuid::from_u128(0xA1),
                    api_key_path: PathBuf::from("api-key.json"),
                    api_base_url: "https://api.turnkey.com".to_string(),
                    default_operator_kind: OperatorKind::Hosted,
                    operators: vec![record],
                    default_alias: false,
                    extra: toml::Table::new(),
                },
            )]),
            ..Config::default()
        }
    }

    fn resolved_member(fixture: &ValidProvisioningDetailsFixture) -> ResolvedHostedOperator {
        resolve_hosted_operator(
            &config_for_member(fixture),
            &Uuid::parse_str(OPERATOR_ID).unwrap(),
        )
        .unwrap()
    }

    fn deployment_manifest_bytes(manifest_envelope: VersionedManifestEnvelope) -> Vec<u8> {
        manifest_envelope.manifest().to_storage_vec().unwrap()
    }

    #[test]
    fn passes_through_exact_deployment_manifest_bytes_after_safe_verification() {
        let fixture = fixture();
        let details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let operator = resolved_member(&fixture);
        let manifest = fixture.manifest_envelope.clone().manifest();
        let deployment_manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        assert_ne!(
            deployment_manifest_bytes,
            manifest.to_storage_vec().unwrap()
        );

        let expected = ReEncryptTvcQuorumKeyShareIntent {
            attestation_doc_b64: fixture.attestation_document_cose_sign1_base64.clone(),
            manifest_b64: BASE64_STANDARD.encode(&deployment_manifest_bytes),
            operator_encrypt_key: operator.encrypt_public_key().to_string(),
            operator_sign_key: operator.sign_public_key().to_string(),
            deployment_id: DEPLOYMENT_ID.to_string(),
            app_quorum_key: hex::encode(&manifest.namespace().quorum_key),
        };

        let intent = build_re_encrypt_intent(
            details,
            deployment_manifest_bytes,
            &operator,
            false,
            Some(fixture.validation_time_secs),
        )
        .unwrap();

        assert_eq!(intent, expected);
        assert_ne!(
            BASE64_STANDARD.decode(&intent.manifest_b64).unwrap(),
            fixture.manifest_envelope.to_storage_vec().unwrap()
        );
    }

    #[test]
    fn verification_failure_prevents_intent_construction() {
        let mut fixture = fixture();
        match &mut fixture.manifest_envelope {
            VersionedManifestEnvelope::V2(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V1(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V0(envelope) => envelope.manifest_set_approvals.clear(),
        }
        let details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let operator = resolved_member(&fixture);

        let error = build_re_encrypt_intent(
            details,
            deployment_manifest_bytes(fixture.manifest_envelope.clone()),
            &operator,
            false,
            Some(fixture.validation_time_secs),
        )
        .unwrap_err();

        assert_eq!(error.to_string(), "failed to verify manifest approvals");
    }

    #[test]
    fn invalid_attestation_prevents_intent_construction() {
        let fixture = fixture();
        let valid_details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let (deployment_id, mut attestation_document, manifest_envelope, fetched_at_unix_ms) =
            valid_details.into_parts();
        *attestation_document.last_mut().unwrap() ^= 0xff;
        let details = FetchedProvisioningDetails::new(
            deployment_id,
            attestation_document,
            manifest_envelope,
            fetched_at_unix_ms,
        );
        let operator = resolved_member(&fixture);

        let error = build_re_encrypt_intent(
            details,
            deployment_manifest_bytes(fixture.manifest_envelope.clone()),
            &operator,
            false,
            Some(fixture.validation_time_secs),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "failed to parse and verify attestation document"
        );
    }

    #[test]
    fn dangerous_skip_allows_unapproved_manifest() {
        let mut fixture = fixture();
        match &mut fixture.manifest_envelope {
            VersionedManifestEnvelope::V2(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V1(envelope) => envelope.manifest_set_approvals.clear(),
            VersionedManifestEnvelope::V0(envelope) => envelope.manifest_set_approvals.clear(),
        }
        let details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let operator = resolved_member(&fixture);

        build_re_encrypt_intent(
            details,
            deployment_manifest_bytes(fixture.manifest_envelope.clone()),
            &operator,
            true,
            None,
        )
        .unwrap();
    }

    #[test]
    fn rejects_operator_outside_manifest_share_set() {
        let fixture = fixture();
        let details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let resolved = resolved_member(&fixture);
        let (deployment_id, attestation_document, mut envelope, fetched_at_unix_ms) =
            details.into_parts();
        match &mut envelope {
            VersionedManifestEnvelope::V2(value) => value.manifest.share_set.members.clear(),
            VersionedManifestEnvelope::V1(value) => value.manifest.share_set.members.clear(),
            VersionedManifestEnvelope::V0(value) => value.manifest.share_set.members.clear(),
        }
        let deployment_manifest_bytes = deployment_manifest_bytes(envelope.clone());
        let details = FetchedProvisioningDetails::new(
            deployment_id,
            attestation_document,
            envelope,
            fetched_at_unix_ms,
        );

        let error =
            build_re_encrypt_intent(details, deployment_manifest_bytes, &resolved, true, None)
                .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "hosted operator 'hosted-operator' ({OPERATOR_ID}) is not part of the manifest share set"
            )
        );
    }

    #[test]
    fn rejects_deployment_manifest_that_differs_from_envelope() {
        let fixture = fixture();
        let details = details(
            &fixture.attestation_document_cose_sign1_base64,
            fixture.manifest_envelope.clone(),
        );
        let operator = resolved_member(&fixture);
        let mut different_manifest = fixture.manifest_envelope.clone().manifest();
        match &mut different_manifest {
            VersionedManifest::V2(manifest) => manifest.namespace.nonce += 1,
            VersionedManifest::V1(manifest) => manifest.namespace.nonce += 1,
            VersionedManifest::V0(manifest) => manifest.namespace.nonce += 1,
        }
        let different_manifest = different_manifest.to_storage_vec().unwrap();

        let error = build_re_encrypt_intent(details, different_manifest, &operator, true, None)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "deployment manifest does not match provisioning manifest envelope"
        );
    }

    #[test]
    fn validates_and_serializes_result() {
        let output = validate_result(ReEncryptTvcQuorumKeyShareResult {
            provisioning_share_id: "provisioning-share-id".to_string(),
        })
        .unwrap();
        let expected = ProvisioningShareCreated {
            provisioning_share_id: "provisioning-share-id".to_string(),
        };

        assert_eq!(output, expected);
        assert_eq!(
            output.to_string(),
            "Provisioning Share ID: provisioning-share-id"
        );

        let value = serde_json::to_value(Outcome::ProvisioningShareCreated(output)).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "reason": "provisioning_share_created",
                "provisioningShareId": "provisioning-share-id",
            })
        );
    }

    #[test]
    fn rejects_empty_provisioning_share_id() {
        let error = validate_result(ReEncryptTvcQuorumKeyShareResult {
            provisioning_share_id: " ".to_string(),
        })
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "re-encrypt TVC quorum-key share response contained an empty provisioning share ID"
        );
    }
}
