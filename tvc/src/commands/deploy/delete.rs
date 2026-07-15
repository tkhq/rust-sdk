//! Deploy delete command - marks a deployment for deletion.

use crate::client::build_client;
use anyhow::Result;
use clap::Args as ClapArgs;
use turnkey_client::generated::{ActivityType, DeleteTvcDeploymentIntent, intent};

/// Delete a TVC deployment by marking it for deletion.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// ID of the deployment to delete.
    #[arg(long, env = "TVC_DEPLOY_ID", value_name = "DEPLOY_ID")]
    pub deploy_id: String,
}

/// Run the deploy delete command.
pub async fn run(args: Args) -> Result<()> {
    let auth = build_client().await?;

    let intent = DeleteTvcDeploymentIntent {
        deployment_id: args.deploy_id.clone(),
    };

    let intent_inner = intent::Inner::DeleteTvcDeploymentIntent(intent.clone());
    let result = match crate::commands::consensus::submit_with_consensus(
        &auth,
        ActivityType::DeleteTvcDeployment,
        "failed to check for pending deployment delete activities",
        "failed to delete TVC deployment",
        |pending_intent| pending_intent == &intent_inner,
        |timestamp_ms| {
            auth.client
                .delete_tvc_deployment(auth.org_id.clone(), timestamp_ms, intent)
        },
    )
    .await?
    {
        crate::commands::consensus::ConsensusSubmission::Submitted(result) => result,
        crate::commands::consensus::ConsensusSubmission::ExistingActivityHandled => return Ok(()),
    };

    println!();
    println!("Deployment delete accepted; deployment is marked for deletion.");
    println!();
    println!("Deployment ID: {}", result.result.deployment_id);
    println!("Activity ID: {}", result.activity_id);
    println!("Activity Status: {:?}", result.status);

    Ok(())
}
