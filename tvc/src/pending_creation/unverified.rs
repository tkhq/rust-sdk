//! Display a coordinator's creation result when the caller explicitly skips provenance verification.

use super::{CreationKind, activity_id_argument};
use crate::{operator::OperatorPublicKey, outcome::Outcome, terminal::TerminalText};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use turnkey_client::{
    SubmissionPendingReason, TurnkeyClientError,
    generated::{
        Activity, ActivityStatus, ActivityType, CreateTvcOperatorResult, CreateTvcQuorumKeyResult,
        immutable::activity::v1::result::Inner,
    },
};
use turnkey_enclave_encrypt::QuorumPublicKey;
use turnkey_proofs::tvc::MAX_QUORUM_RECIPIENTS;
use uuid::Uuid;

pub(super) fn result(
    activity: Activity,
    kind: CreationKind,
    org_id: Uuid,
    expected_activity_id: &str,
) -> Result<Outcome> {
    ensure!(
        !activity.id.trim().is_empty() && activity.id == expected_activity_id,
        "returned activity ID does not match the requested activity ID"
    );
    ensure!(
        Uuid::parse_str(&activity.organization_id).context("invalid activity organization ID")?
            == org_id,
        "returned activity belongs to a different organization"
    );
    let expected_type = match kind {
        CreationKind::Operator => ActivityType::CreateTvcOperator,
        CreationKind::QuorumKey => ActivityType::CreateTvcQuorumKey,
    };
    ensure!(
        activity.r#type == expected_type,
        "expected activity type {}, received {}",
        expected_type.as_str_name(),
        activity.r#type.as_str_name(),
    );
    let pending_reason = match activity.status {
        ActivityStatus::Completed => None,
        ActivityStatus::Created | ActivityStatus::Pending => {
            Some(SubmissionPendingReason::Processing)
        }
        ActivityStatus::ConsensusNeeded => Some(SubmissionPendingReason::ConsensusNeeded),
        ActivityStatus::AuthenticatorsNeeded => Some(SubmissionPendingReason::AuthenticatorsNeeded),
        ActivityStatus::Failed | ActivityStatus::Rejected | ActivityStatus::Unspecified => {
            return Err(TurnkeyClientError::UnexpectedActivityStatus(
                activity.status.as_str_name().into(),
            )
            .into());
        }
    };
    if let Some(reason) = pending_reason {
        return Ok(pending(kind, Some(&activity.id), reason));
    }

    match (kind, activity.result.and_then(|result| result.inner)) {
        (CreationKind::Operator, Some(Inner::CreateTvcOperatorResult(result))) => {
            let CreateTvcOperatorResult {
                wallet_id,
                operator_id,
                encrypt_public_key,
                sign_public_key,
            } = result;
            let encrypt: OperatorPublicKey = encrypt_public_key
                .parse()
                .context("invalid operator encryption public key")?;
            let sign: OperatorPublicKey = sign_public_key
                .parse()
                .context("invalid operator signing public key")?;
            ensure!(
                encrypt != sign,
                "encryption and signing public keys must be distinct"
            );
            Ok(Outcome::UnverifiedOperatorResult(
                UnverifiedOperatorResult {
                    activity_id: activity.id,
                    operator_id: operator_id.parse().context("invalid operator ID")?,
                    wallet_id: wallet_id.parse().context("invalid wallet ID")?,
                    composite_public_key: format!("{encrypt}{sign}"),
                    encrypt_public_key: encrypt.to_string(),
                    sign_public_key: sign.to_string(),
                },
            ))
        }
        (CreationKind::QuorumKey, Some(Inner::CreateTvcQuorumKeyResult(result))) => {
            let CreateTvcQuorumKeyResult {
                quorum_key_id,
                quorum_public_key,
                share_ids,
            } = result;
            let key = QuorumPublicKey::from_string(quorum_public_key)
                .context("invalid quorum public key")?;
            key.encrypt_public_key()
                .context("invalid quorum encryption public key")?;
            key.verifying_key()
                .context("invalid quorum signing public key")?;
            ensure!(
                (2..=MAX_QUORUM_RECIPIENTS).contains(&share_ids.len()),
                "invalid number of quorum share IDs"
            );
            let share_ids = share_ids
                .into_iter()
                .map(|id| id.parse())
                .collect::<Result<Vec<Uuid>, _>>()
                .context("invalid quorum share ID")?;
            Ok(Outcome::UnverifiedQuorumKeyResult(
                UnverifiedQuorumKeyResult {
                    activity_id: activity.id,
                    quorum_key_id: quorum_key_id.parse().context("invalid quorum key ID")?,
                    quorum_public_key: hex::encode(key.as_ref()),
                    share_ids,
                },
            ))
        }
        _ => Err(TurnkeyClientError::InvalidSubmissionActivity(
            "missing or mismatched creation result",
        )
        .into()),
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default, Debug))]
#[serde(
    tag = "verificationStatus",
    rename = "unverified",
    rename_all = "camelCase"
)]
pub struct UnverifiedOperatorResult {
    activity_id: String,
    operator_id: Uuid,
    wallet_id: Uuid,
    encrypt_public_key: String,
    sign_public_key: String,
    composite_public_key: String,
}

