//! Helpers for activity responses that were accepted but still need quorum.

use anyhow::Result;
use turnkey_client::TurnkeyClientError;

pub(crate) struct PendingConsensusActivity<'a> {
    pub activity_id: &'a str,
    pub fingerprint: &'a str,
}

pub(crate) fn pending_consensus_from_error(
    error: &TurnkeyClientError,
) -> Option<PendingConsensusActivity<'_>> {
    match error {
        TurnkeyClientError::ActivityPendingConsensus {
            activity_id,
            fingerprint,
        } => Some(PendingConsensusActivity {
            activity_id,
            fingerprint,
        }),
        _ => None,
    }
}

pub(crate) fn print_pending_consensus(activity: &PendingConsensusActivity<'_>) {
    println!();
    println!("Consensus needed. The activity was created successfully but is pending quorum.");
    println!();
    println!("Activity ID: {}", activity.activity_id);
    println!("Fingerprint: {}", activity.fingerprint);
    println!();
    println!("Next steps:");
    println!(
        "  - Ask another quorum member to run `tvc activity approve --fingerprint {}`",
        activity.fingerprint
    );
}

pub(crate) fn pending_consensus_result(activity: &PendingConsensusActivity<'_>) -> Result<()> {
    print_pending_consensus(activity);
    Err(crate::exit::ExitError::consensus_needed().into())
}
