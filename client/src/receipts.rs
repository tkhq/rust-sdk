use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use turnkey_api_key_stamper::{
    API_KEY_STAMP_HEADER_NAME, RandomizedStamp, SIGNATURE_SCHEME_P256, SIGNATURE_SCHEME_SECP256K1,
    StampHeader,
};

use crate::generated::immutable::models::v1::SignatureScheme;
use crate::generated::{ActivityType, GetActivityRequest, result::Inner};
use crate::{
    Activity, ActivityResponse, ActivityResult, ActivityStatus, Stamp, TurnkeyClient,
    TurnkeyClientError,
};

mod ump_signature;

/// SHA-256 fingerprint of a signed request using TKMS's exact signature encoding.
///
/// Includes the signature, which built-in stampers randomize by default to
/// distinguish separate submissions of the same request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RequestFingerprint([u8; 32]);

impl RequestFingerprint {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    fn new(
        scheme: SignatureScheme,
        public_key: &str,
        message: &str,
        signature: &str,
    ) -> Result<Self, TurnkeyClientError> {
        ump_signature::digest(scheme, public_key, message, signature).map(Self)
    }

    pub(crate) fn from_stamp(
        header: &StampHeader,
        message: &str,
    ) -> Result<Self, TurnkeyClientError> {
        let stamp = ApiStamp::try_from(header)?;
        let scheme = match stamp.scheme.as_str() {
            SIGNATURE_SCHEME_P256 => SignatureScheme::TkApiP256,
            SIGNATURE_SCHEME_SECP256K1 => SignatureScheme::TkApiSecp256k1,
            _ => {
                return Err(TurnkeyClientError::InvalidReceiptStamp(
                    "unsupported signature scheme",
                ));
            }
        };
        Self::new(scheme, &stamp.public_key, message, &stamp.signature)
    }
}

impl From<RequestFingerprint> for String {
    fn from(fingerprint: RequestFingerprint) -> Self {
        fingerprint.to_hex()
    }
}

impl TryFrom<String> for RequestFingerprint {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut bytes = [0; 32];
        hex::decode_to_slice(value, &mut bytes).map_err(|_| "invalid request fingerprint")?;
        Ok(Self(bytes))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
}

impl TryFrom<&StampHeader> for ApiStamp {
    type Error = TurnkeyClientError;

    fn try_from(header: &StampHeader) -> Result<Self, Self::Error> {
        if !header.name.eq_ignore_ascii_case(API_KEY_STAMP_HEADER_NAME) {
            return Err(TurnkeyClientError::InvalidReceiptStamp("expected X-Stamp"));
        }
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(&header.value)
            .map_err(|_| TurnkeyClientError::InvalidReceiptStamp("invalid base64url"))?;
        serde_json::from_slice(&bytes)
            .map_err(|_| TurnkeyClientError::InvalidReceiptStamp("invalid stamp JSON"))
    }
}

/// Whether any creation request could have reached the service during this invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionState {
    NeverSubmitted,
    MayHaveSubmitted,
}

/// A receipt failure cause that excludes request, stamp, and response contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SubmissionFailureCause {
    #[error("randomized signing failed")]
    RandomizedSigningFailed,
    #[error("submission stamp was rejected")]
    StampRejected,
    #[error("request construction failed")]
    InvalidRequest,
    #[error("submission transport failed")]
    Transport,
    #[error("submission response was invalid")]
    InvalidResponse,
    #[error("submission service returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("submission service requires a newer client (HTTP status {0})")]
    ClientVersionTooOld(u16),
    #[error("activity requires approval")]
    ApprovalRequired,
    #[error("submission checkpoint persistence failed")]
    CheckpointSaveFailed,
    #[error("activity did not complete successfully")]
    ActivityNotCompleted,
    #[error("submission retry limit was reached")]
    ExceededRetries,
}

/// Failure of one receipt-bearing invocation, including earlier possible sends.
#[derive(Debug, thiserror::Error)]
#[error("{state}: {cause}")]
pub struct SubmissionFailure {
    state: SubmissionState,
    checkpoint: Option<SubmissionCheckpoint>,
    #[source]
    cause: SubmissionFailureCause,
}

impl std::fmt::Display for SubmissionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NeverSubmitted => "creation was never submitted",
            Self::MayHaveSubmitted => "remote creation may have succeeded",
        })
    }
}

impl SubmissionFailure {
    pub fn submission_state(&self) -> SubmissionState {
        self.state
    }

    /// A bounded, printable routing label supplied by the service, not authenticated output.
    pub fn activity_id(&self) -> Option<&str> {
        self.checkpoint
            .as_ref()
            .and_then(SubmissionCheckpoint::activity_id)
    }

    pub fn checkpoint(&self) -> Option<&SubmissionCheckpoint> {
        self.checkpoint.as_ref()
    }

    pub fn cause(&self) -> &SubmissionFailureCause {
        &self.cause
    }

    fn from_error(
        error: TurnkeyClientError,
        state: SubmissionState,
        checkpoint: Option<SubmissionCheckpoint>,
    ) -> Self {
        use SubmissionFailureCause as Cause;
        let cause = match error {
            TurnkeyClientError::StamperError(
                turnkey_api_key_stamper::StamperError::RandomizedSigningFailed,
            ) => Cause::RandomizedSigningFailed,
            TurnkeyClientError::StamperError(_) | TurnkeyClientError::InvalidReceiptStamp(_) => {
                Cause::StampRejected
            }
            TurnkeyClientError::Http(_) => Cause::Transport,
            TurnkeyClientError::BuilderMissingApiKey
            | TurnkeyClientError::ReqwestBuilder(_)
            | TurnkeyClientError::SerdeJsonFailure(_) => Cause::InvalidRequest,
            TurnkeyClientError::CheckpointSaveFailed(_) => Cause::CheckpointSaveFailed,
            TurnkeyClientError::ActivityRequiresApproval(_) => Cause::ApprovalRequired,
            TurnkeyClientError::ActivityFailed(_)
            | TurnkeyClientError::UnexpectedActivityStatus(_) => Cause::ActivityNotCompleted,
            TurnkeyClientError::UnexpectedHttpStatus(status, body) => {
                let too_old = serde_json::from_str::<serde_json::Value>(&body).is_ok_and(|body| {
                    body.get("turnkeyErrorCode")
                        .and_then(serde_json::Value::as_str)
                        == Some("TVC_CLIENT_VERSION_TOO_OLD")
                });
                if too_old {
                    Cause::ClientVersionTooOld(status)
                } else {
                    Cause::HttpStatus(status)
                }
            }
            TurnkeyClientError::RefusedRedirect(status, _) => Cause::HttpStatus(status),
            TurnkeyClientError::ExceededRetries(_) => Cause::ExceededRetries,
            _ => Cause::InvalidResponse,
        };
        Self {
            state,
            checkpoint,
            cause,
        }
    }
}

