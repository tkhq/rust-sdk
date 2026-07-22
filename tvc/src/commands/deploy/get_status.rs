//! Deploy get-status command.

use anyhow::{Context, anyhow};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use turnkey_client::generated::{
    GetAppStatusRequest,
    external::data::v1::{AppStatus, DeploymentStatus},
};

use crate::{
    client::{fetch_tvc_app, fetch_tvc_deployment},
    commands::app_status::{ReplicaCounts, TimestampPayload, format_replica_counts},
    commands::display::format_egress_enabled,
    outcome::Outcome,
    output::StdCtx,
};

/// Get the live status of a deployment from the app status API.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy get-status command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> anyhow::Result<Outcome> {
    let Args {
        deploy_id: deployment_id,
    } = args;

    // TODO (TVC-154):
    // this is a little backwards, all the variables that are needed to build the client
    // are resolved INSIDE the `build-client`. Instead, all the necessary data for the client
    // should be resolved, then passed into an infallible constructor
    let auth = crate::client::build_client().await?;
    let org_id = auth.org_id.clone();

    let deployment = fetch_tvc_deployment(&auth, org_id.clone(), deployment_id.clone()).await?;

    let app_request = GetAppStatusRequest {
        organization_id: org_id.clone(),
        app_id: deployment.app_id.clone(),
    };

    let app_response = auth
        .client
        .get_app_status(app_request)
        .await
        .context("failed to fetch app status")?;

    let app_status = crate::commands::app_status::sanitize_app_status(
        app_response
            .app_status
            .ok_or_else(|| anyhow!("no status returned for app: {}", deployment.app_id))?,
    );
    let app = fetch_tvc_app(&auth, &deployment.app_id).await?;

    let deployment_status = find_deployment_status(&app_status, &deployment_id);

    Ok(Outcome::DeployGetStatus(DeploymentRuntimeStatus {
        deployment_id: deployment.id,
        app_id: app_status.app_id.clone(),
        egress_enabled: app.enable_egress,
        is_targeted: app_status.targeted_deployment_id == deployment_id,
        replicas: deployment_status.map(|status| ReplicaCounts {
            ready: status.ready_replicas,
            desired: status.desired_replicas,
        }),
        last_updated: deployment_status
            .and_then(|status| status.last_updated_time.clone().map(Into::into)),
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentRuntimeStatus {
    deployment_id: String,
    app_id: String,
    egress_enabled: bool,
    is_targeted: bool,
    /// `None` when the deployment is not present in the current app status.
    replicas: Option<ReplicaCounts>,
    last_updated: Option<TimestampPayload>,
}

impl Display for DeploymentRuntimeStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Deployment: {}
App ID: {}
{}
Is Targeted Deployment: {}"#,
            self.deployment_id,
            self.app_id,
            format_egress_enabled(self.egress_enabled),
            if self.is_targeted { "yes" } else { "no" }
        )?;

        match &self.replicas {
            Some(replicas) => {
                write!(
                    f,
                    "\n{}",
                    format_replica_counts(replicas.ready, replicas.desired)
                )?;

                if let Some(updated) = &self.last_updated {
                    write!(
                        f,
                        "\nLast Updated: {}.{:09}s",
                        updated.seconds, updated.nanos
                    )?;
                }
            }
            None => f.write_str(
                r#"
Live Status: unavailable
Reason: deployment not present in current app status"#,
            )?,
        }

        Ok(())
    }
}

fn find_deployment_status<'a>(
    app_status: &'a AppStatus,
    deploy_id: &str,
) -> Option<&'a DeploymentStatus> {
    app_status
        .deployments
        .iter()
        .find(|status| status.deployment_id == deploy_id)
}

#[cfg(test)]
mod tests {
    use super::find_deployment_status;
    use turnkey_client::generated::external::data::v1::{AppStatus, DeploymentStatus};

    #[test]
    fn find_deployment_status_matches_sanitized_ids() {
        let app_status = AppStatus {
            app_id: "app-123".to_string(),
            deployments: vec![DeploymentStatus {
                deployment_id: "5376f492-d014-4e01-a6bb-20fc97448e25".to_string(),
                ready_replicas: 3,
                desired_replicas: 3,
                last_updated_time: None,
            }],
            targeted_deployment_id: "5376f492-d014-4e01-a6bb-20fc97448e25".to_string(),
        };

        let deployment_status =
            find_deployment_status(&app_status, "5376f492-d014-4e01-a6bb-20fc97448e25");

        assert!(deployment_status.is_some());
    }
}
