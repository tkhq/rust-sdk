//! Advisory warning for apps that have accumulated live deployments.
//!
//! TVC bills for what is deployed, not for what receives traffic, and the
//! deploy/test iteration loop makes it easy to leave old deployments running.
//! Nothing in the CLI used to say so, so callers — humans mid-iteration and
//! coding agents driving `tvc` non-interactively — had no signal to clean up
//! after themselves.

use crate::client::AuthenticatedClient;
use crate::output::StdCtx;
use anyhow::Result;
use tracing::debug;
use turnkey_client::generated::GetTvcAppDeploymentsRequest;
use turnkey_client::generated::external::data::v1::TvcDeployment;

/// A blue/green rollout needs exactly two live deployments at once: the one
/// currently taking traffic and the one replacing it. Anything above two is
/// accumulation from the deploy/test iteration loop rather than a rollout, and
/// each extra deployment keeps costing money until it is deleted.
pub const MAX_EXPECTED_LIVE_DEPLOYMENTS: usize = 2;

/// IDs of the deployments that are still live, i.e. not marked for deletion.
///
/// `TvcDeployment::delete` is the API's only live/torn-down distinction, so it
/// is what decides whether a deployment is still being billed.
pub fn live_deployment_ids(deployments: &[TvcDeployment]) -> Vec<&str> {
    deployments
        .iter()
        .filter(|deployment| !deployment.delete)
        .map(|deployment| deployment.id.as_str())
        .collect()
}

/// The warning text for an app's live deployments, or `None` when the count is
/// within [`MAX_EXPECTED_LIVE_DEPLOYMENTS`].
///
/// Advisory only: callers emit this on stderr and never change their exit code
/// because of it.
pub fn live_deployment_warning(app_id: &str, live_deployment_ids: &[&str]) -> Option<String> {
    let count = live_deployment_ids.len();
    if count <= MAX_EXPECTED_LIVE_DEPLOYMENTS {
        return None;
    }

    Some(format!(
        r#"app {app_id} has {count} live deployments, more than the {MAX_EXPECTED_LIVE_DEPLOYMENTS} a blue/green rollout needs.
  Each live deployment is billed even when it receives no traffic.
  Live deployments: {ids}
  Delete the ones you no longer need: tvc deploy delete --deploy-id <DEPLOY_ID>"#,
        ids = live_deployment_ids.join(", ")
    ))
}

/// Warn if `live_deployment_ids` is above the threshold.
///
/// Use this when a command already has the app's live deployments in hand, so
/// no extra API round trip is needed.
pub fn warn_on_live_deployments(
    ctx: &mut StdCtx,
    app_id: &str,
    live_deployment_ids: &[&str],
) -> Result<()> {
    if let Some(warning) = live_deployment_warning(app_id, live_deployment_ids) {
        ctx.shell().warn(warning)?;
    }
    Ok(())
}

/// Fetch the app's deployments and warn if too many are live.
///
/// Best effort: this runs after the command's real work has already succeeded,
/// so a failure to fetch is logged and swallowed rather than surfaced. Only use
/// this when the caller does not already have the deployment list.
pub async fn warn_on_fetched_live_deployments(
    ctx: &mut StdCtx,
    auth: &AuthenticatedClient,
    app_id: &str,
) {
    let deployments = match auth
        .client
        .get_tvc_app_deployments(GetTvcAppDeploymentsRequest {
            organization_id: auth.org_id.clone(),
            app_id: app_id.to_string(),
        })
        .await
    {
        Ok(response) => response.tvc_deployments,
        Err(error) => {
            debug!(%app_id, %error, "skipping live deployment warning: failed to list deployments");
            return;
        }
    };

    if let Err(error) = warn_on_live_deployments(ctx, app_id, &live_deployment_ids(&deployments)) {
        debug!(%app_id, %error, "failed to write live deployment warning");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turnkey_client::generated::external::data::v1::TvcContainerSpec;
    use turnkey_client::generated::immutable::common::v1::TvcHealthCheckType;

    const APP_ID: &str = "11111111-1111-1111-1111-111111111111";

    fn make_deployment(id: &str, delete: bool) -> TvcDeployment {
        TvcDeployment {
            id: id.to_string(),
            organization_id: "test-org".to_string(),
            app_id: APP_ID.to_string(),
            manifest_set: None,
            share_set: None,
            manifest: None,
            manifest_approvals: Vec::new(),
            qos_version: "test-qos".to_string(),
            pivot_container: Some(TvcContainerSpec {
                container_url: "ghcr.io/team/app".to_string(),
                path: "/usr/bin/app".to_string(),
                args: Vec::new(),
                has_pull_secret: false,
                health_check_type: TvcHealthCheckType::Http,
                health_check_port: 3000,
                public_ingress_port: 3000,
            }),
            created_at: None,
            updated_at: None,
            delete,
            debug_mode: false,
        }
    }

    #[test]
    fn no_warning_without_deployments() {
        assert_eq!(live_deployment_warning(APP_ID, &[]), None);
    }

    #[test]
    fn no_warning_for_a_single_deployment() {
        assert_eq!(live_deployment_warning(APP_ID, &["dep-1"]), None);
    }

    /// Two live deployments is the steady state of a blue/green rollout, so the
    /// threshold must not fire on it.
    #[test]
    fn no_warning_at_the_threshold() {
        assert_eq!(live_deployment_warning(APP_ID, &["dep-1", "dep-2"]), None);
    }

    /// Golden text: the count, the cost, every live deployment ID, and the exact
    /// remediation command an agent has to run are all part of the interface.
    #[test]
    fn warns_just_above_the_threshold() {
        assert_eq!(
            live_deployment_warning(APP_ID, &["dep-1", "dep-2", "dep-3"]).as_deref(),
            Some(
                r#"app 11111111-1111-1111-1111-111111111111 has 3 live deployments, more than the 2 a blue/green rollout needs.
  Each live deployment is billed even when it receives no traffic.
  Live deployments: dep-1, dep-2, dep-3
  Delete the ones you no longer need: tvc deploy delete --deploy-id <DEPLOY_ID>"#
            )
        );
    }

    /// The incident that motivated this warning: four live deployments on one app.
    #[test]
    fn warns_further_above_the_threshold() {
        assert_eq!(
            live_deployment_warning(APP_ID, &["dep-1", "dep-2", "dep-3", "dep-4"]).as_deref(),
            Some(
                r#"app 11111111-1111-1111-1111-111111111111 has 4 live deployments, more than the 2 a blue/green rollout needs.
  Each live deployment is billed even when it receives no traffic.
  Live deployments: dep-1, dep-2, dep-3, dep-4
  Delete the ones you no longer need: tvc deploy delete --deploy-id <DEPLOY_ID>"#
            )
        );
    }

    #[test]
    fn live_deployment_ids_skips_deployments_marked_for_deletion() {
        let deployments = vec![
            make_deployment("live-1", false),
            make_deployment("deleted-1", true),
            make_deployment("live-2", false),
        ];

        assert_eq!(live_deployment_ids(&deployments), vec!["live-1", "live-2"]);
    }

    /// Three deployments where one is already torn down is not accumulation.
    #[test]
    fn deleted_deployments_do_not_trip_the_threshold() {
        let deployments = vec![
            make_deployment("live-1", false),
            make_deployment("live-2", false),
            make_deployment("deleted-1", true),
        ];

        assert_eq!(
            live_deployment_warning(APP_ID, &live_deployment_ids(&deployments)),
            None
        );
    }
}