/// Sanitized persistence failure. Keep storage-specific diagnostics in the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("submission checkpoint persistence failed")]
pub struct CheckpointSaveError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SubmissionKind {
    CreateTvcOperator,
    CreateTvcQuorumKey,
}

impl SubmissionKind {
    fn activity_type(self) -> ActivityType {
        match self {
            Self::CreateTvcOperator => ActivityType::CreateTvcOperator,
            Self::CreateTvcQuorumKey => ActivityType::CreateTvcQuorumKey,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointData {
    version: u32,
    organization_id: String,
    kind: SubmissionKind,
    fingerprints: Vec<RequestFingerprint>,
    activity_id: Option<String>,
}

/// Local origin evidence to persist before sending each signed creation request.
///
/// Load only caller-owned state. Coordinator-supplied checkpoints cannot establish
/// request origin. Prepared fingerprints do not establish whether delivery occurred.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct SubmissionCheckpoint(CheckpointData);

impl<'de> Deserialize<'de> for SubmissionCheckpoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = CheckpointData::deserialize(deserializer)?;
        if data.version != 1
            || data.organization_id.is_empty()
            || data.fingerprints.is_empty()
            || data
                .activity_id
                .as_deref()
                .is_some_and(|id| safe_activity_id(id).is_none())
        {
            return Err(serde::de::Error::custom("invalid submission checkpoint"));
        }
        Ok(Self(data))
    }
}

impl SubmissionCheckpoint {
    fn submission_error(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::from_error(
            error,
            SubmissionState::MayHaveSubmitted,
            Some(self.clone()),
        ))
    }

    fn with_activity_id(self, id: &str) -> Self {
        Self(CheckpointData {
            activity_id: safe_activity_id(id).or(self.0.activity_id),
            ..self.0
        })
    }

    pub fn organization_id(&self) -> &str {
        &self.0.organization_id
    }

    /// An untrusted lookup hint, never an authenticated activity identifier.
    pub fn activity_id(&self) -> Option<&str> {
        self.0.activity_id.as_deref()
    }

    pub fn sent_fingerprints(&self) -> impl Iterator<Item = &RequestFingerprint> {
        self.0.fingerprints.iter()
    }
}

/// A submission has completed or retains local evidence for a later query.
#[derive(Debug, Clone)]
pub enum SubmissionOutcome<T> {
    Completed(T),
    Pending {
        checkpoint: SubmissionCheckpoint,
        reason: SubmissionPendingReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmissionPendingReason {
    Processing,
    ConsensusNeeded,
    AuthenticatorsNeeded,
}

#[derive(Default)]
struct SubmissionInvocation {
    checkpoint: Option<SubmissionCheckpoint>,
    attempted: bool,
}

impl SubmissionInvocation {
    fn fail(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::from_error(
            error,
            if self.attempted {
                SubmissionState::MayHaveSubmitted
            } else {
                SubmissionState::NeverSubmitted
            },
            self.checkpoint.clone(),
        ))
    }
}

fn safe_activity_id(id: &str) -> Option<String> {
    (!id.is_empty() && id.len() <= 128 && id.chars().all(|c| !c.is_control()))
        .then(|| id.to_owned())
}

/// A completed unverified activity and its original local submission checkpoint.
#[derive(Debug, Clone)]
pub struct ActivityReceipt {
    activity: Activity,
    checkpoint: SubmissionCheckpoint,
}

impl ActivityReceipt {
    pub(crate) fn submission_error(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        self.checkpoint.submission_error(error)
    }

    pub fn activity(&self) -> &Activity {
        &self.activity
    }

    pub fn checkpoint(&self) -> &SubmissionCheckpoint {
        &self.checkpoint
    }

    pub fn sent_fingerprints(&self) -> impl Iterator<Item = &RequestFingerprint> {
        self.checkpoint.sent_fingerprints()
    }
}

/// The typed activity result and local submission receipt.
///
/// Receipt methods require a built-in API key with randomized stamping support.
///
/// Example:
/// ```compile_fail
/// use turnkey_api_key_stamper::{Stamp, StampHeader, StamperError};
/// use turnkey_client::{TurnkeyClient, generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent}};
/// struct Custom;
/// impl Stamp for Custom {
///     fn stamp(&self, _: &[u8]) -> Result<StampHeader, StamperError> {
///         Ok(StampHeader { name: "Custom".into(), value: "stamp".into() })
///     }
/// }
/// async fn unsupported(client: TurnkeyClient<Custom>, operator: CreateTvcOperatorIntent, quorum: CreateTvcQuorumKeyIntent) {
///     client.create_tvc_operator_with_receipt("org".into(), 100, operator, async |_| Ok(())).await;
///     client.create_tvc_quorum_key_with_receipt("org".into(), 100, quorum, async |_| Ok(())).await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SubmittedActivityResult<T> {
    result: ActivityResult<T>,
    receipt: ActivityReceipt,
}

impl<T> SubmittedActivityResult<T> {
    pub(crate) fn new(result: ActivityResult<T>, receipt: ActivityReceipt) -> Self {
        Self { result, receipt }
    }

    pub fn activity_result(&self) -> &ActivityResult<T> {
        &self.result
    }

    pub fn receipt(&self) -> &ActivityReceipt {
        &self.receipt
    }

    pub fn into_parts(self) -> (ActivityResult<T>, ActivityReceipt) {
        (self.result, self.receipt)
    }
}

