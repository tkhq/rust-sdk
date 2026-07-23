//! Deploy status command.

use anyhow::Context;
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::VersionedManifest;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use turnkey_client::generated::GetTvcDeploymentRequest;
use turnkey_client::generated::external::data::v1::TvcDeployment;

use crate::approvals::{ApprovalValidation, validate_deployment_approvals};
use crate::client::fetch_tvc_app;
use crate::commands::app_status::TimestampPayload;
use crate::commands::display::{format_egress_enabled, yes_no};
use crate::outcome::Outcome;
use crate::output::StdCtx;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy status command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
    let auth = crate::client::build_client().await?;

    let request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id.clone(),
    };

    let response = auth
        .client
        .get_tvc_deployment(request)
        .await
        .context("failed to fetch deployment")?;

    let deployment = response
        .tvc_deployment
        .ok_or_else(|| anyhow::anyhow!("deployment not found: {}", args.deploy_id))?;

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

    let manifest = manifest.ok_or_else(|| anyhow::anyhow!("manifest not found in deployment"))?;
    let app = fetch_tvc_app(&auth, &app_id).await?;

    let manifest_approvals = match VersionedManifest::try_from_slice_compat(&manifest.manifest) {
        Ok(parsed) => Some(validate_deployment_approvals(&parsed, &manifest_approvals)),
        Err(error) => {
            ctx.shell().human().warn(format!(
                "failed to parse manifest; cannot validate approvals: {error}"
            ))?;
            None
        }
    };

    Ok(Outcome::DeployStatus(DeploymentStatusReport {
        deployment_id: id,
        app_id,
        egress_enabled: app.enable_egress,
        manifest_id: manifest.id,
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
    manifest_approvals: Option<ApprovalValidation>,
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

        if let Some(validation) = &self.manifest_approvals {
            write!(
                f,
                "\n\nManifest Approvals: {}/{} valid",
                validation.valid_count, validation.threshold
            )?;
            for approval in &validation.approvals {
                write!(f, "\n  {}: {}", approval.operator_label(), approval.verdict)?;
            }
            write!(
                f,
                "\nQuorum reached: {}",
                yes_no(validation.quorum_reached())
            )?;
        }

        Ok(())
    }
}

fn format_marked_for_deletion(delete: bool) -> String {
    format!("Marked for deletion: {}", yes_no(delete))
}

#[cfg(test)]
mod tests {
    use super::{DeploymentStatusReport, format_marked_for_deletion};
    use crate::approvals::{ApprovalValidation, ApprovalVerdict, ValidatedApproval};

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
            manifest_approvals: Some(ApprovalValidation {
                approvals: vec![
                    ValidatedApproval {
                        operator_id: "op-1".to_string(),
                        operator_name: "operator-alice".to_string(),
                        verdict: ApprovalVerdict::Valid,
                    },
                    ValidatedApproval {
                        operator_id: "op-2".to_string(),
                        operator_name: "operator-bob".to_string(),
                        verdict: ApprovalVerdict::InvalidSignature,
                    },
                ],
                valid_count: 1,
                threshold: 2,
            }),
            ..DeploymentStatusReport::default()
        };

        assert!(report.to_string().ends_with(
            "Manifest Approvals: 1/2 valid\n\
             \x20 operator-alice (op-1): valid\n\
             \x20 operator-bob (op-2): invalid signature\n\
             Quorum reached: no"
        ));
    }
}
