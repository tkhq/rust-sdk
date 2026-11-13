//! Durable local origin evidence shared by creation and restart recovery.

use crate::{
    client::AuthenticatedClient,
    commands::{keys::create_quorum_key::validate_result, operator::create::save_hosted_operator},
    config::turnkey::{config_dir, config_file_path},
    operator::hosted_creation_error,
    outcome::Outcome,
    output::StdCtx,
    terminal::TerminalText,
    util::{atomic_write_owner_only, lock_owner_only_file, owner_only_dir},
};
use anyhow::{Context, Result, ensure};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use turnkey_client::{
    ActivityReceipt, CheckpointSaveError, SubmissionCheckpoint, SubmissionOutcome,
    SubmissionPendingReason, SubmissionState, SubmittedActivityResult, TurnkeyClientError,
    generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent, CreateTvcQuorumKeyResult},
};
use turnkey_proofs::tvc::{OperatorExpectation, QuorumExpectation, TvcProvenance};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum Creation {
    Operator {
        intent: CreateTvcOperatorIntent,
        org_alias: String,
        make_default: bool,
    },
    Quorum {
        intent: CreateTvcQuorumKeyIntent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedCreation {
    version: u32,
    api_base_url: String,
    timestamp_ms: String,
    creation: Creation,
    checkpoint: SubmissionCheckpoint,
}

#[derive(Debug, thiserror::Error)]
#[error("{submission}")]
pub(crate) struct CheckpointPersistenceFailure {
    pub(crate) submission: TurnkeyClientError,
    #[source]
    source: anyhow::Error,
}

/// The lock remains on a separate inode when the JSON file is replaced.
pub(crate) struct Operation {
    id: Uuid,
    path: PathBuf,
    _lock: File,
}

impl Operation {
    pub(crate) fn open(id: Uuid) -> Result<Self> {
        Self::at(&config_dir()?.join("pending-creations"), id)
    }

    fn at(directory: &Path, id: Uuid) -> Result<Self> {
        owner_only_dir(directory).with_context(|| {
            format!(
                "failed to create saved creation directory {}",
                directory.display()
            )
        })?;
        let lock_path = directory.join(format!("{id}.lock"));
        let lock = lock_owner_only_file(&lock_path).with_context(|| {
            format!("cannot lock creation {id}; another invocation may be using it")
        })?;
        Ok(Self {
            id,
            path: directory.join(format!("{id}.json")),
            _lock: lock,
        })
    }

    pub(crate) fn load(&self) -> Result<SavedCreation> {
        let metadata = std::fs::symlink_metadata(&self.path)
            .with_context(|| format!("failed to inspect saved creation {}", self.id))?;
        ensure!(
            metadata.is_file(),
            "saved creation {} must be a regular file, not a symlink",
            self.id
        );
        ensure!(
            metadata.permissions().mode() & 0o077 == 0,
            "saved creation {} must be accessible only to its owner",
            self.id
        );
        let contents = std::fs::read(&self.path)
            .with_context(|| format!("failed to read saved creation {}", self.id))?;
        let saved: SavedCreation = serde_json::from_slice(&contents)
            .with_context(|| format!("failed to parse saved creation {}", self.id))?;
        ensure!(
            saved.version == 1,
            "unsupported saved creation version {}",
            saved.version
        );
        saved
            .timestamp_ms
            .parse::<u128>()
            .context("invalid saved creation timestamp")?;
        Ok(saved)
    }

    fn save(&self, saved: &SavedCreation) -> Result<()> {
        let contents = serde_json::to_vec_pretty(saved)?;
        atomic_write_owner_only(&self.path, &contents).with_context(|| {
            format!(
                "failed to save creation {} at {}",
                self.id,
                self.path.display()
            )
        })
    }

    fn recovery(&self) -> String {
        format!(
            "saved creation {}; resume with `tvc creation resume {}` (add `--activity-id <candidate>` if no hint was received); do not retry creation blindly",
            self.id, self.id
        )
    }
}

fn new_operation_id() -> Result<Uuid> {
    let mut bytes = [0; 16];
    OsRng
        .try_fill_bytes(&mut bytes)
        .context("failed to generate local creation UUID")?;
    Ok(uuid::Builder::from_random_bytes(bytes).into_uuid())
}

pub(crate) async fn submit(
    ctx: &mut StdCtx,
    auth: AuthenticatedClient,
    creation: Creation,
    timestamp_ms: u128,
    verbose_proofs: bool,
) -> Result<Outcome> {
    let operation = Operation::open(new_operation_id()?)?;
    operation
        .submit(ctx, auth, creation, timestamp_ms, verbose_proofs)
        .await
}

impl Operation {
    async fn submit(
        &self,
        ctx: &mut StdCtx,
        auth: AuthenticatedClient,
        creation: Creation,
        timestamp_ms: u128,
        verbose_proofs: bool,
    ) -> Result<Outcome> {
        let operation = self;
        let save_error = Arc::new(Mutex::new(None));
        let callback_error = Arc::clone(&save_error);
        let api_base_url = auth.api_base_url.clone();
        let saved_intent = creation.clone();
        let path = operation.path.clone();
        let operation_id = operation.id;
        let save = move |checkpoint: &SubmissionCheckpoint| {
            let saved = SavedCreation {
                version: 1,
                api_base_url: api_base_url.clone(),
                timestamp_ms: timestamp_ms.to_string(),
                creation: saved_intent.clone(),
                checkpoint: checkpoint.clone(),
            };
            let result = serde_json::to_vec_pretty(&saved)
                .map_err(anyhow::Error::new)
                .and_then(|contents| {
                    atomic_write_owner_only(&path, &contents).with_context(|| {
                        format!(
                            "failed to save creation {operation_id} at {}",
                            path.display()
                        )
                    })
                });
            std::future::ready(result.map_err(|error| {
                *callback_error
                    .lock()
                    .expect("checkpoint error slot poisoned") = Some(error);
                CheckpointSaveError
            }))
        };
        let result = match &creation {
            Creation::Operator { intent, .. } => auth
                .client
                .create_tvc_operator_with_receipt(
                    auth.org_id.clone(),
                    timestamp_ms,
                    intent.clone(),
                    save,
                )
                .await
                .map(receipt_outcome),
            Creation::Quorum { intent } => auth
                .client
                .create_tvc_quorum_key_with_receipt(
                    auth.org_id.clone(),
                    timestamp_ms,
                    intent.clone(),
                    save,
                )
                .await
                .map(receipt_outcome),
        };
        let save_error = save_error
            .lock()
            .expect("checkpoint error slot poisoned")
            .take();
        if let Some(error) = save_error {
            let recovery = match &result {
                Err(TurnkeyClientError::SubmissionFailed(failure))
                    if failure.submission_state() == SubmissionState::NeverSubmitted =>
                {
                    format!(
                        "local operation {}; no remote creation request was sent",
                        operation.id
                    )
                }
                _ => operation.recovery(),
            };
            let error = match result {
                Err(submission) => anyhow::Error::new(CheckpointPersistenceFailure {
                    submission,
                    source: error,
                }),
                Ok(_) => error,
            };
            return Err(error.context(format!(
                "checkpoint save failed; latest attempt was not sent; {recovery}"
            )));
        }
        // A pre-send callback may not run if signing fails. Preserve that SDK failure.
        let saved = match operation.load() {
            Ok(saved) => saved,
            Err(error) => {
                return match result {
                    Err(submission) => Err(hosted_creation_error(
                        "submit TVC creation",
                        submission,
                    )
                    .context(format!(
                        "local operation {} has no saved checkpoint; {error:#}",
                        operation.id
                    ))),
                    Ok(_) => Err(error),
                };
            }
        };
        finish(ctx, &auth, operation, saved, result, verbose_proofs).await
    }
}

fn receipt_outcome<T>(
    outcome: SubmissionOutcome<SubmittedActivityResult<T>>,
) -> SubmissionOutcome<ActivityReceipt> {
    match outcome {
        SubmissionOutcome::Completed(submitted) => {
            SubmissionOutcome::Completed(submitted.into_parts().1)
        }
        SubmissionOutcome::Pending { checkpoint, reason } => {
            SubmissionOutcome::Pending { checkpoint, reason }
        }
    }
}

impl SavedCreation {
    pub(crate) fn for_client(self, auth: &AuthenticatedClient) -> Result<Self> {
        ensure!(
            auth.org_id == self.checkpoint.organization_id(),
            "saved creation belongs to a different organization"
        );
        ensure!(
            auth.api_base_url == self.api_base_url,
            "saved creation belongs to a different API environment"
        );
        Ok(self)
    }

    pub(crate) async fn resume(
        self,
        ctx: &mut StdCtx,
        auth: &AuthenticatedClient,
        operation: &Operation,
        activity_id: Option<String>,
        verbose_proofs: bool,
    ) -> Result<Outcome> {
        let result = auth
            .client
            .resume_submission(self.checkpoint.clone(), activity_id)
            .await;
        finish(ctx, auth, operation, self, result, verbose_proofs).await
    }
}

async fn finish(
    ctx: &mut StdCtx,
    auth: &AuthenticatedClient,
    operation: &Operation,
    saved: SavedCreation,
    result: std::result::Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError>,
    verbose_proofs: bool,
) -> Result<Outcome> {
    let checkpoint = match &result {
        Ok(SubmissionOutcome::Completed(receipt)) => Some(receipt.checkpoint()),
        Ok(SubmissionOutcome::Pending { checkpoint, .. }) => Some(checkpoint),
        Err(TurnkeyClientError::SubmissionFailed(failure)) => failure.checkpoint(),
        Err(_) => None,
    };
    let saved = match checkpoint {
        Some(checkpoint) => SavedCreation {
            checkpoint: checkpoint.clone(),
            ..saved
        },
        None => saved,
    };
    operation
        .save(&saved)
        .with_context(|| operation.recovery())?;
    let outcome = result
        .map_err(|error| hosted_creation_error("complete TVC creation", error))
        .with_context(|| operation.recovery())?;
    match outcome {
        SubmissionOutcome::Pending { checkpoint, reason } => {
            Ok(Outcome::CreationPending(CreationPending {
                operation_id: operation.id,
                activity_id: checkpoint.activity_id().map(str::to_owned),
                status: match reason {
                    SubmissionPendingReason::Processing => "processing",
                    SubmissionPendingReason::ConsensusNeeded => "consensus_needed",
                    SubmissionPendingReason::AuthenticatorsNeeded => "authenticators_needed",
                }
                .to_owned(),
            }))
        }
        SubmissionOutcome::Completed(receipt) => {
            let verifier = TvcProvenance::for_api_url(&saved.api_base_url)?;
            match saved.creation {
                Creation::Operator {
                    intent,
                    org_alias,
                    make_default,
                } => {
                    let expected = OperatorExpectation::parse(
                        saved.checkpoint.organization_id().to_owned(),
                        intent.wallet_id,
                        intent.path,
                    )?;
                    let (result, report) = verifier
                        .accept_operator_reported(&auth.client, &expected, receipt)
                        .await
                        .into_parts();
                    crate::proofs::present(ctx.shell(), &result, &report, verbose_proofs);
                    let accepted = result
                        .context("proof acceptance failed; no keys were saved")
                        .with_context(|| operation.recovery())?;
                    save_hosted_operator(
                        &config_file_path()?,
                        &org_alias,
                        saved.checkpoint.organization_id(),
                        &saved.api_base_url,
                        intent.operator_name,
                        make_default,
                        accepted.into_parts(),
                    )
                    .await
                    .map(Outcome::OperatorCreated)
                    .with_context(|| operation.recovery())
                }
                Creation::Quorum { intent } => {
                    let count = intent.operator_encrypt_keys.len();
                    let expected = QuorumExpectation::parse(
                        saved.checkpoint.organization_id().to_owned(),
                        intent.threshold,
                        intent.operator_encrypt_keys,
                    )?;
                    let (result, report) = verifier
                        .accept_quorum_reported(&auth.client, &expected, receipt)
                        .await
                        .into_parts();
                    crate::proofs::present(ctx.shell(), &result, &report, verbose_proofs);
                    let accepted = result
                        .context("proof acceptance failed; no keys were accepted")
                        .with_context(|| operation.recovery())?;
                    let (quorum_key_id, quorum_public_key, share_ids) = accepted.into_parts();
                    validate_result(
                        CreateTvcQuorumKeyResult {
                            quorum_key_id,
                            quorum_public_key,
                            share_ids,
                        },
                        count,
                    )
                    .map(Outcome::QuorumKeyCreated)
                    .with_context(|| operation.recovery())
                }
            }
        }
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct CreationPending {
    operation_id: Uuid,
    activity_id: Option<String>,
    status: String,
}

impl Display for CreationPending {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(activity_id) = &self.activity_id {
            writeln!(
                f,
                "Activity ID (untrusted lookup hint): {}",
                TerminalText(activity_id).single_line()
            )?;
        }
        write!(
            f,
            "Creation is pending ({}). No keys have been accepted.\nResume with: tvc creation resume {}",
            TerminalText(&self.status).single_line(),
            self.operation_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ColorChoice, MessageFormat, Shell};
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;
    use turnkey_client::{TurnkeyClient, TurnkeyP256ApiKey};
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{method, path},
    };

    const ID: &str = "11111111-1111-4111-8111-111111111111";
    const API: &str = "https://api.turnkey.com";

    fn context() -> StdCtx {
        StdCtx::new(
            Shell::standard(MessageFormat::Json, ColorChoice::Never),
            true,
        )
    }

    fn auth(server: &MockServer) -> AuthenticatedClient {
        AuthenticatedClient {
            client: TurnkeyClient::builder()
                .api_key(TurnkeyP256ApiKey::generate())
                .base_url(server.uri())
                .build()
                .unwrap()
                .with_app_proofs(),
            org_id: "org-id".into(),
            api_base_url: API.into(),
        }
    }

    fn creation(quorum: bool) -> Creation {
        if quorum {
            let public = qos_p256::P256Pair::generate()
                .unwrap()
                .public_key()
                .to_bytes();
            Creation::Quorum {
                intent: CreateTvcQuorumKeyIntent {
                    threshold: 2,
                    operator_encrypt_keys: vec![
                        hex::encode(&public[..65]),
                        hex::encode(&public[65..]),
                    ],
                },
            }
        } else {
            Creation::Operator {
                intent: CreateTvcOperatorIntent {
                    operator_name: "original operator".into(),
                    wallet_name: Some("original wallet".into()),
                    wallet_id: None,
                    path: "m/0".into(),
                },
                org_alias: "original alias".into(),
                make_default: true,
            }
        }
    }

    fn checkpoint(quorum: bool) -> SubmissionCheckpoint {
        serde_json::from_value(json!({
            "version":1, "organization_id":"org-id",
            "kind": if quorum { "CreateTvcQuorumKey" } else { "CreateTvcOperator" },
            "fingerprints":["ab".repeat(32)], "activity_id":"activity-a"
        }))
        .unwrap()
    }

    fn saved(quorum: bool) -> SavedCreation {
        SavedCreation {
            version: 1,
            api_base_url: API.into(),
            timestamp_ms: "100".into(),
            creation: creation(quorum),
            checkpoint: checkpoint(quorum),
        }
    }

    fn response(quorum: bool, id: &str, status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({"activity":{
            "id":id, "organizationId":"org-id", "status":status, "fingerprint":"untrusted-server-fingerprint",
            "type": if quorum { "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY" } else { "ACTIVITY_TYPE_CREATE_TVC_OPERATOR" },
            "appProofs":[], "result": if quorum {
                json!({"createTvcQuorumKeyResult":{"quorumKeyId":"quorum-id", "quorumPublicKey":"untrusted-key", "shareIds":["share-a","share-b"]}})
            } else {
                json!({"createTvcOperatorResult":{"walletId":"wallet-id", "operatorId":"operator-id", "encryptPublicKey":"untrusted-encrypt", "signPublicKey":"untrusted-sign"}})
            }
        }}))
    }

    #[test]
    fn saved_records_roundtrip_privately_and_locks_survive_record_replacement() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        let original = saved(true);
        operation.save(&original).unwrap();
        operation.save(&original).unwrap();
        assert_eq!(
            serde_json::to_value(operation.load().unwrap()).unwrap(),
            serde_json::to_value(original).unwrap()
        );
        assert_eq!(
            std::fs::metadata(&operation.path)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(directory.path())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert!(Operation::at(directory.path(), ID.parse().unwrap()).is_err());
        let other = Operation::at(
            directory.path(),
            "22222222-2222-4222-8222-222222222222".parse().unwrap(),
        )
        .unwrap();
        other.save(&saved(false)).unwrap();
        drop(operation);
        assert!(Operation::at(directory.path(), ID.parse().unwrap()).is_ok());
    }

    #[test]
    fn malformed_and_unsupported_records_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        operation.save(&saved(false)).unwrap();
        let bad_version = {
            let mut value = serde_json::to_value(saved(false)).unwrap();
            value["version"] = json!(2);
            value
        };
        let bad_checkpoint = {
            let mut value = serde_json::to_value(saved(false)).unwrap();
            value["checkpoint"]["fingerprints"] = json!([]);
            value
        };
        let bad_timestamp = {
            let mut value = serde_json::to_value(saved(false)).unwrap();
            value["timestamp_ms"] = json!("invalid");
            value
        };
        for (bad, expected) in [
            (json!({}), "missing field `version`"),
            (bad_version, "unsupported saved creation version 2"),
            (bad_checkpoint, "invalid submission checkpoint"),
            (bad_timestamp, "invalid saved creation timestamp"),
        ] {
            std::fs::write(&operation.path, serde_json::to_vec(&bad).unwrap()).unwrap();
            let error = operation.load().unwrap_err();
            assert!(
                error
                    .chain()
                    .any(|cause| cause.to_string().contains(expected)),
                "{error:#}"
            );
        }
    }

    #[test]
    fn shared_or_nonregular_records_are_rejected_before_loading_origin_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        operation.save(&saved(false)).unwrap();
        std::fs::set_permissions(&operation.path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(operation.load().is_err());
        std::fs::set_permissions(&operation.path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(operation.load().is_ok());
        let target = directory.path().join("target.json");
        std::fs::rename(&operation.path, &target).unwrap();
        std::os::unix::fs::symlink(&target, &operation.path).unwrap();
        assert!(operation.load().is_err());
        std::fs::remove_file(&operation.path).unwrap();
        std::fs::create_dir(&operation.path).unwrap();
        assert!(operation.load().is_err());
    }

    #[tokio::test]
    async fn failed_returned_hint_save_retains_the_earlier_durable_checkpoint() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        let server = MockServer::start().await;
        let record_directory = directory.path().to_owned();
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/create_tvc_operator"))
            .respond_with(move |_: &Request| {
                std::fs::set_permissions(&record_directory, std::fs::Permissions::from_mode(0o500))
                    .unwrap();
                response(false, "activity-a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")
            })
            .expect(1)
            .mount(&server)
            .await;
        let result = operation
            .submit(&mut context(), auth(&server), creation(false), 100, false)
            .await;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let error = result.err().unwrap();
        assert!(error.chain().any(|cause| cause.is::<std::io::Error>()));
        assert!(error.to_string().contains(ID));
        let saved = operation.load().unwrap();
        assert_eq!(saved.checkpoint.activity_id(), None);
        assert_eq!(saved.checkpoint.sent_fingerprints().count(), 1);
        server.verify().await;
    }

    #[tokio::test]
    async fn both_creations_save_before_post_and_restart_without_reposting() {
        for quorum in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
            let server = MockServer::start().await;
            let record_path = operation.path.clone();
            Mock::given(method("POST"))
                .and(path(if quorum {
                    "/public/v1/submit/create_tvc_quorum_key"
                } else {
                    "/public/v1/submit/create_tvc_operator"
                }))
                .respond_with(move |_: &Request| {
                    let durable: SavedCreation =
                        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
                    assert_eq!(durable.checkpoint.sent_fingerprints().count(), 1);
                    assert_eq!(durable.timestamp_ms, "100");
                    response(quorum, "activity-a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")
                })
                .expect(1)
                .mount(&server)
                .await;
            let outcome = operation
                .submit(&mut context(), auth(&server), creation(quorum), 100, false)
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(outcome).unwrap(),
                json!({
                    "reason":"creation_pending", "operationId":ID, "activityId":"activity-a", "status":"consensus_needed"
                })
            );
            let before = serde_json::to_value(operation.load().unwrap()).unwrap();
            drop(operation);
            Mock::given(method("POST"))
                .and(path("/public/v1/query/get_activity"))
                .respond_with(response(quorum, "activity-b", "ACTIVITY_STATUS_PENDING"))
                .expect(1)
                .mount(&server)
                .await;
            let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
            let client = auth(&server);
            let loaded = operation.load().unwrap().for_client(&client).unwrap();
            let outcome = loaded
                .resume(
                    &mut context(),
                    &client,
                    &operation,
                    Some("activity-b".into()),
                    false,
                )
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(outcome).unwrap()["status"],
                "processing"
            );
            let mut expected = before;
            expected["checkpoint"]["activity_id"] = json!("activity-b");
            assert_eq!(
                serde_json::to_value(operation.load().unwrap()).unwrap(),
                expected
            );
            server.verify().await;
        }
    }

    #[tokio::test]
    async fn storage_failure_keeps_io_cause_and_prevents_the_creation_post() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        std::fs::create_dir(&operation.path).unwrap();
        let server = MockServer::start().await;
        let error = operation
            .submit(&mut context(), auth(&server), creation(false), 100, false)
            .await
            .err()
            .unwrap();
        let failure = error
            .downcast_ref::<CheckpointPersistenceFailure>()
            .unwrap();
        let TurnkeyClientError::SubmissionFailed(submission) = &failure.submission else {
            panic!("expected typed submission failure")
        };
        assert_eq!(
            submission.submission_state(),
            SubmissionState::NeverSubmitted
        );
        assert!(error.chain().any(|cause| cause.is::<std::io::Error>()));
        assert!(error.to_string().contains(ID));
        assert!(
            error
                .to_string()
                .contains("no remote creation request was sent")
        );
        assert!(!error.to_string().contains("resume with"));
        assert!(server.received_requests().await.unwrap().is_empty());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 2);
    }

    #[tokio::test]
    async fn wrong_organization_and_environment_are_rejected_before_network() {
        let server = MockServer::start().await;
        let mut client = auth(&server);
        client.org_id = "other-org".into();
        assert!(saved(false).for_client(&client).is_err());
        client.org_id = "org-id".into();
        client.api_base_url = "https://api.preprod.turnkey.engineering".into();
        assert!(saved(false).for_client(&client).is_err());
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn initial_and_resumed_completion_reject_missing_proofs_and_retain_the_receipt() {
        for quorum in [false, true] {
            for resumed in [false, true] {
                let directory = tempfile::tempdir().unwrap();
                let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
                let server = MockServer::start().await;
                let endpoint = if resumed {
                    "/public/v1/query/get_activity"
                } else if quorum {
                    "/public/v1/submit/create_tvc_quorum_key"
                } else {
                    "/public/v1/submit/create_tvc_operator"
                };
                Mock::given(method("POST"))
                    .and(path(endpoint))
                    .respond_with(response(quorum, "activity-a", "ACTIVITY_STATUS_COMPLETED"))
                    .expect(1)
                    .mount(&server)
                    .await;
                let client = auth(&server);
                let result = if resumed {
                    operation.save(&saved(quorum)).unwrap();
                    operation
                        .load()
                        .unwrap()
                        .for_client(&client)
                        .unwrap()
                        .resume(&mut context(), &client, &operation, None, false)
                        .await
                } else {
                    operation
                        .submit(&mut context(), client, creation(quorum), 100, false)
                        .await
                };
                let error = result.err().unwrap();
                assert_eq!(
                    crate::errors::classify(&error).code,
                    crate::errors::ErrorCode::ProofVerificationFailed
                );
                assert_eq!(
                    operation.load().unwrap().checkpoint.activity_id(),
                    Some("activity-a")
                );
                server.verify().await;
            }
        }
    }

    #[tokio::test]
    async fn failed_activity_persists_the_updated_candidate_hint() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap()).unwrap();
        operation.save(&saved(false)).unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/get_activity"))
            .respond_with(response(false, "activity-b", "ACTIVITY_STATUS_REJECTED"))
            .expect(1)
            .mount(&server)
            .await;
        let client = auth(&server);
        let error = operation
            .load()
            .unwrap()
            .for_client(&client)
            .unwrap()
            .resume(
                &mut context(),
                &client,
                &operation,
                Some("activity-b".into()),
                false,
            )
            .await
            .err()
            .unwrap();
        assert_eq!(
            crate::errors::classify(&error).code,
            crate::errors::ErrorCode::ApiError
        );
        assert_eq!(
            operation.load().unwrap().checkpoint.activity_id(),
            Some("activity-b")
        );
        server.verify().await;
    }
}
