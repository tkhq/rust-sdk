//! Deploy status command.

use crate::output::Output;
use anyhow::Context;
use clap::Args as ClapArgs;
use serde::Serialize;
use turnkey_client::generated::GetTvcDeploymentRequest;

/// Get the status of a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment.
    #[arg(short, long, env = "TVC_DEPLOY_ID")]
    pub deploy_id: String,
}

#[derive(Serialize)]
struct DeployStatusOutput {
    deployment_id: String,
    app_id: String,
    manifest_id: String,
    qos_version: String,
    stage: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pivot_container: Option<PivotContainerOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

#[derive(Serialize)]
struct PivotContainerOutput {
    container_url: String,
    path: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
}

/// Run the deploy status command.
pub async fn run(args: Args, global: &crate::cli::GlobalOpts) -> anyhow::Result<()> {
    let output = Output::new(global);

    let auth = crate::client::build_client(&global.client_overrides()).await?;

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
        .ok_or_else(|| anyhow::anyhow!("manifest not found in deployment"))?;

    let pivot_container = deployment
        .pivot_container
        .as_ref()
        .map(|p| PivotContainerOutput {
            container_url: p.container_url.clone(),
            path: p.path.clone(),
            args: p.args.clone(),
        });

    let created_at = deployment
        .created_at
        .as_ref()
        .map(|t| format!("{}.{:09}s", t.seconds, t.nanos));
    let updated_at = deployment
        .updated_at
        .as_ref()
        .map(|t| format!("{}.{:09}s", t.seconds, t.nanos));

    let stage_str = format!("{:?}", deployment.stage);

    let result_data = DeployStatusOutput {
        deployment_id: deployment.id.clone(),
        app_id: deployment.app_id.clone(),
        manifest_id: manifest.id.clone(),
        qos_version: deployment.qos_version.clone(),
        stage: stage_str.clone(),
        pivot_container,
        created_at: created_at.clone(),
        updated_at: updated_at.clone(),
    };

    output.result(&result_data, || {
        println!("Deployment: {}", deployment.id);
        println!("App ID: {}", deployment.app_id);
        println!("Manifest ID: {}", manifest.id);
        println!("QOS Version: {}", deployment.qos_version);
        println!("Stage: {stage_str}");

        if let Some(pivot) = &deployment.pivot_container {
            println!();
            println!("Pivot Container:");
            println!("  URL: {}", pivot.container_url);
            println!("  Path: {}", pivot.path);
            if !pivot.args.is_empty() {
                println!("  Args: {:?}", pivot.args);
            }
        }

        if let Some(ref ts) = created_at {
            println!();
            println!("Created: {ts}");
        }

        if let Some(ref ts) = updated_at {
            println!("Updated: {ts}");
        }
    })?;

    Ok(())
}
