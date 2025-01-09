use std::collections::HashMap;

use crate::errors::{StampError, TurnkeyError};
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use dotenv::dotenv;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey};
use p256::FieldBytes;
use reqwest::header::HeaderValue;
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::env;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
}

#[derive(Debug)]
pub struct StampInput<'a> {
    sealed_body: &'a str,
    public_key_hex: String,
    private_key_hex: String,
}

#[derive(Debug)]
pub struct SealedRequestInput {
    pub body: HashMap<String, Value>,
    pub public_key_hex: Option<String>,
    pub private_key_hex: Option<String>,
}

#[derive(Debug)]
pub struct RequestInput {
    pub uri: String,
    pub method: String,
    pub headers: Option<HeadersShape>,
    pub query: Option<QueryShape>,
    pub body: Option<BodyShape>,
    pub substitution: Option<SubstitutionShape>,
}

#[derive(Debug)]
pub struct SealedRequestOutput {
    pub sealed_body: String,
    pub x_stamp: String,
}

pub struct TurnkeyApiKey {
    pub private_key_hex: Option<String>,
    pub public_key_hex: Option<String>,
}

pub struct Turnkey {
    stamper: TurnkeyApiKey,
    base_url: String,
    client: Client,
}

#[derive(Debug)]
pub enum QueryValueShape {
    Single(String),
    Array(Vec<String>),
}

pub type HeadersShape = HashMap<String, String>;
pub type BodyShape = HashMap<String, Value>;
pub type QueryShape = HashMap<String, QueryValueShape>;
pub type SubstitutionShape = HashMap<String, String>;
pub type TurnkeyResult<T> = std::result::Result<T, TurnkeyError>;

impl TurnkeyApiKey {
    pub fn new() -> TurnkeyResult<Self> {
        dotenv::dotenv().ok();

        Ok(Self {
            private_key_hex: env::var("TURNKEY_API_PRIVATE_KEY")
                .map_err(|e| TurnkeyError::OtherError(e.to_string()))
                .ok(),
            public_key_hex: env::var("TURNKEY_API_PUBLIC_KEY")
                .map_err(|e| TurnkeyError::OtherError(e.to_string()))
                .ok(),
        })
    }
}

impl Turnkey {
    pub fn new() -> TurnkeyResult<Self> {
        dotenv().ok();

        Ok(Self {
            base_url: env::var("TURNKEY_BASE_URL")
                .map_err(|e| TurnkeyError::OtherError(e.to_string()))?,
            stamper: TurnkeyApiKey::new()?,
            client: Client::new(),
        })
    }

