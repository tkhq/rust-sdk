use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use serde::Deserialize;
use turnkey_api_key_stamper::{
    API_KEY_STAMP_HEADER_NAME, RandomizedStamp, SIGNATURE_SCHEME_P256, SIGNATURE_SCHEME_SECP256K1,
    StampHeader,
};

use crate::generated::immutable::models::v1::SignatureScheme;
use crate::{Activity, ActivityResult, TurnkeyClient, TurnkeyClientError};

mod ump_signature;

/// SHA-256 fingerprint of a signed request using TKMS's exact signature encoding.
///
/// Includes the signature, which built-in stampers randomize by default to
/// distinguish separate submissions of the same request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
            _ => return Err(invalid("unsupported signature scheme")),
        };
        Self::new(scheme, &stamp.public_key, message, &stamp.signature)
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
            return Err(invalid("expected X-Stamp"));
        }
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(&header.value)
            .map_err(|_| invalid("invalid base64url"))?;
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid stamp JSON"))
    }
}

fn invalid(reason: &'static str) -> TurnkeyClientError {
    TurnkeyClientError::InvalidReceiptStamp(reason)
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
    activity_id: Option<String>,
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
        self.activity_id.as_deref()
    }

    pub fn cause(&self) -> &SubmissionFailureCause {
        &self.cause
    }

    fn from_error(
        error: TurnkeyClientError,
        state: SubmissionState,
        activity_id: Option<String>,
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
            activity_id,
            cause,
        }
    }
}

/// Remembers which signed requests this call tried to send, including retries.
/// Used to check that returned proofs belong to one of those requests.
/// If a send was attempted, an error doesn't mean creation failed.
/// The server's activity ID is only a lookup hint; it hasn't been verified.
#[derive(Default)]
pub(crate) struct SubmissionInvocation {
    pub(crate) sent: Vec<RequestFingerprint>,
    activity_id: Option<String>,
}

impl SubmissionInvocation {
    pub(crate) fn observe(&mut self, activity: &Activity) {
        if let Some(label) = safe_activity_id(&activity.id) {
            self.activity_id = Some(label);
        }
    }

    fn fail(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::from_error(
            error,
            if self.sent.is_empty() {
                SubmissionState::NeverSubmitted
            } else {
                SubmissionState::MayHaveSubmitted
            },
            self.activity_id.clone(),
        ))
    }
}

fn safe_activity_id(id: &str) -> Option<String> {
    (!id.is_empty() && id.len() <= 128 && id.chars().all(|c| !c.is_control()))
        .then(|| id.to_owned())
}

/// A completed activity and this invocation's nonempty submission set.
///
/// Submission metadata does not authenticate the activity or its output proofs.
#[derive(Debug, Clone)]
pub struct ActivityReceipt {
    activity: Activity,
    first: RequestFingerprint,
    remaining: Vec<RequestFingerprint>,
    activity_id: Option<String>,
}

impl ActivityReceipt {
    pub(crate) fn submission_error(&self, error: TurnkeyClientError) -> TurnkeyClientError {
        TurnkeyClientError::SubmissionFailed(SubmissionFailure::from_error(
            error,
            SubmissionState::MayHaveSubmitted,
            self.activity_id.clone(),
        ))
    }

    pub fn activity(&self) -> &Activity {
        &self.activity
    }

    pub fn sent_fingerprints(&self) -> impl Iterator<Item = &RequestFingerprint> {
        std::iter::once(&self.first).chain(self.remaining.iter())
    }
}

/// The typed activity result and local submission receipt.
///
/// Receipt methods require a built-in API key with randomized stamping support.
///
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
///     client.create_tvc_operator_with_receipt("org".into(), 100, operator).await;
///     client.create_tvc_quorum_key_with_receipt("org".into(), 100, quorum).await;
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
    /// Submits with randomized stamps and retains all creation POST fingerprints.
    ///
    /// No receipt is returned after failed, canceled, or ambiguous transport.
    /// The returned activity still requires independent proof verification.
    pub(crate) async fn process_activity_with_receipt<Request: serde::Serialize>(
        &self,
        request: Request,
        path: String,
    ) -> Result<ActivityReceipt, TurnkeyClientError> {
        let mut submission = SubmissionInvocation::default();
        let activity = self
            .process_activity_with_submission(&request, path, Some(&mut submission))
            .await
            .map_err(|error| submission.fail(error))?;
        let first = *submission
            .sent
            .first()
            .ok_or_else(|| submission.fail(invalid("no submitted fingerprint")))?;
        Ok(ActivityReceipt {
            activity,
            first,
            remaining: submission.sent.into_iter().skip(1).collect(),
            activity_id: submission.activity_id,
        })
    }
}