impl Display for UnverifiedOperatorResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Unverified hosted operator result. No operator was saved.
Activity ID: {}
Operator ID: {}
Wallet ID: {}
Encryption public key: {}
Signing public key: {}
Composite public key: {}"#,
            TerminalText(&self.activity_id).single_line(),
            self.operator_id,
            self.wallet_id,
            self.encrypt_public_key,
            self.sign_public_key,
            self.composite_public_key
        )
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default, Debug))]
#[serde(
    tag = "verificationStatus",
    rename = "unverified",
    rename_all = "camelCase"
)]
pub struct UnverifiedQuorumKeyResult {
    activity_id: String,
    quorum_key_id: Uuid,
    quorum_public_key: String,
    share_ids: Vec<Uuid>,
}

impl Display for UnverifiedQuorumKeyResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Unverified quorum key result.
Activity ID: {}
Quorum Key ID: {}
Quorum Public Key: {}
Share IDs: {}"#,
            TerminalText(&self.activity_id).single_line(),
            self.quorum_key_id,
            self.quorum_public_key,
            self.share_ids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default, Debug))]
#[serde(
    tag = "verificationStatus",
    rename = "unverified",
    rename_all = "camelCase"
)]
pub struct UnverifiedCreationPending {
    activity_id: Option<String>,
    #[serde(skip)]
    kind: CreationKind,
    status: &'static str,
}

pub(super) fn pending(
    kind: CreationKind,
    activity_id: Option<&str>,
    reason: SubmissionPendingReason,
) -> Outcome {
    Outcome::UnverifiedCreationPending(UnverifiedCreationPending {
        activity_id: activity_id.map(str::to_owned),
        kind,
        status: match reason {
            SubmissionPendingReason::Processing => "processing",
            SubmissionPendingReason::ConsensusNeeded => "consensus_needed",
            SubmissionPendingReason::AuthenticatorsNeeded => "authenticators_needed",
        },
    })
}

impl Display for UnverifiedCreationPending {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Creation is pending ({}). No keys are available.
Look up the unverified result with: {} {} --dangerous-skip-verification"#,
            self.status,
            self.kind.verify_result_command(),
            activity_id_argument(self.activity_id.as_deref())
                .as_deref()
                .unwrap_or("--activity-id <candidate>")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    const ACTIVITY_ID: &str = "lookup-id";
    const QUORUM_ID: Uuid = Uuid::from_u128(1);
    const ORG: Uuid = Uuid::from_u128(2);

    fn quorum(status: ActivityStatus, result: Value) -> Result<Outcome> {
        let activity = serde_json::from_value(json!({
            "id": ACTIVITY_ID,
            "organizationId": ORG,
            "type": ActivityType::CreateTvcQuorumKey,
            "status": status,
            "fingerprint": "",
            "result": result,
        }))
        .unwrap();
        super::result(activity, CreationKind::QuorumKey, ORG, ACTIVITY_ID)
    }

    fn quorum_result(key: String, share_ids: Value) -> Value {
        json!({"createTvcQuorumKeyResult": {
            "quorumKeyId": QUORUM_ID, "quorumPublicKey": key, "shareIds": share_ids
        }})
    }

    fn assert_pending(status: ActivityStatus, expected: &str) {
        let key = hex::encode(QuorumPublicKey::dev_signer().as_ref());
        let result = quorum_result(key, json!([Uuid::from_u128(3), Uuid::from_u128(4)]));
        let outcome = quorum(status, result).unwrap();
        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            json!({
                "reason": "unverified_creation_pending",
                "verificationStatus": "unverified",
                "activityId": ACTIVITY_ID,
                "status": expected,
            })
        );
        assert_eq!(
            outcome.to_string(),
            format!(
                r#"Creation is pending ({expected}). No keys are available.
Look up the unverified result with: tvc keys create-quorum-key verify-result --activity-id {ACTIVITY_ID} --dangerous-skip-verification"#
            )
        );
    }

    #[test]
    fn created_activity_with_keys_is_still_processing() {
        assert_pending(ActivityStatus::Created, "processing");
    }

    #[test]
    fn pending_activity_with_keys_is_still_processing() {
        assert_pending(ActivityStatus::Pending, "processing");
    }

    #[test]
    fn consensus_needed_activity_with_keys_is_pending() {
        assert_pending(ActivityStatus::ConsensusNeeded, "consensus_needed");
    }

    #[test]
    fn authenticators_needed_activity_with_keys_is_pending() {
        assert_pending(
            ActivityStatus::AuthenticatorsNeeded,
            "authenticators_needed",
        );
    }

    #[test]
    fn rejected_activity_returns_an_error() {
        assert!(quorum(ActivityStatus::Rejected, Value::Null).is_err());
    }

    #[test]
    fn unspecified_status_returns_an_error() {
        assert!(quorum(ActivityStatus::Unspecified, Value::Null).is_err());
    }

    #[test]
    fn completed_quorum_activity_rejects_an_operator_result() {
        let result = json!({"createTvcOperatorResult": {
            "walletId": "", "operatorId": "", "encryptPublicKey": "", "signPublicKey": ""
        }});
        assert!(quorum(ActivityStatus::Completed, result).is_err());
    }

    #[test]
    fn invalid_composite_point_fails_before_display() {
        let result = quorum_result(
            "00".repeat(130),
            json!([Uuid::from_u128(3), Uuid::from_u128(4)]),
        );
        let error = quorum(ActivityStatus::Completed, result).err().unwrap();
        assert_eq!(error.to_string(), "invalid quorum encryption public key");
    }

    #[test]
    fn invalid_share_id_fails_before_display() {
        let key = hex::encode(QuorumPublicKey::dev_signer().as_ref());
        let result = quorum_result(key, json!([Uuid::from_u128(3), "bad"]));
        let error = quorum(ActivityStatus::Completed, result).err().unwrap();
        assert_eq!(error.to_string(), "invalid quorum share ID");
    }
}
