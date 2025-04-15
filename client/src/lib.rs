//! Turnkey Client to interact with the Turnkey API
//! See <https://docs.turnkey.com>
use reqwest;
use tkhq_api_key_stamper::{stamp, TurnkeyApiKey};

mod generated;

pub struct TurnkeyClient {
    http: reqwest::Client,
    base_url: String,
    api_key: TurnkeyApiKey,
}

impl TurnkeyClient {
    pub fn new(base_url: impl Into<String>, api_key: TurnkeyApiKey) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into(),
            api_key: api_key,
        }
    }
}
