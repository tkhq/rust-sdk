//! Shared app status normalization and formatting helpers.

use turnkey_client::generated::external::data::v1::{AppStatus, DeploymentStatus};

/// Normalize API-specific deployment ID prefixes so CLI commands compare bare IDs.
pub fn sanitize_app_status(mut app_status: AppStatus) -> AppStatus {
    app_status.targeted_deployment_id = strip_deploy_prefix(&app_status.targeted_deployment_id);

    for deployment in &mut app_status.deployments {
        deployment.deployment_id = strip_deploy_prefix(&deployment.deployment_id);
    }

    app_status
}

/// Render replica counts consistently across app and deployment status commands.
pub fn format_replica_status(deployment_status: &DeploymentStatus) -> String {
    format!(
        "Healthy / Desired Replicas: {}/{}",
        deployment_status.ready_replicas, deployment_status.desired_replicas
    )
}

fn strip_deploy_prefix(deploy_id: &str) -> String {
    deploy_id
        .strip_prefix("deploy-")
        .unwrap_or(deploy_id)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{format_replica_status, sanitize_app_status};
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
    fn format_replica_status_uses_shared_label() {
        let deployment_status = DeploymentStatus {
            deployment_id: "deploy-123".to_string(),
            ready_replicas: 3,
            desired_replicas: 5,
            last_updated_time: None,
        };

        assert_eq!(
            format_replica_status(&deployment_status),
            "Healthy / Desired Replicas: 3/5"
        );
    }
}
