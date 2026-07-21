//! App delete command - marks an app and all deployments for deletion.

use crate::client::build_client;
use crate::outcome::Outcome;
use crate::output::{Message, StdCtx};
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{ActivityStatus, DeleteTvcAppAndDeploymentsIntent};

/// Delete a TVC application and all of its deployments.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the app to delete.
    #[arg(long, value_name = "APP_ID", env = "TVC_APP_ID")]
    pub app_id: String,
}

/// Run the app delete command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
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

    Ok(Outcome::AppDelete(AppDeleted {
        app_id: result.result.app_id,
        activity_id: result.activity_id,
        activity_status: result.status,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDeleted {
    app_id: String,
    activity_id: String,
    /// Stable proto status name, e.g. `ACTIVITY_STATUS_COMPLETED`.
    activity_status: ActivityStatus,
}

/// Manual because the generated `ActivityStatus` does not implement
/// `Default`; the zero value is the enum's `Unspecified` variant.
impl Default for AppDeleted {
    fn default() -> Self {
        Self {
            app_id: String::default(),
            activity_id: String::default(),
            activity_status: ActivityStatus::Unspecified,
        }
    }
}

impl Message for AppDeleted {
    fn reason(&self) -> &'static str {
        "app-deleted"
    }

    fn human_message(&self) -> String {
        format!(
            r#"App delete accepted.
App and deployments marked for deletion.

App ID: {}
Activity ID: {}
Activity Status: {}"#,
            self.app_id,
            self.activity_id,
            self.activity_status.as_str_name()
        )
    }
}
