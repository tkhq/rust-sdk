//! App status command.

use anyhow::{anyhow, Context};
use clap::Args as ClapArgs;
use turnkey_client::generated::GetAppStatusRequest;

/// Get the live status of an app from the cluster.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the app.
    #[arg(short, long, env = "TVC_APP_ID")]
    pub app_id: String,
}

/// Run the app status command.
pub async fn run(args: Args) -> anyhow::Result<()> {
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

    let app_status = crate::commands::app_status::sanitize_app_status(
        response
            .app_status
            .ok_or_else(|| anyhow!("no status returned for app: {}", args.app_id))?,
    );

    println!("App ID: {}", app_status.app_id);
    println!("Targeted Deployment: {}", app_status.targeted_deployment_id);

    if app_status.deployments.is_empty() {
        println!();
        println!("No deployments found.");
    } else {
        for deployment in &app_status.deployments {
            println!();
            println!("Deployment: {}", deployment.deployment_id);
            println!(
                "  {}",
                crate::commands::app_status::format_replica_status(deployment)
            );

            if let Some(updated) = &deployment.last_updated_time {
                println!("  Last Updated: {}.{:0>9}s", updated.seconds, updated.nanos);
            }
        }
    }

    Ok(())
}
