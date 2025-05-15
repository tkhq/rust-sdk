#![doc = include_str!("../README.md")]
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::header::CONTENT_TYPE;
use thiserror::Error;

use generated::google::rpc::Status;
use generated::Activity;
use generated::ActivityResponse;
use generated::ActivityStatus;

use turnkey_api_key_stamper::StamperError;
// Re-export this for convenience
pub use turnkey_api_key_stamper::TurnkeyP256ApiKey;

pub mod generated;

pub mod retry;
pub use retry::RetryConfig;

const TURNKEY_RUST_SDK_USER_AGENT: &str =
    concat!("turnkey-rust-client/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Error)]
pub enum TurnkeyClientError {
    #[error("Client builder is missing its API key. Call .api_key(...) to configure it.")]
    BuilderMissingApiKey,

    #[error("HTTP client builder failed: {0}")]
    ReqwestBuilder(reqwest::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("HTTP response was not successful: {0} ({1})")]
    UnexpectedHttpStatus(u16, String),

    #[error("HTTP response does not have a Content-Type header")]
    MissingContentTypeHeader,

    #[error("HTTP response header could not be converted to str: {0}")]
    HeaderToStrError(String),

    #[error("HTTP response header could not be parsed from str: {0}")]
    HeaderFromStrError(String),

    #[error("HTTP response MIME type is not application/json (found {0})")]
    UnexpectedMimeType(String),

    #[error("Failed to decode response {0} ({1})")]
    Decode(String, serde_json::Error),

    #[error("Serde JSON failure: {0}")]
    SerdeJsonFailure(#[from] serde_json::Error),

    #[error("Missing activity from response")]
    MissingActivity,

    #[error("Missing result from activity")]
    MissingResult,

    #[error("Missing inner result from activity result")]
    MissingInnerResult,

    #[error("Activity status unexpected: {0}")]
    UnexpectedActivityStatus(String),

    #[error("Unexpected inner activity result: {0}")]
    UnexpectedInnerActivityResult(String),

    #[error("Activity failed processing: {0:?}")]
    ActivityFailed(Option<Status>),

    #[error("This activity ({0}) requires consensus and needs extra approvals")]
    ActivityRequiresApproval(String),

    #[error("Maximum number of attempts reached (after {0} retries)")]
    ExceededRetries(usize),

    #[error("Stamper error")]
    StamperError(#[from] StamperError),
}

/// Builder for [`TurnkeyClient`].
#[derive(Debug)]
pub struct TurnkeyClientBuilder {
    api_key: Option<TurnkeyP256ApiKey>,
    base_url: Option<String>,
    retry_config: Option<RetryConfig>,
    reqwest_builder: reqwest::ClientBuilder,
    timeout: Option<Duration>,
}

impl TurnkeyClientBuilder {
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            retry_config: None,
            reqwest_builder: reqwest::Client::builder(),
            timeout: None,
        }
    }

    /// Sets the API key for the Turnkey client.
    pub fn api_key(mut self, api_key: TurnkeyP256ApiKey) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Sets the base URL for the Turnkey client. Default: `https://api.turnkey.com`
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Sets the retry configuration for the Turnkey client
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = Some(retry_config);
        self
    }

    /// Sets the connect timeout for the underlying `reqwest` client
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.reqwest_builder = self.reqwest_builder.connect_timeout(timeout);
        self
    }

    /// Sets the HTTP timeout for the underlying `reqwest` client
    /// Default: 20 seconds
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self.reqwest_builder = self.reqwest_builder.timeout(timeout);
        self
    }

    /// Provide your own `reqwest::ClientBuilder` (useful for power users who need to configure reqwest in specific ways)
    pub fn with_reqwest_builder(
        mut self,
        f: impl FnOnce(reqwest::ClientBuilder) -> reqwest::ClientBuilder,
    ) -> Self {
        self.reqwest_builder = f(self.reqwest_builder);
        self
    }

    pub fn build(mut self) -> Result<TurnkeyClient, TurnkeyClientError> {
        if self.timeout.is_none() {
            self.reqwest_builder = self.reqwest_builder.timeout(Duration::from_secs(20));
        }

        self.reqwest_builder = self.reqwest_builder.user_agent(TURNKEY_RUST_SDK_USER_AGENT);

        Ok(TurnkeyClient {
            api_key: self
                .api_key
                .ok_or(TurnkeyClientError::BuilderMissingApiKey)?,
            base_url: self
                .base_url
                .unwrap_or("https://api.turnkey.com".to_string()),
            http: self
                .reqwest_builder
                .build()
                .map_err(TurnkeyClientError::ReqwestBuilder)?,
            retry_config: self.retry_config.unwrap_or_default(),
        })
    }
}

