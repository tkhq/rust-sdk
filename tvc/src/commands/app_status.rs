//! Shared app status normalization and formatting helpers.

use serde::Serialize;
use turnkey_client::generated::external::data::v1::{AppStatus, Timestamp};

/// Normalize API-specific deployment ID prefixes so CLI commands compare bare IDs.
pub fn sanitize_app_status(mut app_status: AppStatus) -> AppStatus {
    app_status.targeted_deployment_id = strip_deploy_prefix(&app_status.targeted_deployment_id);

    for deployment in &mut app_status.deployments {
        deployment.deployment_id = strip_deploy_prefix(&deployment.deployment_id);
    }

    app_status
}

/// Serializable `{seconds, nanos}` pair for outcome payloads (both values are
/// stringified integers, mirroring the API's `Timestamp`).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampPayload {
    pub seconds: String,
    pub nanos: String,
}

impl From<Timestamp> for TimestampPayload {
    fn from(timestamp: Timestamp) -> Self {
        // Exhaustive destructure so a new `Timestamp` field forces a decision here.
        let Timestamp { seconds, nanos } = timestamp;
        Self { seconds, nanos }
    }
}

/// Ready/desired replica counts for outcome payloads.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaCounts {
    pub ready: i32,
    pub desired: i32,
}

/// Render replica counts consistently across app and deployment status commands.
pub fn format_replica_counts(ready: i32, desired: i32) -> String {
    format!("Healthy / Desired Replicas: {ready}/{desired}")
}

fn strip_deploy_prefix(deploy_id: &str) -> String {
    deploy_id
        .strip_prefix("deploy-")
        .unwrap_or(deploy_id)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{format_replica_counts, sanitize_app_status};
    use turnkey_client::generated::external::data::v1::{AppStatus, DeploymentStatus};

    #[test]
    fn sanitize_app_status_strips_deploy_prefixes() {
        let app_status = AppStatus {
            app_id: "app-123".to_string(),
            deployments: vec![DeploymentStatus {
                deployment_id: "deploy-5376f492-d014-4e01-a6bb-20fc97448e25".to_string(),
                ready_replicas: 3,
                desired_replicas: 3,
                last_updated_time: None,
            }],
            targeted_deployment_id: "deploy-5376f492-d014-4e01-a6bb-20fc97448e25".to_string(),
        };

        let sanitized = sanitize_app_status(app_status);

        assert_eq!(
            sanitized.targeted_deployment_id,
            "5376f492-d014-4e01-a6bb-20fc97448e25"
        );
        assert_eq!(
            sanitized.deployments[0].deployment_id,
            "5376f492-d014-4e01-a6bb-20fc97448e25"
        );
    }

    #[test]
    fn format_replica_counts_uses_shared_label() {
        assert_eq!(
            format_replica_counts(3, 5),
            "Healthy / Desired Replicas: 3/5"
        );
    }
}
