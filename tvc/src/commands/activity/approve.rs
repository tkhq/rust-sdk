//! Activity approve command - vote to approve a pending activity by fingerprint.

use crate::client::build_client;
use crate::commands::consensus;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use turnkey_client::TurnkeyClientError;
use turnkey_client::generated::{ActivityStatus, ApproveActivityIntent, GetActivitiesRequest};

/// Approve (vote on) an activity pending consensus.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Fingerprint of the activity to approve (see `tvc activity list`).
    #[arg(long, env = "TVC_ACTIVITY_FINGERPRINT")]
    pub fingerprint: String,
}

pub async fn run(args: Args) -> Result<()> {
    let auth = build_client().await?;

    // If the target activity is still pending and the current user already
    // approved it, don't submit a second vote.
    let pending = auth
        .client
        .get_activities(GetActivitiesRequest {
            organization_id: auth.org_id.clone(),
            filter_by_status: vec![ActivityStatus::ConsensusNeeded],
            pagination_options: None,
            filter_by_type: vec![],
        })
        .await
        .context("failed to list pending activities")?;

    if let Some(activity) = pending
        .activities
        .iter()
        .find(|activity| activity.fingerprint == args.fingerprint)
    {
        let user_id = consensus::current_user_id(&auth).await?;
        if consensus::user_already_approved(activity, &user_id) {
            println!("Activity ID: {}", activity.id);
            consensus::print_already_approved(&auth, activity).await;
            return Ok(());
        }
    }

    let timestamp_ms = auth.client.current_timestamp();

    let result = auth
        .client
        .approve_activity(
            auth.org_id.clone(),
            timestamp_ms,
            ApproveActivityIntent {
                fingerprint: args.fingerprint.clone(),
            },
        )
        .await;

    match result {
        Ok(activity) => {
            println!("Approval submitted. Activity {} completed.", activity.id);
            Ok(())
        }
        Err(TurnkeyClientError::ActivityPendingConsensus {
            activity_id,
            fingerprint,
        }) => {
            println!("Vote recorded. Activity {activity_id} is still pending quorum.");
            println!(
                "Other quorum members can vote with `tvc activity approve --fingerprint {fingerprint}`."
            );
            Ok(())
        }
        Err(error) => Err(error).context("failed to approve activity"),
    }
}