impl Default for TurnkeyClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Base client. To create a new client, see `TurnkeyClient::builder`.
#[derive(Debug)]
pub struct TurnkeyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: TurnkeyP256ApiKey,
    retry_config: RetryConfig,
}

impl TurnkeyClient {
    /// Creates a new `TurnkeyClientBuilder`. To get a client from a builder, call `.build()`
    pub fn builder() -> TurnkeyClientBuilder {
        TurnkeyClientBuilder::new()
    }

    /// POSTs an activity and polls until the status is "COMPLETE"
    ///
    /// `process_activity` accepts a  arbitrary `Request` and `path` to POST to the Turnkey API.
    /// It encapsulates the polling logic and is generally meant to be called by other
    /// activity-specific client functions (e.g. `create_sub_organization`).
    ///
    /// Given the Turnkey API is backwards-compatible, this function can be used to submit old versions of activities.
    /// For example, if the latest version for create_sub_organization is "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7",
    /// you may want to use `process_activity` to process `ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V6`. Note that this
    /// requires manually setting the correct URL and activity request type.
    /// The response is an generic Activity. If you're invoking this function manually you'll have to manually look at
    /// the correct `.activity.result` enum.
    ///
    /// This function is used by our generated client methods and sets the right request type, URL, and deserializes into the proper result type as well.
    /// Unless you have good reasons, prefer the type-safe client methods such as `sign_raw_payload`.
    ///
    /// # Returns
    ///
    /// This function returns an `Activity` object which contains the deserialized version of the response.
    ///
    /// # Errors
    ///
    /// If the server errors with a validation error, a server error, a deserialization error, the proper variant of `TurnkeyClientError` is returned.
    /// If the activity is pending and exceeds the maximum amount of retries allowed, `TurnkeyClientError::ExceededRetries` is returned.
    /// If the activity requires consensus, `TurnkeyClientError::ActivityRequiresApproval` is returned.
    pub async fn process_activity<Request: Serialize>(
        &self,
        request: Request,
        path: String,
    ) -> Result<Activity, TurnkeyClientError> {
        let mut retry_count = 0;

        loop {
            let response: ActivityResponse = self.process_request(&request, path.clone()).await?;
            let activity = response
                .activity
                .ok_or_else(|| TurnkeyClientError::MissingActivity)?;

            match activity.status {
                ActivityStatus::Completed => return Ok(activity),
                ActivityStatus::Pending => {
                    if retry_count >= self.retry_config.max_retries {
                        return Err(TurnkeyClientError::ExceededRetries(retry_count));
                    }
                    retry_count += 1;

                    let delay = self.retry_config.compute_delay(retry_count);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                ActivityStatus::Failed => {
                    return Err(TurnkeyClientError::ActivityFailed(activity.failure))
                }
                ActivityStatus::ConsensusNeeded => {
                    return Err(TurnkeyClientError::ActivityRequiresApproval(activity.id))
                }
                ActivityStatus::Unspecified
                | ActivityStatus::Created
                | ActivityStatus::Rejected => {
                    return Err(TurnkeyClientError::UnexpectedActivityStatus(
                        activity.status.as_str_name().to_string(),
                    ))
                }
            }
        }
    }

    /// Returns the current timestamp as a u128.
    pub fn current_timestamp(&self) -> u128 {
        let now = SystemTime::now();

        now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }

    /// Processes a `Request` (at `path`) by:
    /// * Serializing the request to JSON
    /// * Signing the request with the client's API key
    /// * POSTing the POST body and the associated stamp to the Turnkey API
    ///
    /// This function is generic and can handle POSTing queries or activities.
    pub async fn process_request<Request, Response>(
        &self,
        request: &Request,
        path: String,
    ) -> Result<Response, TurnkeyClientError>
    where
        Request: Serialize,
        Response: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, path);
        let post_body = serde_json::to_string(&request)?;
        let stamp = self.api_key.stamp(post_body.clone())?;
        let res = self
            .http
            .post(url)
            .header("X-Stamp", stamp)
            .body(post_body)
            .send()
            .await?;

