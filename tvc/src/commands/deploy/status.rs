//! Deploy status command.

use anyhow::Context;
use clap::Args as ClapArgs;
use turnkey_client::generated::GetTvcDeploymentRequest;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy status command.
pub async fn run(args: Args) -> anyhow::Result<()> {
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

    println!("Deployment: {}", deployment.id);
    println!("App ID: {}", deployment.app_id);
    println!("Manifest ID: {}", deployment.manifest_id);
    println!("QOS Version: {}", deployment.qos_version);
    println!("Stage: {:?}", deployment.stage);

    if let Some(pivot) = &deployment.pivot_container {
        println!();
        println!("Pivot Container:");
        println!("  URL: {}", pivot.container_url);
        println!("  Path: {}", pivot.path);
        if !pivot.args.is_empty() {
            println!("  Args: {:?}", pivot.args);
        }
    }

    if let Some(host) = &deployment.host_container {
        println!();
        println!("Host Container:");
        println!("  URL: {}", host.container_url);
        println!("  Path: {}", host.path);
        if !host.args.is_empty() {
            println!("  Args: {:?}", host.args);
        }
    }

    if let Some(created) = &deployment.created_at {
        println!();
        println!("Created: {}.{:09}s", created.seconds, created.nanos);
    }

    if let Some(updated) = &deployment.updated_at {
        println!("Updated: {}.{:09}s", updated.seconds, updated.nanos);
    }

    Ok(())
}