impl<S: RandomizedStamp> TurnkeyClient<S> {
    pub(crate) async fn process_activity_with_receipt<Request: Serialize>(
        &self,
        request: Request,
        path: String,
        organization_id: String,
        kind: SubmissionKind,
        mut save: impl AsyncFnMut(&SubmissionCheckpoint) -> Result<(), CheckpointSaveError>,
    ) -> Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError> {
        let mut submission = SubmissionInvocation::default();
        let mut retry_count = 0;
        if organization_id.is_empty() {
            return Err(
                submission.fail(TurnkeyClientError::InvalidSubmissionActivity(
                    "empty organization",
                )),
            );
        }
        loop {
            let signed = self
                .prepare_request(&request)
                .map_err(|error| submission.fail(error))?;
            let fingerprint = RequestFingerprint::from_stamp(&signed.stamp, &signed.body)
                .map_err(|error| submission.fail(error))?;
            let request = self
                .build_request(&path, signed)
                .map_err(|error| submission.fail(error))?;
            let candidate = SubmissionCheckpoint(CheckpointData {
                version: 1,
                organization_id: organization_id.clone(),
                kind,
                fingerprints: submission
                    .checkpoint
                    .iter()
                    .flat_map(|checkpoint| checkpoint.sent_fingerprints().copied())
                    .chain(std::iter::once(fingerprint))
                    .collect(),
                activity_id: submission
                    .checkpoint
                    .as_ref()
                    .and_then(|checkpoint| checkpoint.0.activity_id.clone()),
            });
            save(&candidate)
                .await
                .map_err(|error| submission.fail(error.into()))?;
            let checkpoint = candidate;
            submission.attempted = true;
            let response: ActivityResponse = self
                .execute_request(request)
                .await
                .map_err(|error| checkpoint.submission_error(error))?;
            let activity = response
                .activity
                .ok_or_else(|| checkpoint.submission_error(TurnkeyClientError::MissingActivity))?;
            let checkpoint = checkpoint.with_activity_id(&activity.id);
            let reason = match activity.status {
                ActivityStatus::Completed => {
                    return Ok(SubmissionOutcome::Completed(ActivityReceipt {
                        activity,
                        checkpoint,
                    }));
                }
                ActivityStatus::Created => SubmissionPendingReason::Processing,
                ActivityStatus::Pending => {
                    if retry_count < self.retry_config.max_retries {
                        submission.checkpoint = Some(checkpoint);
                        retry_count += 1;
                        tokio::time::sleep(self.retry_config.compute_delay(retry_count)).await;
                        continue;
                    }
                    SubmissionPendingReason::Processing
                }
                ActivityStatus::ConsensusNeeded => SubmissionPendingReason::ConsensusNeeded,
                ActivityStatus::AuthenticatorsNeeded => {
                    SubmissionPendingReason::AuthenticatorsNeeded
                }
                ActivityStatus::Failed | ActivityStatus::Rejected => {
                    return Err(checkpoint
                        .submission_error(TurnkeyClientError::ActivityFailed(activity.failure)));
                }
                ActivityStatus::Unspecified => {
                    return Err(checkpoint.submission_error(
                        TurnkeyClientError::InvalidSubmissionActivity("unspecified status"),
                    ));
                }
            };
            return Ok(SubmissionOutcome::Pending { checkpoint, reason });
        }
    }
}

impl<S: Stamp> TurnkeyClient<S> {
    /// Queries once using saved local origin evidence. Never resubmits creation.
    ///
    /// A candidate ID overrides the untrusted saved hint. The returned receipt and
    /// proofs remain unverified. Persist updated hints from Pending outcomes.
    pub async fn resume_submission(
        &self,
        checkpoint: SubmissionCheckpoint,
        candidate_id: Option<String>,
    ) -> Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError> {
        let activity_id = match candidate_id.as_deref() {
            Some(id) => safe_activity_id(id),
            None => checkpoint.activity_id().map(str::to_owned),
        }
        .ok_or_else(|| {
            checkpoint.submission_error(TurnkeyClientError::InvalidSubmissionActivity(
                "missing or invalid activity ID",
            ))
        })?;
        let response = self
            .get_activity(GetActivityRequest {
                organization_id: checkpoint.organization_id().to_owned(),
                activity_id: activity_id.clone(),
            })
            .await
            .map_err(|error| checkpoint.submission_error(error))?;
        let activity = response
            .activity
            .ok_or_else(|| checkpoint.submission_error(TurnkeyClientError::MissingActivity))?;
        if activity.id != activity_id
            || activity.organization_id != checkpoint.organization_id()
            || activity.r#type != checkpoint.0.kind.activity_type()
        {
            return Err(checkpoint.submission_error(
                TurnkeyClientError::InvalidSubmissionActivity("activity identity mismatch"),
            ));
        }
        let checkpoint = SubmissionCheckpoint(CheckpointData {
            activity_id: Some(activity_id),
            ..checkpoint.0
        });
        let reason = match activity.status {
            ActivityStatus::Completed => {
                match (
                    checkpoint.0.kind,
                    activity
                        .result
                        .as_ref()
                        .and_then(|result| result.inner.as_ref()),
                ) {
                    (
                        SubmissionKind::CreateTvcOperator,
                        Some(Inner::CreateTvcOperatorResult(_)),
                    )
                    | (
                        SubmissionKind::CreateTvcQuorumKey,
                        Some(Inner::CreateTvcQuorumKeyResult(_)),
                    ) => (),
                    _ => {
                        return Err(checkpoint.submission_error(
                            TurnkeyClientError::InvalidSubmissionActivity(
                                "activity result mismatch",
                            ),
                        ));
                    }
                }
                return Ok(SubmissionOutcome::Completed(ActivityReceipt {
                    activity,
                    checkpoint,
                }));
            }
            ActivityStatus::Created | ActivityStatus::Pending => {
                SubmissionPendingReason::Processing
            }
            ActivityStatus::ConsensusNeeded => SubmissionPendingReason::ConsensusNeeded,
            ActivityStatus::AuthenticatorsNeeded => SubmissionPendingReason::AuthenticatorsNeeded,
            ActivityStatus::Failed | ActivityStatus::Rejected => {
                return Err(checkpoint
                    .submission_error(TurnkeyClientError::ActivityFailed(activity.failure)));
            }
            ActivityStatus::Unspecified => {
                return Err(checkpoint.submission_error(
                    TurnkeyClientError::InvalidSubmissionActivity("unspecified status"),
                ));
            }
        };
        Ok(SubmissionOutcome::Pending { checkpoint, reason })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RetryConfig, TurnkeyP256ApiKey, TurnkeySecp256k1ApiKey};
    use serde::{Serialize, Serializer};
    use serde_json::{Value, json};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    use std::time::Duration;
    use turnkey_api_key_stamper::{Stamp, StamperError};
    use wiremock::{
        Mock, MockServer, Request, Respond, ResponseTemplate,
        matchers::{method, path},
    };