        let status = res.status();
        let content_type = res
            .headers()
            .get(CONTENT_TYPE)
            .ok_or(TurnkeyClientError::MissingContentTypeHeader)?
            .to_str()
            .map_err(|e| TurnkeyClientError::HeaderToStrError(e.to_string()))?
            .parse::<mime::Mime>()
            .map_err(|e| TurnkeyClientError::HeaderFromStrError(e.to_string()))?;
        let text = res.text().await?;

        if !status.is_success() {
            return Err(TurnkeyClientError::UnexpectedHttpStatus(
                status.as_u16(),
                text,
            ));
        }

        if content_type != mime::APPLICATION_JSON {
            return Err(TurnkeyClientError::UnexpectedMimeType(
                content_type.to_string(),
            ));
        }

        let response =
            serde_json::from_str(&text).map_err(|e| TurnkeyClientError::Decode(text, e))?;

        Ok(response)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
    use crate::generated::result::Inner;
    use crate::generated::SignRawPayloadIntentV2;
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn simple_activity_intent() -> SignRawPayloadIntentV2 {
        SignRawPayloadIntentV2 {
            sign_with: "0x123456".to_string(),
            payload: "hello".to_string(),
            encoding: PayloadEncoding::TextUtf8,
            hash_function: HashFunction::Keccak256,
        }
    }

