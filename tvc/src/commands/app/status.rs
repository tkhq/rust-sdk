//! App status command.

use anyhow::{Context, anyhow};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::Write as _;
use turnkey_client::generated::GetAppStatusRequest;
use turnkey_client::generated::external::data::v1::{AppStatus, DeploymentStatus};

use crate::client::fetch_tvc_app;
use crate::commands::app_status::{
    ReplicaCounts, TimestampPayload, format_replica_counts, sanitize_app_status,
};
use crate::commands::display::format_egress_enabled;
use crate::outcome::Outcome;
use crate::output::{Message, StdCtx};

/// Get the live status of an app from the cluster.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the app.
    #[arg(short, long, env = "TVC_APP_ID")]
    pub app_id: String,
}

/// Run the app status command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
    let auth = crate::client::build_client().await?;

    let request = GetAppStatusRequest {
        organization_id: auth.org_id.clone(),
        app_id: args.app_id.clone(),
    };

    let response = auth
        .client
        .get_app_status(request)
        .await
        .context("failed to fetch app status")?;

    let app_status = sanitize_app_status(
        response
            .app_status
            .ok_or_else(|| anyhow!("no status returned for app: {}", args.app_id))?,
    );
    let app = fetch_tvc_app(&auth, &args.app_id).await?;

    // Exhaustive destructure so a new `AppStatus` field forces a decision here
    // rather than being silently dropped.
    let AppStatus {
        app_id,
        deployments,
        targeted_deployment_id,
    } = app_status;

    Ok(Outcome::AppStatus(AppStatusReport {
        app_id,
        targeted_deployment_id,
        egress_enabled: app.enable_egress,
        deployments: deployments
            .into_iter()
            .map(|deployment| {
                let DeploymentStatus {
                    deployment_id,
                    ready_replicas,
                    desired_replicas,
                    last_updated_time,
                } = deployment;
                DeploymentReplicaStatus {
                    deployment_id,
                    replicas: ReplicaCounts {
                        ready: ready_replicas,
                        desired: desired_replicas,
                    },
                    last_updated: last_updated_time.map(Into::into),
                }
            })
            .collect(),
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusReport {
    app_id: String,
    targeted_deployment_id: String,
    egress_enabled: bool,
    deployments: Vec<DeploymentReplicaStatus>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeploymentReplicaStatus {
    deployment_id: String,
    replicas: ReplicaCounts,
    last_updated: Option<TimestampPayload>,
}

impl Message for AppStatusReport {
    fn reason(&self) -> &'static str {
        "app-status"
    }

    fn human_message(&self) -> String {
        let mut message = format!(
            r#"App ID: {}
Targeted Deployment: {}
{}"#,
            self.app_id,
            self.targeted_deployment_id,
            format_egress_enabled(self.egress_enabled)
        );

        if self.deployments.is_empty() {
            message.push_str("\n\nNo deployments found.");
        } else {
            for deployment in &self.deployments {
                let _ = write!(
                    message,
                    r#"

Deployment: {}
  {}"#,
                    deployment.deployment_id,
                    format_replica_counts(deployment.replicas.ready, deployment.replicas.desired)
                );

                if let Some(updated) = &deployment.last_updated {
                    let _ = write!(
                        message,
                        "\n  Last Updated: {}.{:0>9}s",
                        updated.seconds, updated.nanos
                    );
                }
            }
        }

        message
    }
}
