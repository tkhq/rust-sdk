//! Deploy restore command.

use anyhow::Context;
use clap::Args as ClapArgs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::RestoreTvcDeploymentIntent;

use crate::output::Ctx;
use crate::shell_println;

/// Restore a deleted deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy restore command.
pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let intent = RestoreTvcDeploymentIntent {
        deployment_id: args.deploy_id,
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .restore_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to restore TVC deployment")?;

    shell_println!(ctx)?;
    shell_println!(
        ctx,
        "Deployment restore accepted; deployment is no longer marked for deletion."
    )?;
    shell_println!(ctx)?;
    shell_println!(ctx, "Deployment ID: {}", result.result.deployment_id)?;
    shell_println!(ctx, "Activity ID: {}", result.activity_id)?;
    shell_println!(ctx, "Activity Status: {:?}", result.status)?;

    Ok(())
}
