//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

use generated::google::rpc::Status;
use generated::Activity;
use generated::ActivityResponse;
use generated::ActivityStatus;

use tkhq_api_key_stamper::StamperError;
use tkhq_api_key_stamper::TurnkeyP256ApiKey;

#[cfg_attr(doc, doc(hidden))]
pub mod generated;

pub mod retry;
pub use retry::RetryConfig;

#[derive(Debug, Error)]
pub enum TurnkeyClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

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

pub struct TurnkeyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: TurnkeyP256ApiKey,
    retry_config: RetryConfig,
}

impl TurnkeyClient {
    /// Creates a new `TurnkeyClient`. If `retry_config` is not provided, `RetryConfig::default()` is used.
    pub fn new(
        base_url: impl Into<String>,
        api_key: TurnkeyP256ApiKey,
        retry_config: Option<RetryConfig>,
    ) -> Self {
        Self {
            api_key,
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            retry_config: retry_config.unwrap_or_default(),
        }
    }

    /// POSTs an activity and polls until the status is "COMPLETE"
    ///
    /// `process_activity` accepts an arbitrary URL, stamp and POST body.
    /// It encapsulates the polling logic and is generally meant to be called by other
    /// activity-specific client functions (e.g. `create_sub_organization`).
    ///
    /// Given the Turnkey API is backwards-compatible, this function can be used to submit old versions of activities.
    /// For example, if the latest version for create_sub_organization is "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7",
    /// you may want to use `process_activity` to process `ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V6`. Note that this
    /// requires manually setting the correct URL, activity type, intent and result types, whereas using other
    /// generated functions on the client is guaranteed type safe).
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
    pub async fn process_activity(
        &self,
        url: String,
        stamp: String,
        post_body: String,
    ) -> Result<Activity, TurnkeyClientError> {
        let mut retry_count = 0;

        loop {
            let res = self
                .http
                .post(url.clone())
                .header("X-Stamp", stamp.clone())
                .body(post_body.clone())
                .send()
                .await?;

            let res_text = res.text().await?;
            let parsed = serde_json::from_str::<ActivityResponse>(&res_text)
                .map_err(|e| TurnkeyClientError::Decode(res_text, e))?;

            let activity = parsed
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

    pub fn current_timestamp(&self) -> String {
        let now = SystemTime::now();

        now.duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::generated::result::Inner;
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn dummy_post_body() -> String {
        "{}".to_string()
    }

    fn dummy_stamp() -> String {
        "dummy-stamp".to_string()
    }

    async fn setup_client_and_server() -> (TurnkeyClient, MockServer) {
        let server = MockServer::start().await;
        let client = TurnkeyClient::new(
            server.uri(),
            TurnkeyP256ApiKey::generate(),
            Some(RetryConfig {
                initial_delay: Duration::from_millis(50),
                multiplier: 2.0,
                max_delay: Duration::from_millis(1000),
                max_retries: 3,
            }),
        );
        (client, server)
    }

    #[tokio::test]
    async fn test_http_error() {
        let (client, server) = setup_client_and_server().await;

        // Simulate 500 Internal Server Error
        let response = ResponseTemplate::new(500).set_body_string("internal server error");
        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(
                format!("{}/fail", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
            .await;

        match result.unwrap_err() {
            TurnkeyClientError::Decode(text, err) => {
                assert_eq!(text, "internal server error");
                assert_eq!(err.classify(), serde_json::error::Category::Syntax);
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
            .process_activity(
                format!("{}/noactivity", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
            .await;

        assert!(matches!(result, Err(TurnkeyClientError::MissingActivity)));
    }

    #[tokio::test]
    async fn test_activity_failed() {
        let (client, server) = setup_client_and_server().await;

        let response = ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "activity": {
                "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
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
            .process_activity(
                format!("{}/failstatus", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
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
                "type": "ACTIVITY_TYPE_CREATE_POLICY_V3",
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
            .process_activity(
                format!("{}/consensusneeded", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
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
            .process_activity(
                format!("{}/rejected", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
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
        let response = ResponseTemplate::new(200).set_body_string(raw_activity_response);

        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;

        let result = client
            .process_activity(
                format!("{}/completed", server.uri()),
                dummy_stamp(),
                dummy_post_body(),
            )
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
                .process_activity(
                    format!("{}/retry-then-success", server.uri()),
                    dummy_stamp(),
                    dummy_post_body(),
                )
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
                .process_activity(
                    format!("{}/retry-fail", server.uri()),
                    dummy_stamp(),
                    dummy_post_body(),
                )
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
