//! App set-live-deploy command.

use crate::client::build_client;
use crate::output::StdCtx;
use crate::shell_println;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::UpdateTvcAppLiveDeploymentIntent;

/// Set the live deployment for a TVC app.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to set live.
    #[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the app set-live-deploy command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<()> {
    let auth = build_client().await?;

    let intent = UpdateTvcAppLiveDeploymentIntent {
        deployment_id: args.deploy_id.clone(),
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .update_tvc_app_live_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to set TVC app live deployment")?;

    shell_println!(ctx)?;
    shell_println!(ctx, "Set-live-deploy accepted.")?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Deployment ID: {}", args.deploy_id)?;
    shell_println!(ctx, "Activity ID: {}", result.activity_id)?;
    shell_println!(ctx, "Activity Status: {:?}", result.status)?;

    Ok(())
}
