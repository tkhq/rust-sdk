//! App set-live-deploy command.

use crate::client::build_client;
use crate::commands::confirmation::confirm_yes_no;
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

    /// DANGEROUS: skip the confirmation prompt.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_CONFIRMATION")]
    pub dangerous_skip_confirmation: bool,
}

/// Run the app set-live-deploy command.
pub async fn run(args: Args) -> Result<()> {
    if !args.dangerous_skip_confirmation {
        confirm_yes_no(&format!(
            "Set TVC app live deployment to '{}'?",
            args.deploy_id
        ))?;
    }

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

    println!();
    println!("Set-live-deploy accepted.");
    println!();
    println!("Deployment ID: {}", args.deploy_id);
    println!("Activity ID: {}", result.activity_id);
    println!("Activity Status: {:?}", result.status);

    Ok(())
}
