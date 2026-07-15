//! Activity reject command - vote to reject a pending activity by fingerprint.

use crate::client::build_client;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use turnkey_client::TurnkeyClientError;
use turnkey_client::generated::RejectActivityIntent;

/// Reject (vote against) an activity pending consensus.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Fingerprint of the activity to reject (see `tvc activity list`).
    #[arg(long, env = "TVC_ACTIVITY_FINGERPRINT")]
    pub fingerprint: String,
}

pub async fn run(args: Args) -> Result<()> {
    let auth = build_client().await?;
    let timestamp_ms = auth.client.current_timestamp();

    let result = auth
        .client
        .reject_activity(
            auth.org_id.clone(),
            timestamp_ms,
            RejectActivityIntent {
                fingerprint: args.fingerprint.clone(),
            },
        )
        .await;

    match result {
        Ok(activity) => {
            println!(
                "Rejection submitted for activity {} (fingerprint {}).",
                activity.id, args.fingerprint
            );
            Ok(())
        }
        // A completed rejection flips the activity to ACTIVITY_STATUS_REJECTED,
        // which the client's polling loop reports as an unexpected status.
        Err(TurnkeyClientError::UnexpectedActivityStatus(status))
            if status == "ACTIVITY_STATUS_REJECTED" =>
        {
            println!(
                "Rejection submitted for activity with fingerprint {}. The activity is now rejected.",
                args.fingerprint
            );
            Ok(())
        }
        Err(TurnkeyClientError::ActivityPendingConsensus { activity_id, .. }) => {
            println!("Rejection vote recorded. Activity {activity_id} is still pending quorum.");
            Ok(())
        }
        Err(error) => Err(error).context("failed to reject activity"),
    }
}
