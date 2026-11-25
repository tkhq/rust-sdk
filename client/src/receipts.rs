use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use turnkey_api_key_stamper::{
    API_KEY_STAMP_HEADER_NAME, SIGNATURE_SCHEME_P256, SIGNATURE_SCHEME_SECP256K1, StampHeader,
    UniqueStamp,
};
use uuid::Uuid;

use crate::generated::immutable::models::v1::SignatureScheme;
use crate::generated::{ActivityType, GetActivityRequest, result::Inner};
use crate::{
    Activity, ActivityResponse, ActivityResult, ActivityStatus, Stamp, TurnkeyClient,
    TurnkeyClientError,
};

mod ump_signature;

const SUBMISSION_CHECKPOINT_VERSION: u32 = 1;

/// SHA-256 fingerprint of a signed request using TKMS's exact signature encoding.
///
/// Includes the signature, which built-in stampers randomize by default to
/// distinguish separate submissions of the same request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RequestFingerprint([u8; 32]);

impl RequestFingerprint {
    fn new(
        scheme: SignatureScheme,
        public_key: &str,
        message: &str,
        signature: &str,
    ) -> Result<Self, TurnkeyClientError> {
        ump_signature::digest(scheme, public_key, message, signature).map(Self)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
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
    #[error("unsupported activity type for creation resume")]
    UnsupportedActivityType,
    #[error("returned activity ID does not match the requested activity ID")]
    ActivityIdMismatch,
    #[error("returned activity organization does not match the saved creation organization")]
    ActivityOrganizationMismatch,
    #[error("returned activity type does not match the saved creation type")]
    ActivityTypeMismatch,
    #[error("activity result mismatch")]
    ActivityResultMismatch,
    #[error("submission service returned HTTP status {0}")]
    HttpStatus(u16),
    #[error("submission service requires a newer client (HTTP status {0})")]
    ClientVersionTooOld(u16),
    #[error("activity requires approval")]
    ApprovalRequired,
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

    fn new(
        cause: impl Into<SubmissionFailureCause>,
        state: SubmissionState,
        checkpoint: Option<SubmissionCheckpoint>,
    ) -> Self {
        Self {
            state,
            checkpoint,
            cause: cause.into(),
        }
    }
}

impl From<TurnkeyClientError> for SubmissionFailureCause {
    fn from(error: TurnkeyClientError) -> Self {
        match error {
            TurnkeyClientError::StamperError(
                turnkey_api_key_stamper::StamperError::RandomizedSigningFailed,
            ) => Self::RandomizedSigningFailed,
            TurnkeyClientError::StamperError(_) | TurnkeyClientError::InvalidReceiptStamp(_) => {
                Self::StampRejected
            }
            TurnkeyClientError::Http(_) => Self::Transport,
            TurnkeyClientError::BuilderMissingApiKey
            | TurnkeyClientError::ReqwestBuilder(_)
            | TurnkeyClientError::SerdeJsonFailure(_) => Self::InvalidRequest,
            TurnkeyClientError::ActivityRequiresApproval(_) => Self::ApprovalRequired,
            TurnkeyClientError::ActivityFailed(_)
            | TurnkeyClientError::UnexpectedActivityStatus(_) => Self::ActivityNotCompleted,
            TurnkeyClientError::UnexpectedHttpStatus(status, body) => {
                let too_old = serde_json::from_str::<serde_json::Value>(&body).is_ok_and(|body| {
                    body.get("turnkeyErrorCode")
                        .and_then(serde_json::Value::as_str)
                        == Some("TVC_CLIENT_VERSION_TOO_OLD")
                });
                if too_old {
                    Self::ClientVersionTooOld(status)
                } else {
                    Self::HttpStatus(status)
                }
            }
            TurnkeyClientError::RefusedRedirect(status, _) => Self::HttpStatus(status),
            TurnkeyClientError::ExceededRetries(_) => Self::ExceededRetries,
            _ => Self::InvalidResponse,
        }
    }
}

/// An error submitting a creation request or saving its checkpoint.
#[derive(Debug, thiserror::Error)]
pub enum ReceiptCreationError<E> {
    #[error("{0}")]
    Client(#[from] TurnkeyClientError),
    /// The current attempt was not sent. After an earlier send, the checkpoint
    /// contains the previous fingerprints and any activity ID returned by the API.
    #[error("{state}: submission checkpoint persistence failed")]
    CheckpointSave {
        #[source]
        source: E,
        state: SubmissionState,
        checkpoint: Option<SubmissionCheckpoint>,
    },
}

impl<E> ReceiptCreationError<E> {
    /// Returns the API's untrusted lookup hint if an earlier attempt received one.
    pub fn activity_id(&self) -> Option<&str> {
        match self {
            Self::Client(TurnkeyClientError::SubmissionFailed(failure)) => failure.activity_id(),
            Self::Client(_) => None,
            Self::CheckpointSave { checkpoint, .. } => checkpoint
                .as_ref()
                .and_then(SubmissionCheckpoint::activity_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SubmissionKind {
    CreateTvcOperator,
    CreateTvcQuorumKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointData {
    version: u32,
    organization_id: Uuid,
    kind: SubmissionKind,
    fingerprints: Vec<RequestFingerprint>,
    activity_id: Option<String>,
}

/// A record of the requests your client signed to create an operator or quorum key.
///
/// Save this checkpoint in storage you control and load it when you resume.
/// The SDK records each request's fingerprint before sending it, so a saved
/// request may never have reached the API. If the API returns an activity ID,
/// use it to look up the result. Use the saved fingerprints to check that the
/// proof matches a request your client signed.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct SubmissionCheckpoint(CheckpointData);

impl<'de> Deserialize<'de> for SubmissionCheckpoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let data = CheckpointData::deserialize(deserializer)?;
        if data.version != SUBMISSION_CHECKPOINT_VERSION
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
    fn submission_error(&self, cause: impl Into<SubmissionFailureCause>) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::new(
            cause,
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

    pub fn organization_id(&self) -> Uuid {
        self.0.organization_id
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

impl<T> SubmissionOutcome<T> {
    /// Extracts the typed result from a completed activity, or passes through
    /// the same checkpoint and reason if the submission is pending.
    pub(crate) fn try_map_completed<U>(
        self,
        map: impl FnOnce(T) -> Result<U, TurnkeyClientError>,
    ) -> Result<SubmissionOutcome<U>, TurnkeyClientError> {
        match self {
            Self::Completed(result) => map(result).map(SubmissionOutcome::Completed),
            Self::Pending { checkpoint, reason } => {
                Ok(SubmissionOutcome::Pending { checkpoint, reason })
            }
        }
    }
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
    fn state(&self) -> SubmissionState {
        if self.attempted {
            SubmissionState::MayHaveSubmitted
        } else {
            SubmissionState::NeverSubmitted
        }
    }

    fn fail(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::new(
            error,
            self.state(),
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
    fn submission_error(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        self.checkpoint.submission_error(error)
    }

    pub fn activity(&self) -> &Activity {
        &self.activity
    }

    /// Consume the receipt into its unverified activity and original submission checkpoint.
    pub fn into_parts(self) -> (Activity, SubmissionCheckpoint) {
        (self.activity, self.checkpoint)
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
/// The creation methods require [`UniqueStamp`]. A stamper that implements only
/// [`Stamp`] cannot create an operator with a receipt.
/// ```compile_fail
/// use turnkey_api_key_stamper::{Stamp, StampHeader, StamperError};
/// use uuid::Uuid;
/// use turnkey_client::{TurnkeyClient, generated::CreateTvcOperatorIntent};
/// struct Custom;
/// impl Stamp for Custom {
///     fn stamp(&self, _: &[u8]) -> Result<StampHeader, StamperError> {
///         Ok(StampHeader { name: "Custom".into(), value: "stamp".into() })
///     }
/// }
/// async fn unsupported(client: TurnkeyClient<Custom>, operator: CreateTvcOperatorIntent) {
///     client.create_tvc_operator_with_receipt(Uuid::from_u128(1), 100, operator, async |_| Ok::<(), std::convert::Infallible>(())).await;
/// }
/// ```
///
/// The same requirement applies to quorum-key creation with a receipt.
/// ```compile_fail
/// # use turnkey_api_key_stamper::{Stamp, StampHeader, StamperError};
/// # use uuid::Uuid;
/// # use turnkey_client::{TurnkeyClient, generated::CreateTvcQuorumKeyIntent};
/// # struct Custom;
/// # impl Stamp for Custom {
/// #     fn stamp(&self, _: &[u8]) -> Result<StampHeader, StamperError> {
/// #         Ok(StampHeader { name: "Custom".into(), value: "stamp".into() })
/// #     }
/// # }
/// async fn unsupported(client: TurnkeyClient<Custom>, quorum: CreateTvcQuorumKeyIntent) {
///     client.create_tvc_quorum_key_with_receipt(Uuid::from_u128(1), 100, quorum, async |_| Ok::<(), std::convert::Infallible>(())).await;
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SubmittedActivityResult<T> {
    result: ActivityResult<T>,
    receipt: ActivityReceipt,
}

impl<T> SubmittedActivityResult<T> {
    pub(crate) fn from_receipt(
        receipt: ActivityReceipt,
        parse_result: impl FnOnce(
            Option<crate::generated::immutable::activity::v1::Result>,
        ) -> Result<T, TurnkeyClientError>,
    ) -> Result<Self, TurnkeyClientError> {
        let activity = receipt.activity();
        let result = parse_result(activity.result.clone())
            .map_err(|error| receipt.submission_error(error))?;
        let result = ActivityResult {
            result,
            activity_id: activity.id.clone(),
            status: activity.status,
            app_proofs: activity.app_proofs.clone(),
        };
        Ok(Self { result, receipt })
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

impl<S: UniqueStamp> TurnkeyClient<S> {
    /// Saves each signed attempt's fingerprint before sending the request.
    pub(crate) async fn process_activity_with_receipt<Request: Serialize, E>(
        &self,
        request: Request,
        path: String,
        organization_id: Uuid,
        kind: SubmissionKind,
        mut save: impl AsyncFnMut(&SubmissionCheckpoint) -> Result<(), E>,
    ) -> Result<SubmissionOutcome<ActivityReceipt>, ReceiptCreationError<E>> {
        let mut submission = SubmissionInvocation::default();
        let mut retry_count = 0;
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
                version: SUBMISSION_CHECKPOINT_VERSION,
                organization_id,
                kind,
                fingerprints: submission
                    .checkpoint
                    .iter()
                    .flat_map(|checkpoint| checkpoint.sent_fingerprints().copied())
                    .chain([fingerprint])
                    .collect(),
                activity_id: submission
                    .checkpoint
                    .as_ref()
                    .and_then(|checkpoint| checkpoint.0.activity_id.clone()),
            });
            if let Err(source) = save(&candidate).await {
                return Err(ReceiptCreationError::CheckpointSave {
                    source,
                    state: submission.state(),
                    checkpoint: submission.checkpoint,
                });
            }
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
                        .submission_error(TurnkeyClientError::ActivityFailed(activity.failure))
                        .into());
                }
                ActivityStatus::Unspecified => {
                    return Err(checkpoint
                        .submission_error(TurnkeyClientError::InvalidSubmissionActivity(
                            "unspecified status",
                        ))
                        .into());
                }
            };
            return Ok(SubmissionOutcome::Pending { checkpoint, reason });
        }
    }
}

impl<S: Stamp> TurnkeyClient<S> {
    /// Queries once using saved local origin evidence. Never resubmits creation.
    ///
    /// An ordinary stamp authenticates the lookup. Proof verification uses the
    /// original creation fingerprints saved in the checkpoint.
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
                organization_id: checkpoint.organization_id().to_string(),
                activity_id: activity_id.clone(),
            })
            .await
            .map_err(|error| checkpoint.submission_error(error))?;
        let activity = response
            .activity
            .ok_or_else(|| checkpoint.submission_error(TurnkeyClientError::MissingActivity))?;
        let kind = match activity.r#type {
            ActivityType::CreateTvcOperator => SubmissionKind::CreateTvcOperator,
            ActivityType::CreateTvcQuorumKey => SubmissionKind::CreateTvcQuorumKey,
            _ => {
                return Err(
                    checkpoint.submission_error(SubmissionFailureCause::UnsupportedActivityType)
                );
            }
        };
        if activity.id != activity_id {
            return Err(checkpoint.submission_error(SubmissionFailureCause::ActivityIdMismatch));
        }
        if Uuid::parse_str(&activity.organization_id).ok() != Some(checkpoint.organization_id()) {
            return Err(
                checkpoint.submission_error(SubmissionFailureCause::ActivityOrganizationMismatch)
            );
        }
        if kind != checkpoint.0.kind {
            return Err(checkpoint.submission_error(SubmissionFailureCause::ActivityTypeMismatch));
        }
        let checkpoint = SubmissionCheckpoint(CheckpointData {
            activity_id: Some(activity_id),
            ..checkpoint.0
        });
        let reason = match activity.status {
            ActivityStatus::Completed => {
                let result = activity.result.as_ref().and_then(|r| r.inner.as_ref());
                if !matches!(
                    (kind, result),
                    (
                        SubmissionKind::CreateTvcOperator,
                        Some(Inner::CreateTvcOperatorResult(_)),
                    ) | (
                        SubmissionKind::CreateTvcQuorumKey,
                        Some(Inner::CreateTvcQuorumKeyResult(_)),
                    )
                ) {
                    return Err(
                        checkpoint.submission_error(SubmissionFailureCause::ActivityResultMismatch)
                    );
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
    use std::{convert::Infallible, io};
    use turnkey_api_key_stamper::{Stamp, StamperError};
    use wiremock::{
        Mock, MockServer, Request, Respond, ResponseTemplate,
        matchers::{method, path},
    };

    const ORGANIZATION_ID: Uuid = uuid::uuid!("3a71f861-e586-45f3-8c03-6cd8b9551a5e");

    impl<T: std::fmt::Debug> SubmissionOutcome<T> {
        fn completed(self) -> T {
            match self {
                Self::Completed(value) => value,
                other => panic!("expected completion, got {other:?}"),
            }
        }
    }

    impl<S: UniqueStamp> TurnkeyClient<S> {
        async fn submit<Request: Serialize>(
            &self,
            request: Request,
            path: String,
        ) -> Result<SubmissionOutcome<ActivityReceipt>, ReceiptCreationError<Infallible>> {
            self.process_activity_with_receipt(
                request,
                path,
                ORGANIZATION_ID,
                SubmissionKind::CreateTvcOperator,
                async |_| Ok::<(), Infallible>(()),
            )
            .await
        }
    }

    fn response(status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({"activity": {
            "id":"synthetic-activity", "organizationId":ORGANIZATION_ID, "fingerprint":"server-value-is-untrusted",
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
        use crate::generated::CreateTvcOperatorIntent;

        let server = MockServer::start().await;
        Mock::given(path("/public/v1/submit/create_tvc_operator"))
            .respond_with(response("ACTIVITY_STATUS_CONSENSUS_NEEDED"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let result = client
            .create_tvc_operator_with_receipt(
                ORGANIZATION_ID,
                100,
                CreateTvcOperatorIntent {
                    wallet_id: None,
                    wallet_name: Some("wallet".into()),
                    path: "path".into(),
                    operator_name: "operator".into(),
                },
                async |_| Ok::<(), Infallible>(()),
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
    async fn first_checkpoint_save_failure_prevents_submission() {
        let server = MockServer::start().await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = client
            .create_tvc_quorum_key_with_receipt(
                ORGANIZATION_ID,
                100,
                crate::generated::CreateTvcQuorumKeyIntent {
                    threshold: 2,
                    operator_encrypt_keys: vec!["one".into(), "two".into()],
                },
                async |_| {
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "test checkpoint path",
                    ))
                },
            )
            .await
            .unwrap_err();
        let ReceiptCreationError::CheckpointSave {
            source,
            state,
            checkpoint,
        } = error
        else {
            panic!("expected a checkpoint save failure")
        };
        assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(source.to_string(), "test checkpoint path");
        assert_eq!(state, SubmissionState::NeverSubmitted);
        assert!(checkpoint.is_none());
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn retry_checkpoint_save_failure_preserves_the_previous_submission() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response("ACTIVITY_STATUS_PENDING"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = client
            .create_tvc_operator_with_receipt(
                ORGANIZATION_ID,
                100,
                crate::generated::CreateTvcOperatorIntent {
                    wallet_id: None,
                    wallet_name: Some("wallet".into()),
                    path: "path".into(),
                    operator_name: "operator".into(),
                },
                async |candidate| {
                    if candidate.sent_fingerprints().count() == 1 {
                        Ok(())
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::StorageFull,
                            "test retry checkpoint",
                        ))
                    }
                },
            )
            .await
            .unwrap_err();
        assert_eq!(error.activity_id(), Some("synthetic-activity"));
        let ReceiptCreationError::CheckpointSave {
            source,
            state,
            checkpoint,
        } = error
        else {
            panic!("expected a checkpoint save failure")
        };
        assert_eq!(source.kind(), io::ErrorKind::StorageFull);
        assert_eq!(source.to_string(), "test retry checkpoint");
        assert_eq!(state, SubmissionState::MayHaveSubmitted);
        let previous = checkpoint.expect("first attempt's checkpoint");
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
                ORGANIZATION_ID,
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    saved = Some(serde_json::to_vec(checkpoint).unwrap());
                    std::future::pending::<Result<(), Infallible>>().await
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
                ORGANIZATION_ID,
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    assert_eq!(server.received_requests().await.unwrap().len(), saved.len());
                    saved.push(checkpoint.clone());
                    Ok::<(), Infallible>(())
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
            "id":"synthetic-activity", "organizationId":ORGANIZATION_ID, "fingerprint":"untrusted",
            "type":"ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status":status,
            "result":{"createTvcOperatorResult":{"walletId":"wallet", "operatorId":"operator", "encryptPublicKey":"encrypt", "signPublicKey":"sign"}}
        }})
    }

    #[tokio::test]
    async fn operator_restart_queries_with_original_fingerprints() {
        assert_restart_queries_with_original_fingerprints(SubmissionKind::CreateTvcOperator).await;
    }

    #[tokio::test]
    async fn quorum_key_restart_queries_with_original_fingerprints() {
        assert_restart_queries_with_original_fingerprints(SubmissionKind::CreateTvcQuorumKey).await;
    }

    async fn assert_restart_queries_with_original_fingerprints(kind: SubmissionKind) {
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
                ORGANIZATION_ID,
                kind,
                async |checkpoint: &SubmissionCheckpoint| {
                    saved = Some(serde_json::to_vec(checkpoint).unwrap());
                    Ok::<(), Infallible>(())
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
        }
        for query in &requests[1..] {
            assert_ne!(captured(query), captured(&requests[0]));
            assert_eq!(
                serde_json::from_slice::<Value>(&query.body).unwrap(),
                json!({
                    "organizationId":ORGANIZATION_ID, "activityId":"synthetic-activity"
                })
            );
        }
    }

    async fn checkpoint_awaiting_authenticators<S: UniqueStamp>(
        client: &TurnkeyClient<S>,
        server: &MockServer,
    ) -> SubmissionCheckpoint {
        Mock::given(path("/submit"))
            .respond_with(response("ACTIVITY_STATUS_AUTHENTICATORS_NEEDED"))
            .expect(1)
            .mount(server)
            .await;
        let outcome = client.submit((), "/submit".into()).await.unwrap();
        let SubmissionOutcome::Pending { checkpoint, reason } = outcome else {
            panic!("expected pending");
        };
        assert_eq!(reason, SubmissionPendingReason::AuthenticatorsNeeded);
        checkpoint
    }

    #[tokio::test]
    async fn resume_without_lookup_id_does_not_query() {
        assert_resume_rejects_unusable_lookup_id(None).await;
    }

    #[tokio::test]
    async fn resume_with_malformed_lookup_id_does_not_query() {
        assert_resume_rejects_unusable_lookup_id(Some("bad\nlabel".into())).await;
    }

    async fn assert_resume_rejects_unusable_lookup_id(candidate: Option<String>) {
        let server = MockServer::start().await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let checkpoint = checkpoint_awaiting_authenticators(&client, &server).await;
        let without_hint = SubmissionCheckpoint(CheckpointData {
            activity_id: None,
            ..checkpoint.0
        });
        let error = failure(
            client
                .resume_submission(without_hint, candidate)
                .await
                .unwrap_err(),
        );
        assert_eq!(error.cause(), &SubmissionFailureCause::InvalidResponse);
        assert!(error.checkpoint().is_some());
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    async fn resume_with_lookup_response(
        response: Value,
    ) -> (
        SubmissionCheckpoint,
        Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError>,
    ) {
        let server = MockServer::start().await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let checkpoint = checkpoint_awaiting_authenticators(&client, &server).await;
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .expect(1)
            .mount(&server)
            .await;
        let outcome = client.resume_submission(checkpoint.clone(), None).await;
        assert_eq!(server.received_requests().await.unwrap().len(), 2);
        server.verify().await;
        (checkpoint, outcome)
    }

    async fn assert_resume_pending(status: &str, expected: SubmissionPendingReason) {
        let (original, outcome) = resume_with_lookup_response(query_activity(status)).await;
        let SubmissionOutcome::Pending { checkpoint, reason } = outcome.unwrap() else {
            panic!("expected pending");
        };
        assert_eq!(reason, expected);
        assert_eq!(
            checkpoint.sent_fingerprints().collect::<Vec<_>>(),
            original.sent_fingerprints().collect::<Vec<_>>()
        );
    }

    async fn assert_resume_failure(response: Value, expected: SubmissionFailureCause) {
        let (original, outcome) = resume_with_lookup_response(response).await;
        let error = failure(outcome.unwrap_err());
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &expected);
        assert_eq!(
            error
                .checkpoint()
                .unwrap()
                .sent_fingerprints()
                .collect::<Vec<_>>(),
            original.sent_fingerprints().collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn resume_reports_created_activity_still_processing() {
        assert_resume_pending(
            "ACTIVITY_STATUS_CREATED",
            SubmissionPendingReason::Processing,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_reports_pending_activity_still_processing() {
        assert_resume_pending(
            "ACTIVITY_STATUS_PENDING",
            SubmissionPendingReason::Processing,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_reports_activity_still_needs_consensus() {
        assert_resume_pending(
            "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            SubmissionPendingReason::ConsensusNeeded,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_reports_activity_still_needs_authenticators() {
        assert_resume_pending(
            "ACTIVITY_STATUS_AUTHENTICATORS_NEEDED",
            SubmissionPendingReason::AuthenticatorsNeeded,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_failed_activity() {
        assert_resume_failure(
            query_activity("ACTIVITY_STATUS_FAILED"),
            SubmissionFailureCause::ActivityNotCompleted,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_rejected_activity() {
        assert_resume_failure(
            query_activity("ACTIVITY_STATUS_REJECTED"),
            SubmissionFailureCause::ActivityNotCompleted,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_unspecified_activity_status() {
        assert_resume_failure(
            query_activity("ACTIVITY_STATUS_UNSPECIFIED"),
            SubmissionFailureCause::InvalidResponse,
        )
        .await;
    }

    fn completed_activity_with(field: &str, value: Value) -> Value {
        let mut response = query_activity("ACTIVITY_STATUS_COMPLETED");
        response["activity"][field] = value;
        response
    }

    #[tokio::test]
    async fn resume_rejects_mismatched_activity_id() {
        assert_resume_failure(
            completed_activity_with("id", json!("other")),
            SubmissionFailureCause::ActivityIdMismatch,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_mismatched_organization_id() {
        assert_resume_failure(
            completed_activity_with("organizationId", json!(Uuid::from_u128(2))),
            SubmissionFailureCause::ActivityOrganizationMismatch,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_mismatched_creation_type() {
        assert_resume_failure(
            completed_activity_with("type", json!("ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY")),
            SubmissionFailureCause::ActivityTypeMismatch,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_unsupported_activity_type() {
        assert_resume_failure(
            completed_activity_with("type", json!("ACTIVITY_TYPE_CREATE_WALLET")),
            SubmissionFailureCause::UnsupportedActivityType,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_missing_activity_result() {
        assert_resume_failure(
            completed_activity_with("result", Value::Null),
            SubmissionFailureCause::ActivityResultMismatch,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_empty_activity_result() {
        assert_resume_failure(
            completed_activity_with("result", json!({})),
            SubmissionFailureCause::ActivityResultMismatch,
        )
        .await;
    }

    #[tokio::test]
    async fn resume_rejects_wrong_activity_result_variant() {
        assert_resume_failure(
            completed_activity_with("result", json!({"createTvcQuorumKeyResult": {"quorumKeyId":"key", "quorumPublicKey":"public", "shareIds":[]}})),
            SubmissionFailureCause::ActivityResultMismatch,
        ).await;
    }

    fn checkpoint_json() -> Value {
        json!({"version":1, "organization_id":ORGANIZATION_ID, "kind":"CreateTvcOperator",
            "fingerprints":["ab".repeat(32)], "activity_id":null})
    }

    #[test]
    fn checkpoint_deserializes_with_saved_fingerprint() {
        let checkpoint: SubmissionCheckpoint = serde_json::from_value(checkpoint_json()).unwrap();
        assert_eq!(checkpoint.organization_id(), ORGANIZATION_ID);
        assert_eq!(
            checkpoint
                .sent_fingerprints()
                .map(RequestFingerprint::as_bytes)
                .collect::<Vec<_>>(),
            [&[0xab; 32]]
        );
    }

    fn assert_checkpoint_rejects_field(field: &str, invalid: Value) {
        let mut value = checkpoint_json();
        value[field] = invalid;
        assert!(serde_json::from_value::<SubmissionCheckpoint>(value).is_err());
    }

    fn assert_checkpoint_requires_field(field: &str) {
        let mut value = checkpoint_json();
        value.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<SubmissionCheckpoint>(value).is_err());
    }

    #[test]
    fn checkpoint_rejects_unknown_version() {
        assert_checkpoint_rejects_field("version", json!(2));
    }

    #[test]
    fn checkpoint_rejects_unknown_creation_kind() {
        assert_checkpoint_rejects_field("kind", json!("Other"));
    }

    #[test]
    fn checkpoint_rejects_empty_organization_id() {
        assert_checkpoint_rejects_field("organization_id", json!(""));
    }

    #[test]
    fn checkpoint_rejects_malformed_organization_id() {
        assert_checkpoint_rejects_field("organization_id", json!("not-a-uuid"));
    }

    #[test]
    fn checkpoint_rejects_no_fingerprints() {
        assert_checkpoint_rejects_field("fingerprints", json!([]));
    }

    #[test]
    fn checkpoint_rejects_short_fingerprint() {
        assert_checkpoint_rejects_field("fingerprints", json!(["ab".repeat(31)]));
    }

    #[test]
    fn checkpoint_rejects_non_hex_fingerprint() {
        assert_checkpoint_rejects_field("fingerprints", json!(["zz".repeat(32)]));
    }

    #[test]
    fn checkpoint_rejects_control_character_in_activity_id() {
        assert_checkpoint_rejects_field("activity_id", json!("bad\nlabel"));
    }

    #[test]
    fn checkpoint_rejects_empty_activity_id() {
        assert_checkpoint_rejects_field("activity_id", json!(""));
    }

    #[test]
    fn checkpoint_rejects_oversized_activity_id() {
        assert_checkpoint_rejects_field("activity_id", json!("x".repeat(129)));
    }

    #[test]
    fn checkpoint_requires_version() {
        assert_checkpoint_requires_field("version");
    }

    #[test]
    fn checkpoint_requires_organization_id() {
        assert_checkpoint_requires_field("organization_id");
    }

    #[test]
    fn checkpoint_requires_creation_kind() {
        assert_checkpoint_requires_field("kind");
    }

    #[test]
    fn checkpoint_requires_fingerprints() {
        assert_checkpoint_requires_field("fingerprints");
    }

    #[tokio::test]
    async fn p256_submissions_fingerprint_the_exact_wire_stamp() {
        assert_submissions_fingerprint_the_exact_wire_stamp(
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        )
        .await;
    }

    #[tokio::test]
    async fn secp256k1_submissions_fingerprint_the_exact_wire_stamp() {
        assert_submissions_fingerprint_the_exact_wire_stamp(
            TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap(),
        )
        .await;
    }

    async fn assert_submissions_fingerprint_the_exact_wire_stamp<S: UniqueStamp>(key: S) {
        let server = MockServer::start().await;
        Mock::given(path("/submit"))
            .respond_with(completed())
            .expect(2)
            .mount(&server)
            .await;
        let client = client(&server, key);
        let body = json!({"timestampMs":"100", "text":"é🔑\"\\\n"});
        let first = client
            .submit(&body, "/submit".into())
            .await
            .unwrap()
            .completed();
        let second = client
            .submit(&body, "/submit".into())
            .await
            .unwrap()
            .completed();

        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        for (receipt, request) in [first, second].iter().zip(&requests) {
            assert_eq!(request.body, serde_json::to_vec(&body).unwrap());
            assert_eq!(
                receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
                [captured(request)]
            );
            assert_eq!(receipt.activity().id, "synthetic-activity");
        }
        assert_ne!(captured(&requests[0]), captured(&requests[1]));
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
    async fn cancellation_after_post_preserves_the_sent_fingerprint() {
        let server = MockServer::start().await;
        let received = Arc::new(tokio::sync::Notify::new());
        let signal = Arc::clone(&received);
        Mock::given(path("/submit"))
            .respond_with(move |_: &Request| {
                signal.notify_one();
                completed().set_delay(Duration::from_secs(60))
            })
            .expect(1)
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let mut saved = None;
        {
            let submission = client.process_activity_with_receipt(
                (),
                "/submit".into(),
                ORGANIZATION_ID,
                SubmissionKind::CreateTvcOperator,
                async |checkpoint: &SubmissionCheckpoint| {
                    saved = Some(serde_json::to_vec(checkpoint).unwrap());
                    Ok::<(), Infallible>(())
                },
            );
            tokio::select! {
                result = submission => panic!("submission completed before cancellation: {result:?}"),
                arrival = tokio::time::timeout(Duration::from_secs(5), received.notified()) => {
                    arrival.expect("mock server should receive the POST before cancellation");
                }
            }
        }

        let canceled: SubmissionCheckpoint = serde_json::from_slice(&saved.unwrap()).unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            canceled.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(&requests[0])]
        );
    }

    #[tokio::test]
    async fn request_timeout_preserves_possible_submission() {
        let server = MockServer::start().await;
        Mock::given(path("/submit"))
            .respond_with(completed().set_delay(Duration::from_secs(5)))
            .mount(&server)
            .await;
        let client = TurnkeyClient::builder()
            .api_key(TurnkeySecp256k1ApiKey::from_bytes([1; 32], None).unwrap())
            .base_url(server.uri())
            .timeout(Duration::from_millis(40))
            .build()
            .unwrap();
        let error = failure(client.submit((), "/submit".into()).await.unwrap_err());
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::Transport);
    }

    #[tokio::test]
    async fn failed_activity_preserves_possible_submission() {
        let server = MockServer::start().await;
        Mock::given(path("/submit"))
            .respond_with(response("ACTIVITY_STATUS_FAILED"))
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let error = failure(client.submit((), "/submit".into()).await.unwrap_err());
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::ActivityNotCompleted);
    }

    fn assert_wire_stamp_matches_digest_vector(name: &str) {
        let vectors: Vec<Value> = serde_json::from_str(include_str!(
            "../tests/fixtures/signature_digest_vectors.json"
        ))
        .unwrap();
        let vector = vectors
            .into_iter()
            .find(|vector| vector["name"] == name)
            .unwrap_or_else(|| panic!("missing signature digest fixture {name}"));
        let scheme = match vector["scheme"].as_i64().unwrap() {
            1 => "SIGNATURE_SCHEME_TK_API_P256",
            4 => "SIGNATURE_SCHEME_TK_API_SECP256K1",
            other => panic!("unexpected vector signature scheme {other}"),
        };
        let stamp = json!({
            "scheme": scheme,
            "publicKey": vector["publicKey"],
            "signature": vector["signature"],
        });
        let header = StampHeader {
            name: "X-Stamp".into(),
            value: BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(&stamp).unwrap()),
        };
        let fingerprint =
            RequestFingerprint::from_stamp(&header, vector["message"].as_str().unwrap()).unwrap();
        assert_eq!(
            fingerprint.to_hex(),
            vector["digest"].as_str().unwrap(),
            "{}",
            vector["name"]
        );
    }

    #[test]
    fn p256_stamp_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("p256");
    }

    #[test]
    fn changed_signature_scheme_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("p256-scheme");
    }

    #[test]
    fn changed_public_key_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("p256-publicKey");
    }

    #[test]
    fn changed_request_message_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("p256-message");
    }

    #[test]
    fn changed_signature_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("p256-signature");
    }

    #[test]
    fn captured_p256_stamp_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("captured-0");
    }

    #[test]
    fn captured_secp256k1_stamp_matches_tkms_digest() {
        assert_wire_stamp_matches_digest_vector("captured-2");
    }

    fn stamp_header(value: Value) -> StampHeader {
        StampHeader {
            name: "x-sTaMp".into(),
            value: BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(&value).unwrap()),
        }
    }

    #[test]
    fn fingerprint_rejects_empty_stamp_json() {
        let stamp = stamp_header(json!({}));
        assert!(RequestFingerprint::from_stamp(&stamp, "").is_err());
    }

    #[test]
    fn fingerprint_rejects_unsupported_stamp_scheme() {
        let stamp = stamp_header(json!({"scheme":"OTHER","publicKey":"00","signature":"00"}));
        assert!(RequestFingerprint::from_stamp(&stamp, "").is_err());
    }

    #[test]
    fn fingerprint_rejects_null_stamp_scheme() {
        let stamp = stamp_header(json!({"scheme":null,"publicKey":"00","signature":"00"}));
        assert!(RequestFingerprint::from_stamp(&stamp, "").is_err());
    }

    #[test]
    fn fingerprint_rejects_custom_header_name() {
        let valid = TurnkeyP256ApiKey::from_bytes([1; 32], None)
            .unwrap()
            .stamp(b"body")
            .unwrap();
        let custom = StampHeader {
            name: "custom".into(),
            ..valid
        };
        assert!(RequestFingerprint::from_stamp(&custom, "body").is_err());
    }

    #[test]
    fn fingerprint_rejects_invalid_stamp_base64() {
        let malformed = StampHeader {
            name: "X-Stamp".into(),
            value: "bad!".into(),
        };
        assert!(RequestFingerprint::from_stamp(&malformed, "").is_err());
    }

    #[test]
    fn fingerprint_rejects_duplicate_stamp_fields() {
        let duplicate = StampHeader {
            name: "X-Stamp".into(),
            value: BASE64_URL_SAFE_NO_PAD.encode(
                br#"{
                "scheme":"SIGNATURE_SCHEME_TK_API_P256",
                "scheme":"SIGNATURE_SCHEME_TK_API_SECP256K1",
                "publicKey":"00","signature":"00"
            }"#,
            ),
        };
        assert!(matches!(
            RequestFingerprint::from_stamp(&duplicate, ""),
            Err(TurnkeyClientError::InvalidReceiptStamp(
                "invalid stamp JSON"
            ))
        ));
    }

    struct TestStamper<F>(F);

    impl<F: Fn(&[u8]) -> Result<StampHeader, StamperError>> Stamp for TestStamper<F> {
        fn stamp(&self, body: &[u8]) -> Result<StampHeader, StamperError> {
            self.0(body)
        }
    }

    impl<F: Fn(&[u8]) -> Result<StampHeader, StamperError>> UniqueStamp for TestStamper<F> {}

    fn example_app_proof() -> crate::AppProof {
        use crate::generated::external::data::v1::SignatureScheme;

        crate::AppProof {
            scheme: SignatureScheme::EphemeralKeyP256,
            public_key: "proof-key".into(),
            proof_payload: "proof-payload".into(),
            signature: "proof-signature".into(),
        }
    }

    async fn assert_receipt_matches_ordinary_creation<T: std::fmt::Debug + PartialEq>(
        server: &MockServer,
        ordinary: ActivityResult<T>,
        submitted: SubmittedActivityResult<T>,
        proof: crate::AppProof,
        expected_kind: SubmissionKind,
    ) {
        let (result, receipt) = submitted.into_parts();
        assert_eq!(result.result, ordinary.result);
        assert_eq!(ordinary.app_proofs, [proof]);
        assert_eq!(result.app_proofs, ordinary.app_proofs);
        assert_eq!(receipt.activity().app_proofs, ordinary.app_proofs);
        assert_eq!(receipt.activity().id, ordinary.activity_id);
        assert_eq!(receipt.checkpoint().0.kind, expected_kind);

        let requests = server.received_requests().await.unwrap();
        let [ordinary_post, receipt_post] = requests.as_slice() else {
            panic!("expected one ordinary POST and one receipt POST");
        };
        assert_eq!(ordinary_post.body, receipt_post.body);
        assert_eq!(
            serde_json::from_slice::<Value>(&receipt_post.body).unwrap()["generateAppProofs"],
            true
        );
        assert_eq!(
            receipt.sent_fingerprints().copied().collect::<Vec<_>>(),
            [captured(receipt_post)]
        );
    }

    #[tokio::test]
    async fn operator_receipt_preserves_the_ordinary_request_and_result() {
        use crate::generated::CreateTvcOperatorIntent;

        let server = MockServer::start().await;
        let proof = example_app_proof();
        Mock::given(path("/public/v1/submit/create_tvc_operator"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
                "id": "operator-activity", "organizationId": ORGANIZATION_ID, "fingerprint": "server",
                "type": "ACTIVITY_TYPE_CREATE_TVC_OPERATOR", "status": "ACTIVITY_STATUS_COMPLETED",
                "appProofs": [proof],
                "result": {"createTvcOperatorResult": {
                    "walletId": "wallet", "operatorId": "operator",
                    "encryptPublicKey": "encrypt", "signPublicKey": "sign"
                }}
            }})))
            .mount(&server).await;
        let key = TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap();
        let client = client(&server, TestStamper(|body: &[u8]| key.stamp(body))).with_app_proofs();
        let params = CreateTvcOperatorIntent {
            wallet_id: None,
            wallet_name: Some("wallet".into()),
            path: "p".into(),
            operator_name: "operator".into(),
        };
        let ordinary = client
            .create_tvc_operator(ORGANIZATION_ID.to_string(), 100, params.clone())
            .await
            .unwrap();
        let submitted = client
            .create_tvc_operator_with_receipt(ORGANIZATION_ID, 100, params, async |_| {
                Ok::<(), Infallible>(())
            })
            .await
            .unwrap()
            .completed();
        assert_receipt_matches_ordinary_creation(
            &server,
            ordinary,
            submitted,
            proof,
            SubmissionKind::CreateTvcOperator,
        )
        .await;
    }

    #[tokio::test]
    async fn quorum_key_receipt_preserves_the_ordinary_request_and_result() {
        use crate::generated::CreateTvcQuorumKeyIntent;

        let server = MockServer::start().await;
        let proof = example_app_proof();
        Mock::given(path("/public/v1/submit/create_tvc_quorum_key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
                "id": "quorum-activity", "organizationId": ORGANIZATION_ID, "fingerprint": "server",
                "type": "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY", "status": "ACTIVITY_STATUS_COMPLETED",
                "appProofs": [proof],
                "result": {"createTvcQuorumKeyResult": {
                    "quorumKeyId": "quorum", "quorumPublicKey": "key", "shareIds": ["share"]
                }}
            }})))
            .mount(&server).await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        )
        .with_app_proofs();
        let params = CreateTvcQuorumKeyIntent {
            threshold: 2,
            operator_encrypt_keys: vec!["one".into(), "two".into()],
        };
        let ordinary = client
            .create_tvc_quorum_key(ORGANIZATION_ID.to_string(), 100, params.clone())
            .await
            .unwrap();
        let submitted = client
            .create_tvc_quorum_key_with_receipt(ORGANIZATION_ID, 100, params, async |_| {
                Ok::<(), Infallible>(())
            })
            .await
            .unwrap()
            .completed();
        assert_receipt_matches_ordinary_creation(
            &server,
            ordinary,
            submitted,
            proof,
            SubmissionKind::CreateTvcQuorumKey,
        )
        .await;
    }

    fn failure(error: impl Into<ReceiptCreationError<Infallible>>) -> SubmissionFailure {
        match error.into() {
            ReceiptCreationError::Client(TurnkeyClientError::SubmissionFailed(failure)) => failure,
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
                    ORGANIZATION_ID,
                    100,
                    CreateTvcOperatorIntent {
                        wallet_id: None,
                        wallet_name: Some("wallet".into()),
                        path: "path".into(),
                        operator_name: "operator".into(),
                    },
                    async |_| Ok::<(), Infallible>(()),
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

    #[tokio::test]
    async fn first_signing_failure_prevents_submission_and_redacts_the_stamper_error() {
        let server = MockServer::start().await;
        let stamper = TestStamper(|_: &[u8]| {
            Err(StamperError::InvalidPublicKeyBytes(
                "private request secret".into(),
            ))
        });
        let client = client(&server, stamper);
        let error = failure(client.submit((), "/submit".into()).await.unwrap_err());

        assert_eq!(error.submission_state(), SubmissionState::NeverSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::StampRejected);
        assert!(error.checkpoint().is_none());
        assert!(server.received_requests().await.unwrap().is_empty());
        assert!(!format!("{error:?} {error}").contains("private request secret"));
    }

    #[tokio::test]
    async fn retry_signing_failure_retains_the_first_post_fingerprint() {
        let server = MockServer::start().await;
        Mock::given(path("/submit"))
            .respond_with(response("ACTIVITY_STATUS_PENDING"))
            .mount(&server)
            .await;
        let first_key =
            std::cell::Cell::new(Some(TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap()));
        let stamper = TestStamper(|body: &[u8]| {
            first_key
                .take()
                .ok_or(StamperError::RandomizedSigningFailed)?
                .stamp(body)
        });
        let client = client(&server, stamper);
        let mut saved = Vec::new();
        let error = failure(
            client
                .process_activity_with_receipt(
                    (),
                    "/submit".into(),
                    ORGANIZATION_ID,
                    SubmissionKind::CreateTvcOperator,
                    async |checkpoint: &SubmissionCheckpoint| {
                        saved.push(checkpoint.clone());
                        Ok::<(), Infallible>(())
                    },
                )
                .await
                .unwrap_err(),
        );

        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(
            error.cause(),
            &SubmissionFailureCause::RandomizedSigningFailed
        );
        assert_eq!(error.activity_id(), Some("synthetic-activity"));
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(saved.len(), 1);
        let expected = [captured(&requests[0])];
        assert_eq!(
            saved[0].sent_fingerprints().copied().collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            error
                .checkpoint()
                .unwrap()
                .sent_fingerprints()
                .copied()
                .collect::<Vec<_>>(),
            expected
        );
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
    async fn ordinary_activity_and_query_include_sdk_version() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(completed())
            .mount(&server)
            .await;
        let client = client(
            &server,
            TurnkeyP256ApiKey::from_bytes([1; 32], None).unwrap(),
        );
        let body = json!({"timestampMs": "100"});
        client
            .process_activity(&body, "/ordinary-activity".into())
            .await
            .unwrap();
        client
            .get_activity(GetActivityRequest {
                organization_id: ORGANIZATION_ID.to_string(),
                activity_id: "synthetic-activity".into(),
            })
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
        for request in &requests {
            assert_eq!(
                request.headers.get("x-tvc-client-version").unwrap(),
                env!("CARGO_PKG_VERSION")
            );
        }
        assert_eq!(requests[0].body, serde_json::to_vec(&body).unwrap());
    }
}
