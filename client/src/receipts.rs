use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use p256::ecdsa::signature::Verifier as _;
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use turnkey_api_key_stamper::{
    API_KEY_STAMP_HEADER_NAME, FreshStamp, SIGNATURE_SCHEME_P256, SIGNATURE_SCHEME_SECP256K1,
    StampHeader,
};

use crate::{Activity, ActivityResult, TurnkeyClient, TurnkeyClientError};

/// SHA-256 of the exact authenticated request tuple in Ump's Signature encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestFingerprint([u8; 32]);

impl RequestFingerprint {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    fn signature(
        scheme: i32,
        public_key: &str,
        message: &str,
        signature: &str,
    ) -> Result<Self, TurnkeyClientError> {
        let mut digest = Sha256::new();
        digest.update(scheme.to_le_bytes());
        for text in [public_key, message, signature] {
            let len = u32::try_from(text.len())
                .map_err(|_| invalid("signature field exceeds the Ump length limit"))?;
            digest.update(len.to_le_bytes());
            digest.update(text.as_bytes());
        }
        Ok(Self(digest.finalize().into()))
    }

    pub(crate) fn from_stamp(
        header: &StampHeader,
        message: &str,
    ) -> Result<Self, TurnkeyClientError> {
        if !header.name.eq_ignore_ascii_case(API_KEY_STAMP_HEADER_NAME) {
            return Err(invalid("expected X-Stamp"));
        }
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(&header.value)
            .map_err(|_| invalid("invalid base64url"))?;
        let stamp: ApiStamp =
            serde_json::from_slice(&bytes).map_err(|_| invalid("invalid stamp JSON"))?;
        let public_key =
            hex::decode(&stamp.public_key).map_err(|_| invalid("invalid public key encoding"))?;
        let signature =
            hex::decode(&stamp.signature).map_err(|_| invalid("invalid signature encoding"))?;
        let scheme = match stamp.scheme.as_str() {
            SIGNATURE_SCHEME_P256 => {
                let key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key)
                    .map_err(|_| invalid("invalid P256 key"))?;
                let signature = p256::ecdsa::Signature::from_der(&signature)
                    .map_err(|_| invalid("invalid P256 signature"))?;
                key.verify(message.as_bytes(), &signature)
                    .map_err(|_| invalid("P256 signature does not match request"))?;
                1
            }
            SIGNATURE_SCHEME_SECP256K1 => {
                let key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&public_key)
                    .map_err(|_| invalid("invalid secp256k1 key"))?;
                let signature = k256::ecdsa::Signature::from_der(&signature)
                    .map_err(|_| invalid("invalid secp256k1 signature"))?;
                key.verify(message.as_bytes(), &signature)
                    .map_err(|_| invalid("secp256k1 signature does not match request"))?;
                4
            }
            _ => return Err(invalid("unsupported signature scheme")),
        };
        Self::signature(scheme, &stamp.public_key, message, &stamp.signature)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
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
    #[error("fresh stamping could not obtain OS entropy")]
    EntropyUnavailable,
    #[error("fresh signing failed")]
    FreshSigningFailed,
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
                turnkey_api_key_stamper::StamperError::EntropyUnavailable,
            ) => Cause::EntropyUnavailable,
            TurnkeyClientError::StamperError(
                turnkey_api_key_stamper::StamperError::FreshSigningFailed,
            ) => Cause::FreshSigningFailed,
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

impl<S: FreshStamp> TurnkeyClient<S> {
    /// Submits with fresh stamps and retains all creation POST fingerprints.
    ///
    /// No receipt is returned after failed, cancelled, or ambiguous transport.
    /// The returned activity still requires independent proof verification.
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
    ///     client.process_activity_with_receipt((), "/submit".into()).await;
    ///     client.create_tvc_operator_with_receipt("org".into(), 100, operator).await;
    ///     client.create_tvc_quorum_key_with_receipt("org".into(), 100, quorum).await;
    /// }
    /// ```
    pub async fn process_activity_with_receipt<Request: serde::Serialize>(
        &self,
        request: Request,
        path: String,
    ) -> Result<ActivityReceipt, TurnkeyClientError> {
        self.process_activity_with_receipt_stamper(&request, path, S::stamp_fresh)
            .await
    }

    async fn process_activity_with_receipt_stamper<Request: serde::Serialize>(
        &self,
        request: &Request,
        path: String,
        stamp: fn(&S, &[u8]) -> Result<StampHeader, turnkey_api_key_stamper::StamperError>,
    ) -> Result<ActivityReceipt, TurnkeyClientError> {
        let mut invocation = SubmissionInvocation::default();
        let activity = self
            .process_activity_with_stamper(request, path, stamp, Some(&mut invocation))
            .await
            .map_err(|error| invocation.fail(error))?;
        let first = *invocation
            .sent
            .first()
            .ok_or_else(|| invocation.fail(invalid("no submitted fingerprint")))?;
        Ok(ActivityReceipt {
            activity,
            first,
            remaining: invocation.sent.into_iter().skip(1).collect(),
            activity_id: invocation.activity_id,
        })
    }
}

#[cfg(test)]
#[path = "receipt_tests.rs"]
mod tests;
