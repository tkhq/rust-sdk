//! Deploy status command.

use anyhow::Context;
use clap::Args as ClapArgs;
use turnkey_client::generated::GetTvcDeploymentRequest;
use turnkey_client::generated::external::data::v1::TvcDeployment;

use crate::client::fetch_tvc_app;
use crate::commands::display::{format_egress_enabled, yes_no};
use crate::output::StdCtx;
use crate::shell_println;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy status command.
pub async fn run(ctx: &mut StdCtx, args: Args) -> anyhow::Result<()> {
    let auth = crate::client::build_client().await?;

    let request = GetTvcDeploymentRequest {
        organization_id: auth.org_id.clone(),
        deployment_id: args.deploy_id.clone(),
    };

    let response = auth
        .client
        .get_tvc_deployment(request)
        .await
        .context("failed to fetch deployment")?;

    let deployment = response
        .tvc_deployment
        .ok_or_else(|| anyhow::anyhow!("deployment not found: {}", args.deploy_id))?;

    let manifest = deployment
        .manifest
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("manifest not found in deployment"))?;
    let app = fetch_tvc_app(&auth, &deployment.app_id).await?;

    shell_println!(ctx, "Deployment: {}", deployment.id)?;
    shell_println!(ctx, "App ID: {}", deployment.app_id)?;
    shell_println!(ctx, "{}", format_egress_enabled(app.enable_egress))?;
    shell_println!(ctx, "Manifest ID: {}", manifest.id)?;
    shell_println!(ctx, "QOS Version: {}", deployment.qos_version)?;
    shell_println!(ctx, "{}", format_marked_for_deletion(&deployment))?;

    if let Some(pivot) = &deployment.pivot_container {
        shell_println!(ctx)?;
        shell_println!(ctx, "Pivot Container:")?;
        shell_println!(ctx, "  URL: {}", pivot.container_url)?;
        shell_println!(ctx, "  Path: {}", pivot.path)?;
        if !pivot.args.is_empty() {
            shell_println!(ctx, "  Args: {:?}", pivot.args)?;
        }
    }

    if let Some(created) = &deployment.created_at {
        shell_println!(ctx)?;
        shell_println!(ctx, "Created: {}.{:0>9}s", created.seconds, created.nanos)?;
    }

    if let Some(updated) = &deployment.updated_at {
        shell_println!(ctx, "Updated: {}.{:0>9}s", updated.seconds, updated.nanos)?;
    }

    Ok(())
}

fn format_marked_for_deletion(deployment: &TvcDeployment) -> String {
    format!("Marked for deletion: {}", yes_no(deployment.delete))
}

#[cfg(test)]
mod tests {
    use super::format_marked_for_deletion;
    use turnkey_client::generated::external::data::v1::TvcDeployment;

    fn deployment(delete: bool) -> TvcDeployment {
        TvcDeployment {
            id: "deploy-123".to_string(),
            organization_id: "org-123".to_string(),
            app_id: "app-123".to_string(),
            manifest_set: None,
            share_set: None,
            manifest: None,
            manifest_approvals: vec![],
            qos_version: "qos-v1".to_string(),
            pivot_container: None,
            debug_mode: false,
            created_at: None,
            updated_at: None,
            delete,
        }
    }

    #[test]
    fn marked_for_deletion_formats_yes_when_delete_is_true() {
        assert_eq!(
            format_marked_for_deletion(&deployment(true)),
            "Marked for deletion: yes"
        );
    }

    #[test]
    fn marked_for_deletion_formats_no_when_delete_is_false() {
        assert_eq!(
            format_marked_for_deletion(&deployment(false)),
            "Marked for deletion: no"
        );
    }
}
