//! Deploy status command.

use anyhow::Context;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::Write as _;
use turnkey_client::generated::GetTvcDeploymentRequest;
use turnkey_client::generated::external::data::v1::TvcDeployment;

use crate::client::fetch_tvc_app;
use crate::commands::app_status::TimestampPayload;
use crate::commands::display::{format_egress_enabled, yes_no};
use crate::outcome::Outcome;
use crate::output::{Message, StdCtx};

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy status command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
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
        manifest_approvals: _,
    } = deployment;

    let manifest = manifest.ok_or_else(|| anyhow::anyhow!("manifest not found in deployment"))?;
    let app = fetch_tvc_app(&auth, &app_id).await?;

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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PivotContainerSummary {
    url: String,
    path: String,
    args: Vec<String>,
}

impl Message for DeploymentStatusReport {
    fn reason(&self) -> &'static str {
        "deployment-status"
    }

    fn human_message(&self) -> String {
        let mut message = format!(
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
        );

        if let Some(pivot) = &self.pivot_container {
            let _ = write!(
                message,
                r#"

Pivot Container:
  URL: {}
  Path: {}"#,
                pivot.url, pivot.path
            );
            if !pivot.args.is_empty() {
                let _ = write!(message, "\n  Args: {:?}", pivot.args);
            }
        }

        if let Some(created) = &self.created_at {
            let _ = write!(
                message,
                "\n\nCreated: {}.{:09}s",
                created.seconds, created.nanos
            );
        }

        if let Some(updated) = &self.updated_at {
            let _ = write!(
                message,
                "\nUpdated: {}.{:09}s",
                updated.seconds, updated.nanos
            );
        }

        message
    }
}

fn format_marked_for_deletion(delete: bool) -> String {
    format!("Marked for deletion: {}", yes_no(delete))
}

#[cfg(test)]
mod tests {
    use super::format_marked_for_deletion;

    #[test]
    fn marked_for_deletion_formats_yes_when_delete_is_true() {
        assert_eq!(format_marked_for_deletion(true), "Marked for deletion: yes");
    }

    #[test]
    fn marked_for_deletion_formats_no_when_delete_is_false() {
        assert_eq!(format_marked_for_deletion(false), "Marked for deletion: no");
    }
}