    pub fn stamp(&self, stamp_input: StampInput) -> TurnkeyResult<String> {
        let private_key_bytes = hex::decode(stamp_input.private_key_hex)
            .map_err(|e| TurnkeyError::StampError(StampError::InvalidPrivateKeyString(e)))?;

        let signing_key: SigningKey =
            SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes))
                .map_err(|_| TurnkeyError::StampError(StampError::InvalidPrivateKeyBytes))?;

        let sig: Signature = signing_key.sign(stamp_input.sealed_body.as_bytes());
        let stamp = ApiStamp {
            public_key: stamp_input.public_key_hex.clone(),
            signature: hex::encode(sig.to_der()),
            scheme: "SIGNATURE_SCHEME_TK_API_P256".to_string(),
        };

        let json_stamp = serde_json::to_string(&stamp).unwrap();

        Ok(BASE64_URL_SAFE_NO_PAD.encode(json_stamp.as_bytes()))
    }

    fn substitute_path(uri: String, substitution: SubstitutionShape) -> TurnkeyResult<String> {
        let mut res = uri;
        for (key, val) in &substitution {
            let sub = format!("{{{}}}", key);
            let subbed_str = res.replace(&sub, val);
            if subbed_str == res {
                return Err(TurnkeyError::OtherError(
                    "Substitution key was not found.".to_string(),
                ));
            }
            res = subbed_str;
        }

        // TODO May be a better way to handle this, but {} are not valid uri chars
        if res.contains('{') && res.contains('}') {
            return Err(TurnkeyError::OtherError(
                "Did not substitute all keys.".to_string(),
            ));
        }
        Ok(res)
    }
    pub fn construct_url(
        &self,
        uri: String,
        query: QueryShape,
        substitution: SubstitutionShape,
    ) -> TurnkeyResult<String> {
        let path = Turnkey::substitute_path(uri, substitution)?;
        // TODO: Might need to check if path starts with "/", not sure how its called
        let mut url = format!("{}{}", self.base_url, path);

        // TODO: This can all be cleaned up/simplified with Url crate if needed
        if !query.is_empty() {
            url = format!("{}?", url);
        }
        for (key, val) in query.iter() {
            match val {
                QueryValueShape::Single(item) => {
                    url = format!("{}&{}={}", url, key, item);
                }
                QueryValueShape::Array(items) => {
                    for item in items {
                        url = format!("{}&{}={}", url, key, item);
                    }
                }
            }
        }
        Ok(url)
    }

    pub async fn request<ResponseData>(
        &self,
        request_input: RequestInput,
    ) -> TurnkeyResult<ResponseData>
    where
        ResponseData: DeserializeOwned,
    {
        // TODO: unwrap_or_default correct here? or make construct_url take in an Option
        let url = self.construct_url(
            request_input.uri,
            request_input.query.unwrap_or_default(),
            request_input.substitution.unwrap_or_default(),
        )?;

        let sealed_request_input = SealedRequestInput {
            body: request_input.body.unwrap_or_default(),
            public_key_hex: None,
            private_key_hex: None,
        };
        let sealed_request_output = self
            .seal_and_stamp_request_body(sealed_request_input)
            .await?;

        let response = self
            .client
            .post(&url)
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                if let Ok(x_stamp_header) =
                    HeaderValue::try_from(sealed_request_output.x_stamp.clone())
                {
                    headers.insert("X-Stamp", x_stamp_header);
                }
                match request_input.headers {
                    Some(input_headers) => {
                        for (_, value) in input_headers.iter() {
                            if let Ok(val_header) = HeaderValue::try_from(value) {
                                headers.insert("test keyname", val_header);
                            }
                        }
                        headers
                    }
                    None => headers,
                }
            })
            .body(sealed_request_output.sealed_body)
            .send()
            .await
            .map_err(TurnkeyError::HttpError)?;

        match response.status() {
            reqwest::StatusCode::OK => match response.json::<ResponseData>().await {
                Ok(parsed) => Ok(parsed),
                Err(e) => Err(TurnkeyError::OtherError(e.to_string())),
            },
            other => Err(TurnkeyError::OtherError(format!(
                "Received status code: {}",
                other
            ))),
        }
    }

    pub async fn seal_and_stamp_request_body(
        &self,
        input: SealedRequestInput,
    ) -> TurnkeyResult<SealedRequestOutput> {
        // TODO: Change the "or_else" into "unwrap_or" -> what's fallback val?
        let public_key_hex = input
            .public_key_hex
            .or_else(|| self.stamper.public_key_hex.clone())
            .ok_or_else(|| TurnkeyError::OtherError("No public key given or found".to_string()))?;

        let private_key_hex = input
            .private_key_hex
            .or_else(|| self.stamper.private_key_hex.clone())
            .ok_or_else(|| TurnkeyError::OtherError("No public key given or found".to_string()))?;

        let sealed_body = serde_json::to_string(&input.body)
            .map_err(|e| TurnkeyError::OtherError(e.to_string()))?;

        let stamp_input = StampInput {
            sealed_body: &sealed_body,
            public_key_hex,
            private_key_hex,
        };

        // Encoded serialized stamp
        let x_stamp = self.stamp(stamp_input)?;
        Ok(SealedRequestOutput {
            sealed_body,
            x_stamp,
        })
    }
}
