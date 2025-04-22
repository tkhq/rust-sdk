//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

use generated::Activity;
use generated::google::rpc::Status;
use generated::ActivityStatus;
use generated::ActivityResponse;

use reqwest;
use tkhq_api_key_stamper::TurnkeyApiKey;

pub mod generated;

#[derive(Debug, Error)]
pub enum TurnkeyClientError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Failed to decode response: {0}")]
    Decode(#[from] serde_json::Error),

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
    retry_config: RetryConfig
}

impl TurnkeyClient {
    pub fn new(base_url: impl Into<String>, api_key: TurnkeyApiKey, retry_config: RetryConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: api_key,
            retry_config: retry_config,
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
            let res = self.http
            .post(url.clone())
            .header("X-Stamp", stamp.clone())
            .body(post_body.clone())
            .send()
            .await?;
        
            //let parsed = res.json::<ActivityResponse>().await?;
            let text_response = res.text().await?;
            println!("response: {:?}", text_response);

            let parsed = ActivityResponse {
                activity: None,
            };
            let activity = parsed.activity.ok_or_else(|| TurnkeyClientError::MissingActivity)?;

            let status = ActivityStatus::try_from(activity.status)?;
            match status {
                ActivityStatus::Completed => return Ok(activity),
                ActivityStatus::Pending => {
                    if retry_count >= self.retry_config.max_retries {
                        return Err(TurnkeyClientError::ExceededRetries(retry_count))
                    }
                    retry_count += 1;

                    let delay = compute_retry_delay(self.retry_config.initial_delay, self.retry_config.multiplier, retry_count, self.retry_config.max_delay);
                    tokio::time::sleep(delay).await;
                    continue;
                },
                ActivityStatus::Failed => return Err(TurnkeyClientError::ActivityFailed(activity.failure)),
                ActivityStatus::ConsensusNeeded => return Err(TurnkeyClientError::ActivityRequiresApproval(activity.id)),
                ActivityStatus::Unspecified | ActivityStatus::Created | ActivityStatus::Rejected => {
                    return Err(TurnkeyClientError::UnexpectedActivityStatus(status.as_str_name().to_string()))
                },
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
fn compute_retry_delay(base: Duration, multiplier: f64, attempt_count: usize, max: Duration) -> Duration {
    let factor = multiplier.powi(attempt_count as i32);
    let mut delay = base.mul_f64(factor);
    if delay > max {
        delay = max;
    }

    delay
}