    async fn setup_client_and_server() -> (TurnkeyClient, MockServer) {
        let server = MockServer::start().await;
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::generate())
            .base_url(server.uri())
            .retry_config(RetryConfig {
                initial_delay: Duration::from_millis(50),
                multiplier: 2.0,
                max_delay: Duration::from_millis(1000),
                max_retries: 3,
            })
            .build()
            .unwrap();
        (client, server)
    }

    #[test]
    fn client_requires_an_api_key() {
        assert!(matches!(
            TurnkeyClient::builder().build().unwrap_err(),
            TurnkeyClientError::BuilderMissingApiKey
        ));
    }

    #[tokio::test]
    async fn test_raw_http_error() {
        let (client, server) = setup_client_and_server().await;

        // Simulate 500 Internal Server Error
        let response = ResponseTemplate::new(500).set_body_string("internal server error");
        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::UnexpectedHttpStatus(status, body) => {
                assert_eq!(status, 500);
                assert_eq!(body, "internal server error");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_bad_encoding() {
        let (client, server) = setup_client_and_server().await;

        // Simulate 500 Internal Server Error
        let response = ResponseTemplate::new(200).set_body_string("success but not JSON");
        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::UnexpectedMimeType(mime_type) => {
                assert_eq!(mime_type, "text/plain");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_http_error_with_valid_json() {
        let (client, server) = setup_client_and_server().await;

        // Simulate 500 Internal Server Error
        let response = ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "code":2,
            "message": "some error",
            "details":[],
            "turnkeyErrorCode":"SOME_ERROR_CODE",
        }));

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::UnexpectedHttpStatus(status, body) => {
                assert_eq!(status, 401);
                assert_eq!(body, "{\"code\":2,\"details\":[],\"message\":\"some error\",\"turnkeyErrorCode\":\"SOME_ERROR_CODE\"}");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_missing_activity() {
        let (client, server) = setup_client_and_server().await;

        let response =
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "activity": null }));
        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        assert!(matches!(result, Err(TurnkeyClientError::MissingActivity)));
    }

    #[tokio::test]
    async fn test_activity_failed() {
        let (client, server) = setup_client_and_server().await;

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "status": "ACTIVITY_STATUS_FAILED",
                "id": "some-activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
                "failure": {
                    "code": 1,
                    "message": "failure reason",
                },
            }
        }));

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::ActivityFailed(status) => {
                assert_eq!(
                    status,
                    Some(Status {
                        code: 1,
                        message: "failure reason".to_string(),
                        details: vec![]
                    })
                );
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_activity_needs_consensus() {
        let (client, server) = setup_client_and_server().await;

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2",
                "status": "ACTIVITY_STATUS_CONSENSUS_NEEDED",
                "id": "some-activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
            }
        }));

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::ActivityRequiresApproval(activity_id) => {
                assert_eq!(activity_id, "some-activity-id");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_unexpected_activity_status() {
        let (client, server) = setup_client_and_server().await;

        // Status that is REJECTED triggers UnexpectedActivityStatus
        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
                "status": "ACTIVITY_STATUS_REJECTED",
                "id": "some-activity-id",
                "organizationId": "org-id",
                "fingerprint": "fingerprint",
            }
        }));

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::UnexpectedActivityStatus(status) => {
                assert_eq!(status, "ACTIVITY_STATUS_REJECTED");
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_successful_activity() {
        let (client, server) = setup_client_and_server().await;

        // Real activity response captured from the network
        let raw_activity_response = "{\"activity\":{\"id\":\"019660f7-801d-75d8-a40e-e4f69944b711\", \"organizationId\":\"651b573c-861b-4f10-a478-cbcfe0c226af\", \"status\":\"ACTIVITY_STATUS_COMPLETED\", \"type\":\"ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7\", \"intent\":{\"createSubOrganizationIntentV7\":{\"subOrganizationName\":\"New sub-organization\", \"rootUsers\":[{\"userName\":\"Root User\", \"apiKeys\":[{\"apiKeyName\":\"Test API Key\", \"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"curveType\":\"API_KEY_CURVE_P256\"}], \"authenticators\":[], \"oauthProviders\":[]}], \"rootQuorumThreshold\":1, \"wallet\":{\"walletName\":\"New wallet\", \"accounts\":[{\"curve\":\"CURVE_SECP256K1\", \"pathFormat\":\"PATH_FORMAT_BIP32\", \"path\":\"m/44'/60'/0'/0\", \"addressFormat\":\"ADDRESS_FORMAT_ETHEREUM\"}]}}}, \"result\":{\"createSubOrganizationResultV7\":{\"subOrganizationId\":\"d047f3a2-6c66-40ee-ab71-95f8a0148fe3\", \"wallet\":{\"walletId\":\"ac651e99-579f-5c7c-8e06-16430bc25dc1\", \"addresses\":[\"0x4Cb785085B399570F9Ccc0d145eeE0359CC74aCC\"]}, \"rootUserIds\":[\"00f2d4f0-0bb8-4f77-bf7e-013c76e9302d\"]}}, \"votes\":[{\"id\":\"da7460b7-9871-45bf-8ef5-c42aab01fc77\", \"userId\":\"72465bbd-469d-4f78-94d5-7920a2141641\", \"user\":{\"userId\":\"72465bbd-469d-4f78-94d5-7920a2141641\", \"userName\":\"Root user\", \"userEmail\":\"rno+rust@turnkey.io\", \"authenticators\":[{\"transports\":[\"AUTHENTICATOR_TRANSPORT_HYBRID\", \"AUTHENTICATOR_TRANSPORT_INTERNAL\"], \"attestationType\":\"none\", \"aaguid\":\"-_wwBxVOTsyMC24CBVfXvQ\", \"credentialId\":\"u2oC6Kaoo39ozqvSVZHH1-mWpLI\", \"model\":\"Security key\", \"credential\":{\"publicKey\":\"pQECAyYgASFYIOt5O_a9z7xo9u1xbvg35vywIaB06uWWTvSwxeu26dGAIlggEOmdA0TqzU7AMSuIiHKy2r7TFzKhdGR-CUJRXqGnV00\", \"type\":\"CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR\"}, \"authenticatorId\":\"b34b8c49-3a6c-4b31-a93d-42c0d263d987\", \"authenticatorName\":\"Laptop\", \"createdAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}}], \"apiKeys\":[{\"credential\":{\"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"type\":\"CREDENTIAL_TYPE_API_KEY_P256\"}, \"apiKeyId\":\"7a6c2199-4903-4fe2-94da-e3f89a5e2a56\", \"apiKeyName\":\"Test\", \"createdAt\":{\"seconds\":\"1743191268\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191268\", \"nanos\":\"0\"}}], \"userTags\":[], \"oauthProviders\":[], \"createdAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}}, \"activityId\":\"019660f7-801d-75d8-a40e-e4f69944b711\", \"selection\":\"VOTE_SELECTION_APPROVED\", \"message\":\"{\\\"type\\\":\\\"ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7\\\",\\\"timestampMs\\\":\\\"1745383554864\\\",\\\"organizationId\\\":\\\"651b573c-861b-4f10-a478-cbcfe0c226af\\\",\\\"parameters\\\":{\\\"subOrganizationName\\\":\\\"New sub-organization\\\",\\\"rootUsers\\\":[{\\\"userName\\\":\\\"Root User\\\",\\\"userEmail\\\":null,\\\"userPhoneNumber\\\":null,\\\"apiKeys\\\":[{\\\"apiKeyName\\\":\\\"Test API Key\\\",\\\"publicKey\\\":\\\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\\\",\\\"curveType\\\":\\\"API_KEY_CURVE_P256\\\",\\\"expirationSeconds\\\":null}],\\\"authenticators\\\":[],\\\"oauthProviders\\\":[]}],\\\"rootQuorumThreshold\\\":1,\\\"wallet\\\":{\\\"walletName\\\":\\\"New wallet\\\",\\\"accounts\\\":[{\\\"curve\\\":\\\"CURVE_SECP256K1\\\",\\\"pathFormat\\\":\\\"PATH_FORMAT_BIP32\\\",\\\"path\\\":\\\"m/44'/60'/0'/0\\\",\\\"addressFormat\\\":\\\"ADDRESS_FORMAT_ETHEREUM\\\"}],\\\"mnemonicLength\\\":null},\\\"disableEmailRecovery\\\":null,\\\"disableEmailAuth\\\":null,\\\"disableSmsAuth\\\":null,\\\"disableOtpEmailAuth\\\":null}}\", \"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"signature\":\"30450221008787fb627333f4299f28720775e447e6f6dcdb9c0f21f006aa3d59c90741a7de02205247df721eae2550f3569205b3dd155e693070ea95ae794e87b7e1684eb91814\", \"scheme\":\"SIGNATURE_SCHEME_TK_API_P256\", \"createdAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}}], \"fingerprint\":\"sha256:5a2d45e700283d37f537b4ce0888f1ee93cb90d794de1426c3bc033da120fe53\", \"canApprove\":false, \"canReject\":true, \"createdAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}, \"failure\":null}}";
        let response = ResponseTemplate::new(200)
            .set_body_raw(raw_activity_response.as_bytes(), "application/json");

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        let activity = result.unwrap();
        assert_eq!(
            activity.id,
            "019660f7-801d-75d8-a40e-e4f69944b711".to_string()
        );

        // Now assert that we can access the inner result, and assert that it's the correct result type and content
        match activity.result.unwrap().inner.unwrap() {
            Inner::CreateSubOrganizationResultV7(res) => {
                assert_eq!(
                    res.sub_organization_id,
                    "d047f3a2-6c66-40ee-ab71-95f8a0148fe3"
                )
            }
            _other => {
                panic!("didn't match on the right type!")
            }
        }
    }

    #[tokio::test]
    async fn client_respects_custom_timeout() {
        let server = MockServer::start().await;

        // This response takes 2 seconds to return
        let delayed_response = ResponseTemplate::new(200)
            .set_delay(Duration::from_secs(2))
            .set_body_string("OK");

        // Register the mock handler
        Mock::given(method("POST"))
            .respond_with(delayed_response)
            .mount(&server)
            .await;

        // Build client with a short timeout (e.g. 1s -- intentionally shorter than the server delay of 2s)
        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::generate())
            .base_url(&server.uri())
            .timeout(Duration::from_secs(1))
            .build()
            .unwrap();

        let result: Result<ActivityResponse, TurnkeyClientError> = client
            .process_request(&simple_activity_intent(), "/sign_raw_payload".to_string())
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::Http(err) => {
                assert!(err.is_timeout());
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[tokio::test]
    async fn client_has_custom_user_agent() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            // Expect the request is made with the custom user agent
            .and(header("User-Agent", TURNKEY_RUST_SDK_USER_AGENT))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activity": {
                    "type": "ACTIVITY_TYPE_SIGN_RAW_PAYLOAD",
                    "status": "ACTIVITY_STATUS_COMPLETED",
                    "id": "some-activity-id",
                    "organizationId": "org-id",
                    "fingerprint": "fingerprint",
                }
            })))
            .mount(&server)
            .await;

        let client = TurnkeyClient::builder()
            .api_key(TurnkeyP256ApiKey::generate())
            .base_url(&server.uri())
            .build()
            .unwrap();

        let res: ActivityResponse = client
            .process_request(&simple_activity_intent(), "/sign_raw_payload".to_string())
            .await
            .unwrap();
        assert_eq!(res.activity.unwrap().id, "some-activity-id".to_string());
    }

    mod retry {
        use super::*;
        use wiremock::{Request, Respond};

        // Custom Responder: fail N times, then succeed
        struct FailThenSucceedResponder {
            failures_left: Arc<std::sync::Mutex<usize>>,
        }

        impl Respond for FailThenSucceedResponder {
            fn respond(&self, _req: &Request) -> ResponseTemplate {
                let mut lock = self.failures_left.lock().unwrap();
                if *lock > 0 {
                    *lock -= 1;
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "activity": {
                            "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
                            "status": "ACTIVITY_STATUS_PENDING",
                            "id": "retried-activity-id",
                            "organizationId": "org-id",
                            "fingerprint": "fingerprint",
                        }
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "activity": {
                            "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
                            "status": "ACTIVITY_STATUS_COMPLETED",
                            "id": "retried-activity-id",
                            "organizationId": "org-id",
                            "fingerprint": "fingerprint",
                        }
                    }))
                }
            }
        }

        #[tokio::test]
        async fn test_retry_then_success() {
            let (client, server) = setup_client_and_server().await;

            let failures_left = Arc::new(std::sync::Mutex::new(2)); // Fail 2 times
            let responder = FailThenSucceedResponder {
                failures_left: failures_left.clone(),
            };

            Mock::given(method("POST"))
                .respond_with(responder)
                .mount(&server)
                .await;

            let result = client
                .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
                .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap().id, "retried-activity-id");
        }

        #[tokio::test]
        async fn test_retry_exceeds_max_retries() {
            let (client, server) = setup_client_and_server().await;

            // Always pending, no success
            let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "activity": {
                    "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
                    "status": "ACTIVITY_STATUS_PENDING",
                    "id": "some-activity-id",
                    "organizationId": "org-id",
                    "fingerprint": "fingerprint",
                }
            }));

            Mock::given(method("POST"))
                .respond_with(response)
                .mount(&server)
                .await;

            let result = client
                .process_activity(simple_activity_intent(), "/sign_raw_payload".to_string())
                .await;

            match result.unwrap_err() {
                TurnkeyClientError::ExceededRetries(n) => {
                    // Our retry configuration has 5 retries configured
                    assert_eq!(n, 3);
                }
                other => panic!("unexpected error: {:?}", other),
            }
        }
    }
}
