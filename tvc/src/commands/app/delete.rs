//! App delete command - marks an app and all deployments for deletion.

use crate::client::build_client;
use crate::output::Shell;
use crate::shell_line;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::DeleteTvcAppAndDeploymentsIntent;

/// Delete a TVC application and all of its deployments.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the app to delete.
    #[arg(long, value_name = "APP_ID", env = "TVC_APP_ID")]
    pub app_id: String,
}

/// Run the app delete command.
pub async fn run<O: Write, E: Write>(args: Args, shell: &mut Shell<O, E>) -> Result<()> {
    let auth = build_client().await?;

    let intent = DeleteTvcAppAndDeploymentsIntent {
        app_id: args.app_id.clone(),
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .delete_tvc_app_and_deployments(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to delete TVC app and deployments")?;

    shell_line!(shell, "App delete accepted.")?;
    shell_line!(shell, "App and deployments marked for deletion.")?;
    shell_line!(shell, "App ID: {}", result.result.app_id)?;
    shell_line!(shell, "Activity ID: {}", result.activity_id)?;
    shell_line!(shell, "Activity Status: {:?}", result.status)?;

    Ok(())
}
