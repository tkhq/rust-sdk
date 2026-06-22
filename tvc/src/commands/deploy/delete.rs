//! Deploy delete command - marks a deployment for deletion.

use crate::client::build_client;
use crate::output::Shell;
use crate::shell_line;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::DeleteTvcDeploymentIntent;

/// Delete a TVC deployment by marking it for deletion.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to delete.
    #[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy delete command.
pub async fn run<O: Write, E: Write>(args: Args, shell: &mut Shell<O, E>) -> Result<()> {
    let auth = build_client().await?;

    let intent = DeleteTvcDeploymentIntent {
        deployment_id: args.deploy_id.clone(),
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .delete_tvc_deployment(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to delete TVC deployment")?;

    shell_line!(shell)?;
    shell_line!(
        shell,
        "Deployment delete accepted; deployment is marked for deletion."
    )?;
    shell_line!(shell)?;
    shell_line!(shell, "Deployment ID: {}", result.result.deployment_id)?;
    shell_line!(shell, "Activity ID: {}", result.activity_id)?;
    shell_line!(shell, "Activity Status: {:?}", result.status)?;

    Ok(())
}
