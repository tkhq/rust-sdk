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
            p256.process_activity_with_receipt(&body, "/p256".into())
                .await
                .unwrap(),
        );
        receipts.push(
            p256.process_activity_with_receipt(&body, "/p256".into())
                .await
                .unwrap(),
        );
        receipts.push(
            secp.process_activity_with_receipt(&body, "/secp".into())
                .await
                .unwrap(),
        );
        receipts.push(
            secp.process_activity_with_receipt(&body, "/secp".into())
                .await
                .unwrap(),
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
            .process_activity_with_receipt(CountSerialize(serializations.clone()), "/submit".into())
            .await
            .unwrap();
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
            client.process_activity_with_receipt(left, "/left".into()),
            client.process_activity_with_receipt(right, "/right".into()),
            client.process_request::<_, Value>(&query_body, "/query".into()),
        );
        query.unwrap();
        let requests = server.received_requests().await.unwrap();
        for (receipt, route) in [(left.unwrap(), "/left"), (right.unwrap(), "/right")] {
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
        let result = tokio::time::timeout(
            Duration::from_millis(40),
            client.process_activity_with_receipt((), "/slow".into()),
        )
        .await;
        assert!(result.is_err());
        let receipt = client
            .process_activity_with_receipt((), "/ok".into())
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 2);
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
        assert!(
            matches!(short.process_activity_with_receipt((), "/slow".into()).await,
                Err(TurnkeyClientError::SubmissionFailed(e))
                    if e.submission_state() == SubmissionState::MayHaveSubmitted
                        && e.cause() == &SubmissionFailureCause::Transport)
        );
        Mock::given(path("/failed"))
            .respond_with(response("ACTIVITY_STATUS_FAILED"))
            .mount(&server)
            .await;
        assert!(matches!(
            client
                .process_activity_with_receipt((), "/failed".into())
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
            .create_tvc_operator_with_receipt("org".into(), 100, params)
            .await
            .unwrap();
        assert_eq!(ordinary.result, submitted.activity_result().result);
        assert_eq!(submitted.receipt().activity().id, ordinary.activity_id);
        let params = CreateTvcQuorumKeyIntent {
            threshold: 2,
            operator_encrypt_keys: vec!["one".into(), "two".into()],
        };
        let ordinary = client
            .create_tvc_quorum_key("org".into(), 100, params.clone())
            .await
            .unwrap();
        let (submitted, receipt) = client
            .create_tvc_quorum_key_with_receipt("org".into(), 100, params)
            .await
            .unwrap()
            .into_parts();
        assert_eq!(ordinary.result, submitted.result);
        assert_eq!(receipt.activity().id, ordinary.activity_id);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].body, requests[1].body);
        assert_eq!(requests[2].body, requests[3].body);
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
        let error = failure(
            client
                .process_activity_with_receipt((), "/submit".into())
                .await
                .unwrap_err(),
        );
        assert_eq!(error.submission_state(), SubmissionState::MayHaveSubmitted);
        assert_eq!(error.cause(), &SubmissionFailureCause::HttpStatus(503));
        assert_eq!(error.activity_id(), None);
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
                .process_activity_with_receipt(
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
    async fn direct_tvc_creation_requests_include_sdk_version_without_changing_body() {
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
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 5);
        for (index, request) in requests[..4].iter().enumerate() {
            assert_eq!(
                request.headers["x-tvc-client-version"],
                env!("CARGO_PKG_VERSION")
            );
            assert_eq!(
                serde_json::from_slice::<Value>(&request.body).unwrap(),
                json!({"generateAppProofs": index < 2})
            );
        }
        assert!(!requests[4].headers.contains_key("x-tvc-client-version"));
    }
}