    impl<T: std::fmt::Debug> SubmissionOutcome<T> {
        fn completed(self) -> T {
            match self {
                Self::Completed(value) => value,
                other => panic!("expected completion, got {other:?}"),
            }
        }
    }

    impl<S: RandomizedStamp> TurnkeyClient<S> {
        async fn submit<Request: Serialize>(
            &self,
            request: Request,
            path: String,
        ) -> Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError> {
            self.process_activity_with_receipt(
                request,
                path,
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |_| Ok(()),
            )
            .await
        }
    }

    fn response(status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({"activity": {
            "id":"synthetic-activity", "organizationId":"synthetic-org", "fingerprint":"server-value-is-untrusted",
            "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":status
        }}))
    }

    fn completed() -> ResponseTemplate {
        response("ACTIVITY_STATUS_COMPLETED")
    }

    fn client<S: Stamp>(server: &MockServer, key: S) -> TurnkeyClient<S> {
        TurnkeyClient::builder()
            .api_key(key)
            .base_url(server.uri())
            .retry_config(RetryConfig {
                initial_delay: Duration::from_millis(1),
                multiplier: 1.0,
                max_delay: Duration::from_millis(1),
                max_retries: 3,
            })
            .build()
            .unwrap()
    }

    fn captured(request: &Request) -> RequestFingerprint {
        let header = StampHeader {
            name: "x-stamp".into(),
            value: request
                .headers
                .get("x-stamp")
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        };
        RequestFingerprint::from_stamp(&header, std::str::from_utf8(&request.body).unwrap())
            .unwrap()
    }

    #[tokio::test]
    async fn consensus_needed_retains_submission_for_resume() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response("ACTIVITY_STATUS_CONSENSUS_NEEDED"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let result = client
            .process_activity_with_receipt(
                json!({"timestampMs":"100"}),
                "/submit".into(),
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |_| Ok(()),
            )
            .await;
        let SubmissionOutcome::Pending { checkpoint, reason } = result.unwrap() else {
            panic!("expected a resumable consensus submission")
        };
        assert_eq!(reason, SubmissionPendingReason::ConsensusNeeded);
        assert_eq!(checkpoint.activity_id(), Some("synthetic-activity"));
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            checkpoint.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(&requests[0])]
        );
    }

    #[tokio::test]
    async fn persistence_failure_forbids_the_attempt_and_retains_previous_evidence() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response("ACTIVITY_STATUS_PENDING"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let first = failure(
            client
                .process_activity_with_receipt(
                    (),
                    "/submit".into(),
                    "synthetic-org".into(),
                    SubmissionKind::CreateTvcOperator,
                    async |_| Err(CheckpointSaveError),
                )
                .await
                .unwrap_err(),
        );
        assert_eq!(first.submission_state(), SubmissionState::NeverSubmitted);
        assert_eq!(first.cause(), &SubmissionFailureCause::CheckpointSaveFailed);
        assert!(first.checkpoint().is_none());
        assert!(server.received_requests().await.unwrap().is_empty());

        let mut saved = Vec::new();
        let later = failure(
            client
                .process_activity_with_receipt(
                    (),
                    "/submit".into(),
                    "synthetic-org".into(),
                    SubmissionKind::CreateTvcOperator,
                    async |checkpoint: &SubmissionCheckpoint| {
                        assert_eq!(server.received_requests().await.unwrap().len(), saved.len());
                        assert_eq!(checkpoint.sent_fingerprints().count(), saved.len() + 1);
                        if saved.is_empty() {
                            saved.push(serde_json::to_vec(checkpoint).unwrap());
                            Ok(())
                        } else {
                            Err(CheckpointSaveError)
                        }
                    },
                )
                .await
                .unwrap_err(),
        );
        assert_eq!(later.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(later.cause(), &SubmissionFailureCause::CheckpointSaveFailed);
        let previous: SubmissionCheckpoint = serde_json::from_slice(&saved[0]).unwrap();
        assert_eq!(
            later
                .checkpoint()
                .unwrap()
                .sent_fingerprints()
                .collect::<Vec<_>>(),
            previous.sent_fingerprints().collect::<Vec<_>>()
        );
        assert_eq!(later.activity_id(), Some("synthetic-activity"));
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            previous.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(&requests[0])]
        );
    }

    #[tokio::test]
    async fn cancellation_while_saving_preserves_local_evidence_without_posting() {
        let server = MockServer::start().await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let mut saved = None;
        let canceled = tokio::time::timeout(
            Duration::from_millis(40),
            client.process_activity_with_receipt(
                (),
                "/submit".into(),
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    saved = Some(serde_json::to_vec(checkpoint).unwrap());
                    std::future::pending::<Result<(), CheckpointSaveError>>().await
                },
            ),
        )
        .await;
        assert!(canceled.is_err());
        let checkpoint: SubmissionCheckpoint = serde_json::from_slice(&saved.unwrap()).unwrap();
        assert_eq!(checkpoint.sent_fingerprints().count(), 1);
        assert!(checkpoint.activity_id().is_none());
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn retries_save_each_signed_attempt_before_post_and_exhaust_as_pending() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response("ACTIVITY_STATUS_PENDING"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let mut saved = Vec::new();
        let outcome = client
            .process_activity_with_receipt(
                (),
                "/submit".into(),
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    assert_eq!(server.received_requests().await.unwrap().len(), saved.len());
                    saved.push(checkpoint.clone());
                    Ok(())
                },
            )
            .await
            .unwrap();
        let SubmissionOutcome::Pending { checkpoint, reason } = outcome else {
            panic!("expected pending")
        };
        assert_eq!(reason, SubmissionPendingReason::Processing);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 4);
        for (index, checkpoint) in saved.iter().enumerate() {
            assert_eq!(
                checkpoint.sent_fingerprints().copied().collect::<Vec<_>>(),
                requests[..=index].iter().map(captured).collect::<Vec<_>>()
            );
        }
        assert_eq!(
            checkpoint.sent_fingerprints().copied().collect::<Vec<_>>(),
            requests.iter().map(captured).collect::<Vec<_>>()
        );
        assert_eq!(checkpoint.activity_id(), Some("synthetic-activity"));
    }

    fn query_activity(status: &str) -> Value {
        json!({"activity": {
            "id":"synthetic-activity", "organizationId":"synthetic-org", "fingerprint":"untrusted",
            "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":status,
            "result":{"createTvcOperatorResult":{"walletId":"wallet", "operatorId":"operator", "encryptPublicKey":"encrypt", "signPublicKey":"sign"}}
        }})
    }

    #[tokio::test]
    async fn restart_queries_once_with_original_fingerprints_for_both_supported_kinds() {
        for kind in [
            SubmissionKind::CreateTvcOperator,
            SubmissionKind::CreateTvcQuorumKey,
        ] {
            let server = MockServer::start().await;
            Mock::given(path("/submit"))
                .respond_with(response("ACTIVITY_STATUS_CONSENSUS_NEEDED"))
                .mount(&server)
                .await;
            let mut query = query_activity("ACTIVITY_STATUS_COMPLETED");
            if kind == SubmissionKind::CreateTvcQuorumKey {
                query["activity"]["type"] = "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY".into();
                query["activity"]["result"] = json!({"createTvcQuorumKeyResult": {
                    "quorumKeyId":"key", "quorumPublicKey":"public", "shareIds":["share"]
                }});
            }
            Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(query))
                .expect(2)
                .mount(&server)
                .await;
            let client = client(
                &server,
                TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
            );
            let mut saved = None;
            let pending = client
                .process_activity_with_receipt(
                    (),
                    "/submit".into(),
                    "synthetic-org".into(),
                    kind,
                    async |checkpoint: &SubmissionCheckpoint| {
                        saved = Some(serde_json::to_vec(checkpoint).unwrap());
                        Ok(())
                    },
                )
                .await
                .unwrap();
            let SubmissionOutcome::Pending { checkpoint, .. } = pending else {
                panic!("expected pending")
            };
            let restored: SubmissionCheckpoint = serde_json::from_slice(&saved.unwrap()).unwrap();
            assert!(restored.activity_id().is_none());
            let restored_with_hint =
                serde_json::from_slice(&serde_json::to_vec(&checkpoint).unwrap()).unwrap();
            let resumed = client
                .resume_submission(restored, Some("synthetic-activity".into()))
                .await
                .unwrap()
                .completed();
            let resumed_with_hint = client
                .resume_submission(restored_with_hint, None)
                .await
                .unwrap()
                .completed();
            let requests = server.received_requests().await.unwrap();
            assert_eq!(
                requests
                    .iter()
                    .filter(|request| request.url.path() == "/submit")
                    .count(),
                1
            );
            for receipt in [resumed, resumed_with_hint] {
                assert_eq!(
                    receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
                    [captured(&requests[0])]
                );
                assert_eq!(
                    receipt.checkpoint().activity_id(),
                    Some("synthetic-activity")
                );
                for query in &requests[1..] {
                    assert_ne!(captured(query), captured(&requests[0]));
                    assert_eq!(
                        serde_json::from_slice::<Value>(&query.body).unwrap(),
                        json!({
                            "organizationId":"synthetic-org", "activityId":"synthetic-activity"
                        })
                    );
                }
            }
        }
    }

    #[tokio::test]
    async fn resume_checks_identity_status_and_result_without_creating() {
        let server = MockServer::start().await;
        Mock::given(path("/submit"))
            .respond_with(response("ACTIVITY_STATUS_AUTHENTICATORS_NEEDED"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let outcome = client
            .process_activity_with_receipt(
                (),
                "/submit".into(),
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |_| Ok(()),
            )
            .await
            .unwrap();
        let SubmissionOutcome::Pending { checkpoint, reason } = outcome else {
            panic!("expected pending")
        };
        assert_eq!(reason, SubmissionPendingReason::AuthenticatorsNeeded);
        let without_hint = SubmissionCheckpoint(CheckpointData {
            activity_id: None,
            ..checkpoint.0.clone()
        });
        for candidate in [None, Some("bad\nlabel".into())] {
            let error = failure(
                client
                    .resume_submission(without_hint.clone(), candidate)
                    .await
                    .unwrap_err(),
            );
            assert_eq!(error.cause(), &SubmissionFailureCause::InvalidResponse);
            assert!(error.checkpoint().is_some());
        }
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
        for (status, expected) in [
            (
                "ACTIVITY_STATUS_CREATED",
                Some(SubmissionPendingReason::Processing),
            ),
            (
                "ACTIVITY_STATUS_PENDING",
                Some(SubmissionPendingReason::Processing),
            ),
            (
                "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                Some(SubmissionPendingReason::ConsensusNeeded),
            ),
            (
                "ACTIVITY_STATUS_AUTHENTICATORS_NEEDED",
                Some(SubmissionPendingReason::AuthenticatorsNeeded),
            ),
            ("ACTIVITY_STATUS_FAILED", None),
            ("ACTIVITY_STATUS_REJECTED", None),
            ("ACTIVITY_STATUS_UNSPECIFIED", None),
        ] {
            let guard = Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(query_activity(status)))
                .expect(1)
                .mount_as_scoped(&server)
                .await;
            let outcome = client.resume_submission(checkpoint.clone(), None).await;
            if let Some(expected) = expected {
                let SubmissionOutcome::Pending {
                    checkpoint: retained,
                    reason,
                } = outcome.unwrap()
                else {
                    panic!("expected pending")
                };
                assert_eq!(reason, expected);
                assert_eq!(
                    retained.sent_fingerprints().collect::<Vec<_>>(),
                    checkpoint.sent_fingerprints().collect::<Vec<_>>()
                );
            } else {
                let error = failure(outcome.unwrap_err());
                assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
                assert!(error.checkpoint().is_some());
                assert_eq!(
                    error.cause(),
                    if status == "ACTIVITY_STATUS_UNSPECIFIED" {
                        &SubmissionFailureCause::InvalidResponse
                    } else {
                        &SubmissionFailureCause::ActivityNotCompleted
                    }
                );
            }
            drop(guard);
        }
        for (field, wrong) in [
            ("id", json!("other")),
            ("organizationId", json!("other")),
            ("type", json!("ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY")),
            ("result", json!(null)),
            ("result", json!({})),
            (
                "result",
                json!({"createTvcQuorumKeyResult": {"quorumKeyId":"key", "quorumPublicKey":"public", "shareIds":[]}}),
            ),
        ] {
            let mut activity = query_activity("ACTIVITY_STATUS_COMPLETED");
            activity["activity"][field] = wrong;
            let guard = Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(activity))
                .expect(1)
                .mount_as_scoped(&server)
                .await;
            let error = failure(
                client
                    .resume_submission(checkpoint.clone(), None)
                    .await
                    .unwrap_err(),
            );
            assert_eq!(error.cause(), &SubmissionFailureCause::InvalidResponse);
            assert_eq!(
                error
                    .checkpoint()
                    .unwrap()
                    .sent_fingerprints()
                    .collect::<Vec<_>>(),
                checkpoint.sent_fingerprints().collect::<Vec<_>>()
            );
            drop(guard);
        }
        assert_eq!(
            server
                .received_requests()
                .await
                .unwrap()
                .iter()
                .filter(|request| request.url.path() == "/submit")
                .count(),
            1
        );
    }

    #[test]
    fn checkpoint_deserialization_rejects_missing_or_malformed_origin_evidence() {
        let valid = json!({"version":1, "organization_id":"org", "kind":"CreateTvcOperator",
            "fingerprints":["ab".repeat(32)], "activity_id":null});
        let checkpoint: SubmissionCheckpoint = serde_json::from_value(valid.clone()).unwrap();
        assert_eq!(checkpoint.organization_id(), "org");
        assert_eq!(checkpoint.sent_fingerprints().count(), 1);
        for (field, invalid) in [
            ("version", json!(2)),
            ("kind", json!("Other")),
            ("organization_id", json!("")),
            ("fingerprints", json!([])),
            ("fingerprints", json!(["short"])),
            ("fingerprints", json!(["zz".repeat(32)])),
            ("activity_id", json!("bad\nlabel")),
            ("activity_id", json!("")),
            ("activity_id", json!("x".repeat(129))),
        ] {
            let mut value = valid.clone();
            value[field] = invalid;
            assert!(
                serde_json::from_value::<SubmissionCheckpoint>(value).is_err(),
                "{field}"
            );
        }
        for field in ["version", "organization_id", "kind", "fingerprints"] {
            let mut value = valid.clone();
            value.as_object_mut().unwrap().remove(field);
            assert!(
                serde_json::from_value::<SubmissionCheckpoint>(value).is_err(),
                "{field}"
            );
        }
    }

    #[tokio::test]
    async fn exact_loopback_tuples_for_both_curves_and_equal_body_time() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let p256 = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let secp = client(
            &server,
            TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let body = json!({"timestampMs":"100", "text":"é🔑\"\\\n"});
        let mut receipts = Vec::new();
        receipts.push(
            p256.submit(&body, "/p256".into())
                .await
                .unwrap()
                .completed(),
        );
        receipts.push(
            p256.submit(&body, "/p256".into())
                .await
                .unwrap()
                .completed(),
        );
        receipts.push(
            secp.submit(&body, "/secp".into())
                .await
                .unwrap()
                .completed(),
        );
        receipts.push(
            secp.submit(&body, "/secp".into())
                .await
                .unwrap()
                .completed(),
        );
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 4);
        for (receipt, request) in receipts.iter().zip(&requests) {
            assert_eq!(request.body, serde_json::to_vec(&body).unwrap());
            assert_eq!(
                receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
                [captured(request)]
            );
            assert_eq!(receipt.activity().id, "synthetic-activity");
        }
        assert_ne!(captured(&requests[0]), captured(&requests[1]));
        assert_ne!(captured(&requests[2]), captured(&requests[3]));
    }

    struct PendingTwice(AtomicUsize);
    impl Respond for PendingTwice {
        fn respond(&self, _: &Request) -> ResponseTemplate {
            if self.0.fetch_add(1, Ordering::SeqCst) < 2 {
                response("ACTIVITY_STATUS_PENDING")
            } else {
                completed()
            }
        }
    }

    struct CountSerialize(Arc<AtomicUsize>);
    impl Serialize for CountSerialize {
        fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
            self.0.fetch_add(1, Ordering::SeqCst);
            json!({"timestampMs":"100", "message":"same"}).serialize(serializer)
        }
    }

    #[tokio::test]
    async fn pending_posts_each_serialize_once_and_retain_all_fingerprints() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(PendingTwice(AtomicUsize::new(0)))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let serializations = Arc::new(AtomicUsize::new(0));
        let receipt = client
            .submit(CountSerialize(serializations.clone()), "/submit".into())
            .await
            .unwrap()
            .completed();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(serializations.load(Ordering::SeqCst), 3);
        assert_eq!(requests.len(), 3);
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            requests.iter().map(captured).collect::<Vec<_>>()
        );
        for request in &requests {
            assert_eq!(
                serde_json::from_slice::<Value>(&request.body).unwrap(),
                json!({"timestampMs":"100", "message":"same"})
            );
        }
        assert!(
            requests
                .windows(2)
                .all(|pair| captured(&pair[0]) != captured(&pair[1]))
        );
    }

    #[tokio::test]
    async fn concurrent_invocations_and_unrelated_queries_remain_isolated() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let left = json!({"invocation":"left"});
        let right = json!({"invocation":"right"});
        let query_body = json!({"query":"unrelated"});
        let (left, right, query) = tokio::join!(
            client.submit(left, "/left".into()),
            client.submit(right, "/right".into()),
            client.process_request::<_, Value>(&query_body, "/query".into()),
        );
        query.unwrap();
        let requests = server.received_requests().await.unwrap();
        for (receipt, route) in [
            (left.unwrap().completed(), "/left"),
            (right.unwrap().completed(), "/right"),
        ] {
            let request = requests
                .iter()
                .find(|request| request.url.path() == route)
                .unwrap();
            assert_eq!(
                receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
                [captured(request)]
            );
        }
    }

    #[tokio::test]
    async fn cancellation_timeout_and_failed_status_produce_no_accepted_receipt() {
        let server = MockServer::start().await;
        Mock::given(path("/slow"))
            .respond_with(completed().set_delay(Duration::from_millis(200)))
            .mount(&server)
            .await;
        Mock::given(path("/ok"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let mut saved = None;
        let result = tokio::time::timeout(
            Duration::from_millis(40),
            client.process_activity_with_receipt(
                (),
                "/slow".into(),
                "synthetic-org".into(),
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    saved = Some(serde_json::to_vec(checkpoint).unwrap());
                    Ok(())
                },
            ),
        )
        .await;
        assert!(result.is_err());
        let receipt = client.submit((), "/ok".into()).await.unwrap().completed();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        let canceled: SubmissionCheckpoint = serde_json::from_slice(&saved.unwrap()).unwrap();
        assert_eq!(
            canceled.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(
                requests.iter().find(|r| r.url.path() == "/slow").unwrap()
            )]
        );
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(
                requests.iter().find(|r| r.url.path() == "/ok").unwrap()
            )]
        );
        let short = TurnkeyClient::builder()
            .api_key(TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap())
            .base_url(server.uri())
            .timeout(Duration::from_millis(40))
            .build()
            .unwrap();
        assert!(matches!(short.submit((), "/slow".into()).await,
                Err(TurnkeyClientError::SubmissionFailed(e))
                    if e.submission_state() == SubmissionState::MayHaveSubmitted
                        && e.cause() == &SubmissionFailureCause::Transport));
        Mock::given(path("/failed"))
            .respond_with(response("ACTIVITY_STATUS_FAILED"))
            .mount(&server)
            .await;
        assert!(matches!(
            client
                .submit((), "/failed".into())
                .await,
            Err(TurnkeyClientError::SubmissionFailed(e))
                if e.submission_state() == SubmissionState::MayHaveSubmitted
                    && e.cause() == &SubmissionFailureCause::ActivityNotCompleted
        ));
    }

    #[test]
    fn strict_stamp_parsing_and_exact_text() {
        let key = TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap();
        let body = "é🔑\"\\\n";
        let valid = key.stamp(body.as_bytes()).unwrap();
        let original = RequestFingerprint::from_stamp(&valid, body).unwrap();
        let mut json: Value =
            serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(&valid.value).unwrap()).unwrap();
        json["publicKey"] = json["publicKey"].as_str().unwrap().to_uppercase().into();
        json["signature"] = json["signature"].as_str().unwrap().to_uppercase().into();
        let header = |value: &Value| StampHeader {
            name: "x-sTaMp".into(),
            value: BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(value).unwrap()),
        };
        assert_ne!(
            RequestFingerprint::from_stamp(&header(&json), body).unwrap(),
            original
        );
        assert_ne!(
            RequestFingerprint::from_stamp(&valid, "substitution").unwrap(),
            original
        );
        for value in [
            json!({}),
            json!({"scheme":"OTHER","publicKey":"00","signature":"00"}),
            json!({"scheme":null,"publicKey":"00","signature":"00"}),
        ] {
            assert!(RequestFingerprint::from_stamp(&header(&value), body).is_err());
        }
        for malformed in [
            StampHeader {
                name: "custom".into(),
                value: valid.value.clone(),
            },
            StampHeader {
                name: "X-Stamp".into(),
                value: "bad!".into(),
            },
        ] {
            assert!(RequestFingerprint::from_stamp(&malformed, body).is_err());
        }
        let duplicate = StampHeader {
            name: "X-Stamp".into(),
            value: BASE64_URL_SAFE_NO_PAD
                .encode(br#"{"scheme":"a","scheme":"b","publicKey":"00","signature":"00"}"#),
        };
        assert!(RequestFingerprint::from_stamp(&duplicate, body).is_err());
    }

    #[tokio::test]
    async fn generated_variants_share_ordinary_request_and_typed_result_extraction() {
        use crate::generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent};
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/submit/create_tvc_operator")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
            "id":"operator-activity", "organizationId":"org", "fingerprint":"server", "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":"ACTIVITY_STATUS_COMPLETED",
            "result":{"createTvcOperatorResult":{"walletId":"wallet", "operatorId":"operator", "encryptPublicKey":"encrypt", "signPublicKey":"sign"}}
        }}))).mount(&server).await;
        Mock::given(path("/public/v1/submit/create_tvc_quorum_key")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
            "id":"quorum-activity", "organizationId":"org", "fingerprint":"server", "type":"ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY", "status":"ACTIVITY_STATUS_COMPLETED",
            "result":{"createTvcQuorumKeyResult":{"quorumKeyId":"quorum", "quorumPublicKey":"key", "shareIds":["share"]}}
        }}))).mount(&server).await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        )
        .with_app_proofs();
        let params = CreateTvcOperatorIntent {
            wallet_id: None,
            wallet_name: Some("wallet".into()),
            path: "p".into(),
            operator_name: "operator".into(),
        };
        let ordinary = client
            .create_tvc_operator("org".into(), 100, params.clone())
            .await
            .unwrap();
        let submitted = client
            .create_tvc_operator_with_receipt("org".into(), 100, params, async |_| Ok(()))
            .await
            .unwrap()
            .completed();
        assert_eq!(ordinary.result, submitted.activity_result().result);
        assert_eq!(submitted.receipt().activity().id, ordinary.activity_id);
        assert_eq!(
            submitted.receipt().checkpoint().0.kind,
            SubmissionKind::CreateTvcOperator
        );
        let params = CreateTvcQuorumKeyIntent {
            threshold: 2,
            operator_encrypt_keys: vec!["one".into(), "two".into()],
        };
        let ordinary = client
            .create_tvc_quorum_key("org".into(), 100, params.clone())
            .await
            .unwrap();
        let (submitted, receipt) = client
            .create_tvc_quorum_key_with_receipt("org".into(), 100, params, async |_| Ok(()))
            .await
            .unwrap()
            .completed()
            .into_parts();
        assert_eq!(ordinary.result, submitted.result);
        assert_eq!(receipt.activity().id, ordinary.activity_id);
        assert_eq!(
            receipt.checkpoint().0.kind,
            SubmissionKind::CreateTvcQuorumKey
        );
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].body, requests[1].body);
        assert_eq!(requests[2].body, requests[3].body);
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(&requests[3])]
        );
        for request in &requests {
            assert_eq!(
                request.headers["x-tvc-client-version"],
                env!("CARGO_PKG_VERSION")
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&request.body).unwrap()["generateAppProofs"],
                true
            );
        }
    }

    fn failure(error: TurnkeyClientError) -> SubmissionFailure {
        match error {
            TurnkeyClientError::SubmissionFailed(failure) => failure,
            other => panic!("expected submission failure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn receipt_errors_preserve_submission_state_and_hide_response_contents() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(503).set_body_json(json!({"secret":"stamp-body-secret"})),
            )
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = failure(client.submit((), "/submit".into()).await.unwrap_err());
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::HttpStatus(503));
        assert_eq!(error.activity_id(), None);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            error
                .checkpoint()
                .unwrap()
                .sent_fingerprints()
                .copied()
                .collect::<Vec<_>>(),
            [captured(&requests[0])]
        );
        assert!(!format!("{error:?} {error}").contains("stamp-body-secret"));
        let mut source = std::error::Error::source(&error);
        while let Some(error) = source {
            assert!(!format!("{error:?} {error}").contains("stamp-body-secret"));
            source = error.source();
        }
    }

    #[tokio::test]
    async fn generated_extraction_failure_retains_possible_submission_and_safe_label() {
        use crate::generated::CreateTvcOperatorIntent;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = failure(
            client
                .create_tvc_operator_with_receipt(
                    "org".into(),
                    100,
                    CreateTvcOperatorIntent {
                        wallet_id: None,
                        wallet_name: Some("wallet".into()),
                        path: "path".into(),
                        operator_name: "operator".into(),
                    },
                    async |_| Ok(()),
                )
                .await
                .unwrap_err(),
        );
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::InvalidResponse);
        assert_eq!(error.activity_id(), Some("synthetic-activity"));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[test]
    fn diagnostic_activity_labels_are_bounded_and_printable() {
        assert_eq!(
            safe_activity_id("untrusted [label] é"),
            Some("untrusted [label] é".into())
        );
        assert_eq!(safe_activity_id("bad\nlabel"), None);
        assert_eq!(safe_activity_id(&"x".repeat(129)), None);
        assert_eq!(safe_activity_id(""), None);
    }

    #[test]
    fn signing_failure_mapping_preserves_submission_state_without_sensitive_details() {
        for state in [
            SubmissionState::NeverSubmitted,
            SubmissionState::MayHaveSubmitted,
        ] {
            let error = SubmissionFailure::from_error(
                TurnkeyClientError::StamperError(StamperError::RandomizedSigningFailed),
                state,
                None,
            );
            assert_eq!(error.submission_state(), state);
            assert_eq!(
                error.cause(),
                &SubmissionFailureCause::RandomizedSigningFailed
            );
            assert_eq!(error.activity_id(), None);
            assert_eq!(
                error.to_string(),
                format!("{state}: randomized signing failed")
            );

            let rejected = SubmissionFailure::from_error(
                TurnkeyClientError::StamperError(StamperError::InvalidPublicKeyBytes(
                    "private request secret".into(),
                )),
                state,
                None,
            );
            assert_eq!(rejected.submission_state(), state);
            assert_eq!(rejected.cause(), &SubmissionFailureCause::StampRejected);
            assert!(!format!("{rejected:?} {rejected}").contains("private request secret"));
        }
    }

    struct FailsAfterFirstSerialization(AtomicUsize);
    impl Serialize for FailsAfterFirstSerialization {
        fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                json!({"timestampMs":"100"}).serialize(serializer)
            } else {
                Err(serde::ser::Error::custom("private request secret"))
            }
        }
    }

    #[tokio::test]
    async fn pending_then_serialization_failure_retains_prior_submission() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response("ACTIVITY_STATUS_PENDING"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = failure(
            client
                .submit(
                    FailsAfterFirstSerialization(AtomicUsize::new(0)),
                    "/submit".into(),
                )
                .await
                .unwrap_err(),
        );
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.activity_id(), Some("synthetic-activity"));
        assert_eq!(error.cause(), &SubmissionFailureCause::InvalidRequest);
        assert!(!format!("{error:?} {error}").contains("private request secret"));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn all_requests_include_sdk_version_without_changing_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        for proof_flag in [true, false] {
            for route in [
                "/public/v1/submit/create_tvc_operator",
                "/public/v1/submit/create_tvc_quorum_key",
            ] {
                let body = json!({"generateAppProofs": proof_flag});
                client.process_activity(&body, route.into()).await.unwrap();
            }
        }
        client
            .process_activity((), "/unrelated".into())
            .await
            .unwrap();
        client
            .get_activity(GetActivityRequest {
                organization_id: "synthetic-org".into(),
                activity_id: "synthetic-activity".into(),
            })
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 6);
        for request in &requests {
            assert_eq!(
                request.headers.get("x-tvc-client-version").unwrap(),
                env!("CARGO_PKG_VERSION")
            );
        }
        for (index, request) in requests[..4].iter().enumerate() {
            assert_eq!(
                serde_json::from_slice::<Value>(&request.body).unwrap(),
                json!({"generateAppProofs": index < 2})
            );
        }
    }
}
