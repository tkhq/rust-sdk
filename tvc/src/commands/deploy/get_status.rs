//! Deploy get-status command.

use anyhow::{anyhow, Context};
use clap::Args as ClapArgs;
use turnkey_client::generated::external::data::v1::{AppStatus, DeploymentStatus};
use turnkey_client::generated::{GetAppStatusRequest, GetTvcDeploymentRequest};

/// Get the live status of a deployment from the app status API.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy get-status command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let deployment_request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id.clone(),
    };

    let deployment_response = auth
        .client
        .get_tvc_deployment(deployment_request)
        .await
        .context("failed to fetch deployment")?;

    let deployment = deployment_response
        .tvc_deployment
        .ok_or_else(|| anyhow!("deployment not found: {}", args.deploy_id))?;

    let app_request = GetAppStatusRequest {
        organization_id: auth.org_id.clone(),
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

    println!("Deployment: {}", deployment.id);
    println!("App ID: {}", app_status.app_id);
    println!(
        "Is Targeted Deployment: {}",
        if app_status.targeted_deployment_id == args.deploy_id {
            "yes"
        } else {
            "no"
        }
    );
    if let Some(deployment_status) = find_deployment_status(&app_status, &args.deploy_id) {
        println!(
            "{}",
            crate::commands::app_status::format_replica_status(deployment_status)
        );

        if let Some(updated) = &deployment_status.last_updated_time {
            println!("Last Updated: {}.{:09}s", updated.seconds, updated.nanos);
        }
    } else {
        println!("Live Status: unavailable");
        println!("Reason: deployment not present in current app status");
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
