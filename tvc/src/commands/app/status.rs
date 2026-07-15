//! App status command.

use anyhow::{Context, anyhow};
use clap::Args as ClapArgs;
use turnkey_client::generated::GetAppStatusRequest;

use crate::client::fetch_tvc_app;
use crate::commands::app_status::{format_replica_status, sanitize_app_status};
use crate::commands::display::format_egress_enabled;
use crate::output::StdCtx;
use crate::shell_println;

/// Get the live status of an app from the cluster.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the app.
    #[arg(short, long, env = "TVC_APP_ID")]
    pub app_id: String,
}

/// Run the app status command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<()> {
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

    shell_println!(ctx, "App ID: {}", app_status.app_id)?;
    shell_println!(
        ctx,
        "Targeted Deployment: {}",
        app_status.targeted_deployment_id
    )?;
    shell_println!(ctx, "{}", format_egress_enabled(app.enable_egress))?;

    if app_status.deployments.is_empty() {
        shell_println!(ctx)?;
        shell_println!(ctx, "No deployments found.")?;
    } else {
        for deployment in &app_status.deployments {
            shell_println!(ctx)?;
            shell_println!(ctx, "Deployment: {}", deployment.deployment_id)?;
            shell_println!(ctx, "  {}", format_replica_status(deployment))?;

            if let Some(updated) = &deployment.last_updated_time {
                shell_println!(
                    ctx,
                    "  Last Updated: {}.{:0>9}s",
                    updated.seconds,
                    updated.nanos
                )?;
            }
        }
    }

    Ok(())
}
