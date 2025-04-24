//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use generated::google::rpc::Status;
use generated::Activity;
use generated::ActivityResponse;
use generated::ActivityStatus;

use tkhq_api_key_stamper::TurnkeyApiKey;

#[cfg_attr(doc, doc(hidden))]
pub mod generated;

#[derive(Debug, Error)]
pub enum TurnkeyClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to decode response {0} ({1})")]
    Decode(String, serde_json::Error),

    #[error("Serde JSON failure: {0}")]
    SerdeJsonFailure(#[from] serde_json::Error),

    #[error("Request timed out")]
    Timeout,

    #[error("Missing activity from response")]
    MissingActivity,

    #[error("Prost decoding error: {0}")]
    ProstDecode(#[from] prost::DecodeError),

    #[error("Prost encoding error: {0}")]
    ProstEncode(#[from] prost::EncodeError),

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
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial delay before the first retry
    pub initial_delay: Duration,
    /// Multiplier for `initial_delay`, yielding the next delay
    pub multiplier: f64,
    /// Maximum delay between retries to cap the time between retries.
    pub max_delay: Duration,
    /// Maximum number of retries.
    pub max_retries: usize,
}

impl Default for RetryConfig {
    fn default() -> Self {
        RetryConfig {
            initial_delay: Duration::from_millis(500),
            multiplier: 2.0,
            max_delay: Duration::from_secs(10),
            max_retries: 5,
        }
    }
}

impl RetryConfig {
    /// Returns a `RetryConfig` which doesn't allow any retries
    /// Use this if you do not want the TurnkeyClient to retry on your behalf.
    /// If you need a retry configuration, look at `::default()`
    pub fn none() -> Self {
        RetryConfig {
            initial_delay: Duration::from_millis(0),
            multiplier: 0.0,
            max_delay: Duration::from_secs(0),
            max_retries: 0,
        }
    }
}

pub struct TurnkeyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: TurnkeyApiKey,
    retry_config: RetryConfig,
}

impl TurnkeyClient {
    pub fn new(
        base_url: impl Into<String>,
        api_key: TurnkeyApiKey,
        retry_config: RetryConfig,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key,
            retry_config,
        }
    }

    // POSTs an activity and polls until the status is "COMPLETE"
    // If the max number of retry is reached, an error is thrown.
    // Activities requiring consensus throw an error.
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

                    let delay = compute_retry_delay(
                        self.retry_config.initial_delay,
                        self.retry_config.multiplier,
                        retry_count,
                        self.retry_config.max_delay,
                    );
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

// Computes the delay given a base duration, a multiplier, attempt_count, and max
fn compute_retry_delay(
    base: Duration,
    multiplier: f64,
    attempt_count: usize,
    max: Duration,
) -> Duration {
    let factor = multiplier.powi(attempt_count as i32);
    let mut delay = base.mul_f64(factor);
    if delay > max {
        delay = max;
    }

    delay
}

#[cfg(test)]
mod test {
    use crate::generated::{result::Inner, ActivityResponse};

    #[test]
    fn test_create_sub_organization_response_parsing() {
        let raw_string = "{\"activity\":{\"id\":\"019660f7-801d-75d8-a40e-e4f69944b711\", \"organizationId\":\"651b573c-861b-4f10-a478-cbcfe0c226af\", \"status\":\"ACTIVITY_STATUS_COMPLETED\", \"type\":\"ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7\", \"intent\":{\"createSubOrganizationIntentV7\":{\"subOrganizationName\":\"New sub-organization\", \"rootUsers\":[{\"userName\":\"Root User\", \"apiKeys\":[{\"apiKeyName\":\"Test API Key\", \"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"curveType\":\"API_KEY_CURVE_P256\"}], \"authenticators\":[], \"oauthProviders\":[]}], \"rootQuorumThreshold\":1, \"wallet\":{\"walletName\":\"New wallet\", \"accounts\":[{\"curve\":\"CURVE_SECP256K1\", \"pathFormat\":\"PATH_FORMAT_BIP32\", \"path\":\"m/44'/60'/0'/0\", \"addressFormat\":\"ADDRESS_FORMAT_ETHEREUM\"}]}}}, \"result\":{\"createSubOrganizationResultV7\":{\"subOrganizationId\":\"d047f3a2-6c66-40ee-ab71-95f8a0148fe3\", \"wallet\":{\"walletId\":\"ac651e99-579f-5c7c-8e06-16430bc25dc1\", \"addresses\":[\"0x4Cb785085B399570F9Ccc0d145eeE0359CC74aCC\"]}, \"rootUserIds\":[\"00f2d4f0-0bb8-4f77-bf7e-013c76e9302d\"]}}, \"votes\":[{\"id\":\"da7460b7-9871-45bf-8ef5-c42aab01fc77\", \"userId\":\"72465bbd-469d-4f78-94d5-7920a2141641\", \"user\":{\"userId\":\"72465bbd-469d-4f78-94d5-7920a2141641\", \"userName\":\"Root user\", \"userEmail\":\"rno+rust@turnkey.io\", \"authenticators\":[{\"transports\":[\"AUTHENTICATOR_TRANSPORT_HYBRID\", \"AUTHENTICATOR_TRANSPORT_INTERNAL\"], \"attestationType\":\"none\", \"aaguid\":\"-_wwBxVOTsyMC24CBVfXvQ\", \"credentialId\":\"u2oC6Kaoo39ozqvSVZHH1-mWpLI\", \"model\":\"Security key\", \"credential\":{\"publicKey\":\"pQECAyYgASFYIOt5O_a9z7xo9u1xbvg35vywIaB06uWWTvSwxeu26dGAIlggEOmdA0TqzU7AMSuIiHKy2r7TFzKhdGR-CUJRXqGnV00\", \"type\":\"CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR\"}, \"authenticatorId\":\"b34b8c49-3a6c-4b31-a93d-42c0d263d987\", \"authenticatorName\":\"Laptop\", \"createdAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}}], \"apiKeys\":[{\"credential\":{\"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"type\":\"CREDENTIAL_TYPE_API_KEY_P256\"}, \"apiKeyId\":\"7a6c2199-4903-4fe2-94da-e3f89a5e2a56\", \"apiKeyName\":\"Test\", \"createdAt\":{\"seconds\":\"1743191268\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191268\", \"nanos\":\"0\"}}], \"userTags\":[], \"oauthProviders\":[], \"createdAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1743191135\", \"nanos\":\"0\"}}, \"activityId\":\"019660f7-801d-75d8-a40e-e4f69944b711\", \"selection\":\"VOTE_SELECTION_APPROVED\", \"message\":\"{\\\"type\\\":\\\"ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7\\\",\\\"timestampMs\\\":\\\"1745383554864\\\",\\\"organizationId\\\":\\\"651b573c-861b-4f10-a478-cbcfe0c226af\\\",\\\"parameters\\\":{\\\"subOrganizationName\\\":\\\"New sub-organization\\\",\\\"rootUsers\\\":[{\\\"userName\\\":\\\"Root User\\\",\\\"userEmail\\\":null,\\\"userPhoneNumber\\\":null,\\\"apiKeys\\\":[{\\\"apiKeyName\\\":\\\"Test API Key\\\",\\\"publicKey\\\":\\\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\\\",\\\"curveType\\\":\\\"API_KEY_CURVE_P256\\\",\\\"expirationSeconds\\\":null}],\\\"authenticators\\\":[],\\\"oauthProviders\\\":[]}],\\\"rootQuorumThreshold\\\":1,\\\"wallet\\\":{\\\"walletName\\\":\\\"New wallet\\\",\\\"accounts\\\":[{\\\"curve\\\":\\\"CURVE_SECP256K1\\\",\\\"pathFormat\\\":\\\"PATH_FORMAT_BIP32\\\",\\\"path\\\":\\\"m/44'/60'/0'/0\\\",\\\"addressFormat\\\":\\\"ADDRESS_FORMAT_ETHEREUM\\\"}],\\\"mnemonicLength\\\":null},\\\"disableEmailRecovery\\\":null,\\\"disableEmailAuth\\\":null,\\\"disableSmsAuth\\\":null,\\\"disableOtpEmailAuth\\\":null}}\", \"publicKey\":\"03bf162576eb8dfecf33d9275d09595284f6c4df0db6156c3c582777886a0ee0ac\", \"signature\":\"30450221008787fb627333f4299f28720775e447e6f6dcdb9c0f21f006aa3d59c90741a7de02205247df721eae2550f3569205b3dd155e693070ea95ae794e87b7e1684eb91814\", \"scheme\":\"SIGNATURE_SCHEME_TK_API_P256\", \"createdAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}}], \"fingerprint\":\"sha256:5a2d45e700283d37f537b4ce0888f1ee93cb90d794de1426c3bc033da120fe53\", \"canApprove\":false, \"canReject\":true, \"createdAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}, \"updatedAt\":{\"seconds\":\"1745383555\", \"nanos\":\"0\"}, \"failure\":null}}";

        let parsed = serde_json::from_str::<ActivityResponse>(&raw_string).unwrap();
        assert_eq!(
            parsed.activity.clone().unwrap().id,
            "019660f7-801d-75d8-a40e-e4f69944b711"
        );

        let inner = parsed.activity.unwrap().result.unwrap().inner.unwrap();
        match inner {
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
        //assert_eq!(parsed.activity.unwrap().result.unwrap().inner.unwrap(), "019660f7-801d-75d8-a40e-e4f69944b711")
    }
}
