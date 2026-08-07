//! Deploy status command.

use anyhow::Context;
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::VersionedManifest;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use tracing::debug;
use turnkey_client::generated::GetTvcDeploymentRequest;
use turnkey_client::generated::external::data::v1::{TvcDeployment, TvcManifest};
use uuid::Uuid;

use crate::approvals::{ApprovalValidationWithMeta, OperatorApproval, ValidatedManifest};
use crate::client::fetch_tvc_app;
use crate::commands::app_status::TimestampPayload;
use crate::commands::display::{OrUnknown, format_egress_enabled, yes_no};
use crate::errors::MissingResource;
use crate::outcome::Outcome;
use tracing::instrument;

use crate::output::StdCtx;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: Uuid,
}

/// Run the deploy status command.
#[instrument(skip_all)]
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
    let auth = crate::client::build_client().await?;
    let deploy_id = args.deploy_id.to_string();

    let request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.to_string(),
        deployment_id: deploy_id.clone(),
    };

    let response = auth
        .client
        .get_tvc_deployment(request)
        .await
        .with_context(|| format!("failed to fetch deployment {deploy_id}"))?;

    let deployment = response
        .tvc_deployment
        .ok_or_else(|| MissingResource::new("deployment", args.deploy_id))?;

    // Exhaustive destructure (rather than `..`) so a new `TvcDeployment` field
    // forces a compile error here and forces a deliberate decision about usage
    let TvcDeployment {
        id,
        app_id,
        manifest,
        qos_version,
        pivot_container,
        created_at,
        updated_at,
        delete,
        debug_mode,
        organization_id: _,
        manifest_set: _,
        share_set: _,
        manifest_approvals,
    } = deployment;

    let TvcManifest {
        id: manifest_id,
        manifest: manifest_bytes,
        created_at: _,
        updated_at: _,
    } = manifest.ok_or_else(|| MissingResource::new("manifest", format!("deployment {id}")))?;

    let app = fetch_tvc_app(&auth, &app_id).await?;

    let manifest_approvals = manifest_approvals
        .into_iter()
        .map(OperatorApproval::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    let manifest = VersionedManifest::try_from_slice_compat(&manifest_bytes)
        .inspect_err(|error| {
            let write_result = ctx.shell().human().warn(format!(
                "failed to parse manifest; cannot validate approvals: {error}"
            ));

            if let Err(write_error) = write_result {
                debug!(%write_error, "failed to write manifest-parse warning");
            }
        })
        .ok();

    let manifest_approvals = manifest
        .as_ref()
        .map(ValidatedManifest::try_from)
        .transpose()?
        .map(|manifest| manifest.validate(manifest_approvals).with_meta());

    Ok(Outcome::DeploymentStatus(DeploymentStatusReport {
        deployment_id: id,
        app_id,
        egress_enabled: app.enable_egress,
        manifest_id,
        qos_version,
        marked_for_deletion: delete,
        debug_mode,
        pivot_container: pivot_container.map(|pivot| PivotContainerSummary {
            url: pivot.container_url,
            path: pivot.path,
            args: pivot.args,
        }),
        created_at: created_at.map(Into::into),
        updated_at: updated_at.map(Into::into),
        manifest_approvals,
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentStatusReport {
    deployment_id: String,
    app_id: String,
    egress_enabled: bool,
    manifest_id: String,
    qos_version: String,
    marked_for_deletion: bool,
    debug_mode: bool,
    pivot_container: Option<PivotContainerSummary>,
    created_at: Option<TimestampPayload>,
    updated_at: Option<TimestampPayload>,
    /// Cryptographic validation of the posted approvals against the manifest
    /// set; `None` when the manifest bytes could not be parsed.
    manifest_approvals: Option<ApprovalValidationWithMeta>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PivotContainerSummary {
    url: String,
    path: String,
    args: Vec<String>,
}

impl Display for DeploymentStatusReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Deployment: {}
App ID: {}
{}
Manifest ID: {}
QOS Version: {}
{}
Debug Mode: {}"#,
            self.deployment_id,
            self.app_id,
            format_egress_enabled(self.egress_enabled),
            self.manifest_id,
            self.qos_version,
            format_marked_for_deletion(self.marked_for_deletion),
            yes_no(self.debug_mode)
        )?;

        if let Some(pivot) = &self.pivot_container {
            write!(
                f,
                r#"

Pivot Container:
  URL: {}
  Path: {}"#,
                pivot.url, pivot.path
            )?;
            if !pivot.args.is_empty() {
                write!(f, "\n  Args: {:?}", pivot.args)?;
            }
        }

        if let Some(created) = &self.created_at {
            write!(f, "\n\nCreated: {}.{:09}s", created.seconds, created.nanos)?;
        }

        if let Some(updated) = &self.updated_at {
            write!(f, "\nUpdated: {}.{:09}s", updated.seconds, updated.nanos)?;
        }

        let (approvals, threshold, meta) =
            if let Some(validation_with_meta) = &self.manifest_approvals {
                (
                    validation_with_meta.validation.approvals.as_slice(),
                    validation_with_meta.validation.threshold.into(),
                    (&validation_with_meta.meta).into(),
                )
            } else {
                ([].as_slice(), None, None)
            };

        let threshold = OrUnknown(threshold);
        let valid_count = OrUnknown(meta.map(|meta| meta.valid_count));
        let quorum_reached = OrUnknown(meta.map(|meta| yes_no(meta.quorum_reached)));

        write!(f, "\n\nManifest Approvals: {valid_count}/{threshold} valid")?;

        for validated in approvals {
            write!(f, "\n  {}: {}", validated.approval, validated.verdict)?;
        }

        write!(f, "\nQuorum reached: {quorum_reached}")?;

        Ok(())
    }
}

fn format_marked_for_deletion(delete: bool) -> String {
    format!("Marked for deletion: {}", yes_no(delete))
}

#[cfg(test)]
mod tests {
    use super::{DeploymentStatusReport, format_marked_for_deletion};
    use crate::approvals::{ApprovalValidation, ApprovalVerdict, test_uuid, validated_approval};

    #[test]
    fn marked_for_deletion_formats_yes_when_delete_is_true() {
        assert_eq!(format_marked_for_deletion(true), "Marked for deletion: yes");
    }

    #[test]
    fn marked_for_deletion_formats_no_when_delete_is_false() {
        assert_eq!(format_marked_for_deletion(false), "Marked for deletion: no");
    }

    #[test]
    fn status_report_renders_mixed_approval_verdicts() {
        let report = DeploymentStatusReport {
            manifest_approvals: Some(
                ApprovalValidation {
                    approvals: vec![
                        validated_approval("op-1", "operator-alice", ApprovalVerdict::Valid),
                        validated_approval(
                            "op-2",
                            "operator-bob",
                            ApprovalVerdict::InvalidSignature,
                        ),
                    ],
                    threshold: 2,
                }
                .with_meta(),
            ),
            ..DeploymentStatusReport::default()
        };

        let expected = format!(
            r#"Manifest Approvals: 1/2 valid
  operator-alice ({}): valid
  operator-bob ({}): invalid signature
Quorum reached: no"#,
            test_uuid("op-1"),
            test_uuid("op-2"),
        );

        assert!(report.to_string().ends_with(&expected));
    }
}
