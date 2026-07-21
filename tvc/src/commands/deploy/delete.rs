//! Deploy delete command - marks a deployment for deletion.

use crate::client::build_client;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{ActivityStatus, DeleteTvcDeploymentIntent};

/// Delete a TVC deployment by marking it for deletion.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to delete.
    #[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy delete command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
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

    Ok(Outcome::DeployDelete(DeploymentDeleted {
        deployment_id: result.result.deployment_id,
        activity_id: result.activity_id,
        activity_status: result.status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentDeleted {
    deployment_id: String,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for DeploymentDeleted {
    fn default() -> Self {
        Self {
            deployment_id: String::default(),
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Display for DeploymentDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
Deployment delete accepted; deployment is marked for deletion.

Deployment ID: {}
Activity ID: {}
Activity Status: {}"#,
            self.deployment_id,
            self.activity_id,
            self.activity_status.as_str_name()
        )
    }
}
