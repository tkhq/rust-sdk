//! Helpers for activity responses that were accepted but still need quorum.

use crate::client::AuthenticatedClient;
use anyhow::{Context, Result};
use std::future::Future;
use turnkey_client::generated::intent::Inner;
use turnkey_client::generated::{
    Activity, ActivityStatus, ActivityType, ApproveActivityIntent, GetActivitiesRequest,
    GetOrganizationConfigsRequest, GetWhoamiRequest,
};
use turnkey_client::{ActivityResult, TurnkeyClientError};

const VOTE_SELECTION_APPROVED: &str = "VOTE_SELECTION_APPROVED";

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

/// Find a pending activity whose intent satisfies `matches_intent`.
///
/// The submission timestamp lives in the request envelope (`timestampMs`),
/// not in the intent, so intent matching identifies an operator re-running
/// the same request even though the server fingerprints each raw request
/// body differently.
pub(crate) fn find_matching_pending_activity_by(
    activities: &[Activity],
    matches_intent: impl Fn(&Inner) -> bool,
) -> Option<&Activity> {
    activities.iter().find(|activity| {
        activity
            .intent
            .as_ref()
            .and_then(|intent| intent.inner.as_ref())
            .is_some_and(&matches_intent)
    })
}

pub(crate) enum ConsensusSubmission<T> {
    Submitted(ActivityResult<T>),
    ExistingActivityHandled,
}

pub(crate) async fn submit_with_consensus<T, MatchesIntent, Submit, SubmitFuture>(
    auth: &AuthenticatedClient,
    activity_type: ActivityType,
    list_context: &'static str,
    submit_context: &'static str,
    matches_intent: MatchesIntent,
    submit: Submit,
) -> Result<ConsensusSubmission<T>>
where
    MatchesIntent: Fn(&Inner) -> bool,
    Submit: FnOnce(u128) -> SubmitFuture,
    SubmitFuture: Future<Output = std::result::Result<ActivityResult<T>, TurnkeyClientError>>,
{
    let pending = auth
        .client
        .get_activities(GetActivitiesRequest {
            organization_id: auth.org_id.clone(),
            filter_by_status: vec![ActivityStatus::ConsensusNeeded],
            pagination_options: None,
            filter_by_type: vec![activity_type],
        })
        .await
        .context(list_context)?;

    if let Some(existing) = find_matching_pending_activity_by(&pending.activities, matches_intent) {
        vote_on_existing_activity(auth, existing).await?;
        return Ok(ConsensusSubmission::ExistingActivityHandled);
    }

    let result = submit(auth.client.current_timestamp()).await;
    match result {
        Ok(result) => Ok(ConsensusSubmission::Submitted(result)),
        Err(error) => {
            if let Some(activity) = pending_consensus_from_error(&error) {
                pending_consensus_result(&activity)?;
                unreachable!("pending_consensus_result always returns an error");
            }
            Err(error).context(submit_context)
        }
    }
}

/// Number of approval votes already recorded on an activity.
pub(crate) fn approval_vote_count(activity: &Activity) -> usize {
    activity
        .votes
        .iter()
        .filter(|vote| vote.selection == VOTE_SELECTION_APPROVED)
        .count()
}

/// Whether `user_id` has already recorded an approval vote on the activity.
pub(crate) fn user_already_approved(activity: &Activity, user_id: &str) -> bool {
    activity
        .votes
        .iter()
        .any(|vote| vote.user_id == user_id && vote.selection == VOTE_SELECTION_APPROVED)
}

/// Resolve the authenticated user's ID via the whoami endpoint.
pub(crate) async fn current_user_id(auth: &AuthenticatedClient) -> Result<String> {
    let whoami = auth
        .client
        .get_whoami(GetWhoamiRequest {
            organization_id: auth.org_id.clone(),
        })
        .await
        .context("failed to resolve current user")?;
    Ok(whoami.user_id)
}

/// Best-effort lookup of the org's root quorum threshold for progress display.
async fn root_quorum_threshold(auth: &AuthenticatedClient) -> Option<i32> {
    auth.client
        .get_organization_configs(GetOrganizationConfigsRequest {
            organization_id: auth.org_id.clone(),
        })
        .await
        .ok()?
        .configs?
        .quorum
        .map(|quorum| quorum.threshold)
}

/// Print the "already approved" message with vote progress.
pub(crate) async fn print_already_approved(auth: &AuthenticatedClient, activity: &Activity) {
    let approvals = approval_vote_count(activity);
    let progress = match root_quorum_threshold(auth).await {
        Some(threshold) if threshold > 0 => {
            format!("{approvals} of {threshold} approval votes so far")
        }
        _ => format!("{approvals} approval vote(s) so far"),
    };
    println!(
        "You have already approved this activity; waiting on other quorum members — {progress}."
    );
}

