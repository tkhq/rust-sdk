use anyhow::{anyhow, Context, Result};
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

impl TurnkeySigner {
    /// Builds a signer from an already resolved auth config.
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

    /// Fetches the configured Ed25519 public key bytes from Turnkey.
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

    /// Signs a raw Ed25519 payload through Turnkey and returns the 64-byte signature.
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

fn map_turnkey_error(error: TurnkeyClientError) -> anyhow::Error {
    anyhow!("Turnkey API request failed: {error}")
}

fn decode_public_key(encoded: &str) -> Result<Vec<u8>> {
    let trimmed = encoded.trim().trim_start_matches("0x");
    hex::decode(trimmed).map_err(|_| anyhow!("expected hex-encoded Turnkey public key"))
}

fn decode_signature_parts(r: &str, s: &str, v: &str) -> Result<Vec<u8>> {
    let r = decode_hex(r).context("failed to decode Turnkey signature field r")?;
    let s = decode_hex(s).context("failed to decode Turnkey signature field s")?;
    let v = decode_hex(v).context("failed to decode Turnkey signature field v")?;

    if r.len() == 32 && s.len() == 32 && v.len() == 1 {
        return Ok([r, s].concat());
    }

    Err(anyhow!(
        "unsupported Ed25519 signature layout from Turnkey: r={} bytes, s={} bytes, v={} bytes",
        r.len(),
        s.len(),
        v.len()
    ))
}

fn decode_hex(value: &str) -> Result<Vec<u8>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("expected non-empty hex value"));
    }

    let trimmed = trimmed.trim_start_matches("0x");
    hex::decode(trimmed).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::{decode_public_key, decode_signature_parts};

    #[test]
    fn decode_public_key_rejects_base64_input() {
        let error = decode_public_key("ZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmY=")
            .expect_err("base64 public keys should be rejected");

        assert_eq!(error.to_string(), "expected hex-encoded Turnkey public key");
    }

    #[test]
    fn decode_signature_parts_rejects_empty_v() {
        let r = "11".repeat(32);
        let s = "22".repeat(32);
        let error = decode_signature_parts(&r, &s, "").expect_err("empty v should be rejected");

        assert_eq!(
            error.to_string(),
            "failed to decode Turnkey signature field v"
        );
    }
}
