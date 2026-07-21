//! App set-live-deploy command.

use crate::client::build_client;
use crate::outcome::Outcome;
use crate::output::StdCtx;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{ActivityStatus, UpdateTvcAppLiveDeploymentIntent};

/// Set the live deployment for a TVC app.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to set live.
    #[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the app set-live-deploy command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
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

    Ok(Outcome::AppSetLiveDeploy(LiveDeploymentSet {
        deployment_id: args.deploy_id,
        activity_id: result.activity_id,
        activity_status: result.status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDeploymentSet {
    deployment_id: String,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for LiveDeploymentSet {
    fn default() -> Self {
        Self {
            deployment_id: String::default(),
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Display for LiveDeploymentSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"
Set-live-deploy accepted.

Deployment ID: {}
Activity ID: {}
Activity Status: {}"#,
            self.deployment_id,
            self.activity_id,
            self.activity_status.as_str_name()
        )
    }
}