/// Vote (approve) on an existing pending activity instead of creating a
/// duplicate one.
///
/// Returns `Ok(())` when the vote completed quorum. When the activity is
/// still pending quorum after the vote, prints "vote recorded" and returns
/// the dedicated consensus-needed exit error (code 2) because the overall
/// operation is not complete yet.
pub(crate) async fn vote_on_existing_activity(
    auth: &AuthenticatedClient,
    existing: &Activity,
) -> Result<()> {
    println!();
    println!(
        "An identical request is already pending consensus; voting on it instead of creating a duplicate activity."
    );
    println!();
    println!("Activity ID: {}", existing.id);
    println!("Fingerprint: {}", existing.fingerprint);

    let user_id = current_user_id(auth).await?;
    if user_already_approved(existing, &user_id) {
        println!();
        print_already_approved(auth, existing).await;
        return Ok(());
    }

    let timestamp_ms = auth.client.current_timestamp();
    let result = auth
        .client
        .approve_activity(
            auth.org_id.clone(),
            timestamp_ms,
            ApproveActivityIntent {
                fingerprint: existing.fingerprint.clone(),
            },
        )
        .await;

    match result {
        Ok(activity) => {
            println!();
            println!(
                "Approval vote submitted: quorum reached and activity {} completed.",
                activity.id
            );
            Ok(())
        }
        Err(TurnkeyClientError::ActivityPendingConsensus {
            activity_id,
            fingerprint,
        }) => {
            println!();
            println!("Vote recorded. Activity {activity_id} is still pending quorum.");
            println!();
            println!("Next steps:");
            println!(
                "  - Ask another quorum member to run `tvc activity approve --fingerprint {fingerprint}`"
            );
            Err(crate::exit::ExitError::consensus_needed().into())
        }
        Err(error) => Err(error).context("failed to vote on existing pending activity"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turnkey_client::generated::intent::Inner;
    use turnkey_client::generated::{
        Activity, ActivityStatus, ActivityType, CreateTvcManifestApprovalsIntent, Intent,
        TvcManifestApproval,
    };

    fn approvals_intent(manifest_id: &str, signature: &str) -> Inner {
        Inner::CreateTvcManifestApprovalsIntent(CreateTvcManifestApprovalsIntent {
            manifest_id: manifest_id.to_string(),
            approvals: vec![TvcManifestApproval {
                operator_id: "operator-1".to_string(),
                signature: signature.to_string(),
            }],
        })
    }

    fn pending_activity(id: &str, intent: Option<Inner>) -> Activity {
        Activity {
            id: id.to_string(),
            organization_id: "org-test".to_string(),
            status: ActivityStatus::ConsensusNeeded,
            r#type: ActivityType::CreateTvcManifestApprovals,
            intent: intent.map(|inner| Intent { inner: Some(inner) }),
            result: None,
            votes: vec![],
            app_proofs: vec![],
            fingerprint: format!("fp-{id}"),
            can_approve: true,
            can_reject: true,
            created_at: None,
            updated_at: None,
            failure: None,
        }
    }

    #[test]
    fn matcher_finds_activity_with_identical_intent() {
        let activities = vec![
            pending_activity("other", Some(approvals_intent("manifest-b", "sig-b"))),
            pending_activity("match", Some(approvals_intent("manifest-a", "sig-a"))),
        ];

        let found = find_matching_pending_activity_by(&activities, |inner| {
            inner == &approvals_intent("manifest-a", "sig-a")
        });

        assert_eq!(found.map(|a| a.id.as_str()), Some("match"));
    }

    #[test]
    fn matcher_rejects_activity_with_different_intent_contents() {
        let activities = vec![pending_activity(
            "other",
            Some(approvals_intent("manifest-a", "another-operator-sig")),
        )];

        let found = find_matching_pending_activity_by(&activities, |inner| {
            inner == &approvals_intent("manifest-a", "sig-a")
        });

        assert!(found.is_none());
    }

    #[test]
    fn matcher_ignores_activities_without_intent() {
        let activities = vec![pending_activity("no-intent", None)];

        let found = find_matching_pending_activity_by(&activities, |inner| {
            inner == &approvals_intent("manifest-a", "sig-a")
        });

        assert!(found.is_none());
    }

    #[test]
    fn generic_matcher_finds_activity_with_matching_predicate() {
        let activities = vec![
            pending_activity("other", Some(approvals_intent("manifest-b", "sig-b"))),
            pending_activity("match", Some(approvals_intent("manifest-a", "sig-a"))),
        ];

        let found = find_matching_pending_activity_by(&activities, |inner| {
            inner == &approvals_intent("manifest-a", "sig-a")
        });

        assert_eq!(found.map(|a| a.id.as_str()), Some("match"));
    }

    #[test]
    fn generic_matcher_ignores_activities_without_matching_predicate() {
        let activities = vec![pending_activity(
            "other",
            Some(approvals_intent("manifest-a", "sig-a")),
        )];

        let found = find_matching_pending_activity_by(&activities, |inner| {
            inner == &approvals_intent("manifest-a", "sig-b")
        });

        assert!(found.is_none());
    }

    fn vote(user_id: &str, selection: &str) -> turnkey_client::generated::Vote {
        turnkey_client::generated::Vote {
            id: format!("vote-{user_id}"),
            user_id: user_id.to_string(),
            user: None,
            activity_id: "activity-1".to_string(),
            selection: selection.to_string(),
            message: "{}".to_string(),
            public_key: "public-key".to_string(),
            signature: "signature".to_string(),
            scheme: "SIGNATURE_SCHEME_TK_API_P256".to_string(),
            created_at: None,
        }
    }

    #[test]
    fn approval_vote_count_only_counts_approvals() {
        let mut activity = pending_activity("counted", None);
        activity.votes = vec![
            vote("user-1", VOTE_SELECTION_APPROVED),
            vote("user-2", "VOTE_SELECTION_REJECTED"),
            vote("user-3", VOTE_SELECTION_APPROVED),
        ];

        assert_eq!(approval_vote_count(&activity), 2);
    }

    #[test]
    fn user_already_approved_matches_only_approvals_by_that_user() {
        let mut activity = pending_activity("voted", None);
        activity.votes = vec![
            vote("user-1", VOTE_SELECTION_APPROVED),
            vote("user-2", "VOTE_SELECTION_REJECTED"),
        ];

        assert!(user_already_approved(&activity, "user-1"));
        assert!(!user_already_approved(&activity, "user-2"));
        assert!(!user_already_approved(&activity, "user-3"));
    }
}
