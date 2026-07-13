//! Deploy get-status command.

use anyhow::{Context, anyhow};
use clap::Args as ClapArgs;
use std::io::Write;
use turnkey_client::generated::{
    GetAppStatusRequest,
    external::data::v1::{AppStatus, DeploymentStatus},
};

use crate::{
    client::{fetch_tvc_app, fetch_tvc_deployment},
    commands::display::format_egress_enabled,
    output::Ctx,
    shell_println,
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
pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> anyhow::Result<()> {
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

    shell_println!(ctx, "Deployment: {}", deployment.id)?;
    shell_println!(ctx, "App ID: {}", app_status.app_id)?;
    shell_println!(ctx, "{}", format_egress_enabled(app.enable_egress))?;
    shell_println!(
        ctx,
        "Is Targeted Deployment: {}",
        if app_status.targeted_deployment_id == deployment_id {
            "yes"
        } else {
            "no"
        }
    )?;
    if let Some(deployment_status) = find_deployment_status(&app_status, &deployment_id) {
        shell_println!(
            ctx,
            "{}",
            crate::commands::app_status::format_replica_status(deployment_status)
        )?;

        if let Some(updated) = &deployment_status.last_updated_time {
            shell_println!(
                ctx,
                "Last Updated: {}.{:0>9}s",
                updated.seconds,
                updated.nanos
            )?;
        }
    } else {
        shell_println!(ctx, "Live Status: unavailable")?;
        shell_println!(ctx, "Reason: deployment not present in current app status")?;
    }

    Ok(())
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
