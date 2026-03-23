use anyhow::{anyhow, Context, Result};
use std::future::Future;
use std::pin::Pin;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::immutable::common::v1::HashFunction;
use turnkey_client::generated::immutable::common::v1::PayloadEncoding;
use turnkey_client::generated::{GetPrivateKeyRequest, SignRawPayloadIntentV2};
use turnkey_client::{TurnkeyClient, TurnkeyClientError};

use crate::config::Config;

pub struct TurnkeySigner {
    client: TurnkeyClient<TurnkeyP256ApiKey>,
    config: Config,
}

pub trait Signer {
    fn get_public_key(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>>;
    fn sign_ed25519<'a>(
        &'a self,
        payload: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;
}

impl TurnkeySigner {
    pub fn from_env() -> Result<Self> {
        Self::new(Config::from_env()?)
    }

    pub fn new(config: Config) -> Result<Self> {
        let api_key =
            TurnkeyP256ApiKey::from_strings(&config.api_private_key, Some(&config.api_public_key))
                .context("failed to load Turnkey API key")?;

        let client = TurnkeyClient::builder()
            .api_key(api_key)
            .base_url(&config.api_base_url)
            .build()
            .context("failed to build Turnkey client")?;

        Ok(Self { client, config })
    }

    pub async fn get_public_key(&self) -> Result<Vec<u8>> {
        let response = self
            .client
            .get_private_key(GetPrivateKeyRequest {
                organization_id: self.config.organization_id.clone(),
                private_key_id: self.config.private_key_id.clone(),
            })
            .await
            .map_err(map_turnkey_error)?;

        let private_key = response
            .private_key
            .ok_or_else(|| anyhow!("Turnkey did not return a private key object"))?;

        decode_public_key(&private_key.public_key)
    }

    pub async fn sign_ed25519(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let response = self
            .client
            .sign_raw_payload(
                self.config.organization_id.clone(),
                self.client.current_timestamp(),
                SignRawPayloadIntentV2 {
                    sign_with: self.config.private_key_id.clone(),
                    payload: hex::encode(payload),
                    encoding: PayloadEncoding::Hexadecimal,
                    hash_function: HashFunction::NotApplicable,
                },
            )
            .await
            .map_err(map_turnkey_error)?;

        decode_signature_parts(&response.result.r, &response.result.s, &response.result.v)
    }
}

impl Signer for TurnkeySigner {
    fn get_public_key(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
        Box::pin(async move { TurnkeySigner::get_public_key(self).await })
    }

    fn sign_ed25519<'a>(
        &'a self,
        payload: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
        Box::pin(async move { TurnkeySigner::sign_ed25519(self, payload).await })
    }
}

fn map_turnkey_error(error: TurnkeyClientError) -> anyhow::Error {
    anyhow!("Turnkey API request failed: {error}")
}

fn decode_public_key(encoded: &str) -> Result<Vec<u8>> {
    let trimmed = encoded.trim().trim_start_matches("0x");
    if let Ok(bytes) = hex::decode(trimmed) {
        return Ok(bytes);
    }

    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(trimmed)
        .map_err(|_| anyhow!("unsupported Turnkey public key encoding"))
}

fn decode_signature_parts(r: &str, s: &str, v: &str) -> Result<Vec<u8>> {
    let r = decode_hex_maybe_empty(r).context("failed to decode Turnkey signature field r")?;
    let s = decode_hex_maybe_empty(s).context("failed to decode Turnkey signature field s")?;
    let v = decode_hex_maybe_empty(v).context("failed to decode Turnkey signature field v")?;

    if r.len() == 64 && s.is_empty() && v.is_empty() {
        return Ok(r);
    }

    if r.len() == 32 && s.len() == 32 && (v.is_empty() || v.len() == 1) {
        return Ok([r, s].concat());
    }

    Err(anyhow!(
        "unsupported Ed25519 signature layout from Turnkey: r={} bytes, s={} bytes, v={} bytes",
        r.len(),
        s.len(),
        v.len()
    ))
}

fn decode_hex_maybe_empty(value: &str) -> Result<Vec<u8>> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }

    let trimmed = value.trim().trim_start_matches("0x");
    hex::decode(trimmed).map_err(Into::into)
}
