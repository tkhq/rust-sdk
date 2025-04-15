use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde_json::json;
use std::error::Error;
use std::{env, time};
use tkhq_api_key_stamper::{stamp, TurnkeyApiKey};

// See <https://docs.turnkey.com/api-reference/organizations/create-sub-organization> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";
const CREATE_SUB_ORGANIZATION_PATH: &str = "/public/v1/submit/create_sub_organization";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let current_dir = env::current_dir()?; // should be the workspace root
    dotenvy::from_path(current_dir.join("examples").join(".env"))?;

    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");

    let api_key = TurnkeyApiKey {
        private_key_hex: api_private_key,
        public_key_hex: api_public_key.clone(),
    };

    // We'll reuse the public key we have in our env, for convenience.
    // In a real scenario this will be the public key associated with an end user, or a passkey, etc.
    let suborg_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");

    // Create the request body for our create_sub_organization request
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");
    let timestamp_ms = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let body = json!({
        "type": "ACTIVITY_TYPE_CREATE_SUB_ORGANIZATION_V7",
        "timestampMs": format!("{}", timestamp_ms),
        "organizationId": organization_id,
        "parameters": {
            "subOrganizationName": "New sub-organization",
            "rootUsers": [
                {
                    "userName": "Root User",
                  "apiKeys": [
                    {
                      "apiKeyName": "Test API key",
                      "publicKey": suborg_public_key,
                      "curveType": "API_KEY_CURVE_P256",
                    }
                  ],
                  "oauthProviders": [],
                  "authenticators": [],
                }
            ],
            "rootQuorumThreshold": 1,
            "wallet": {
                "walletName": "New ETH wallet",
                "accounts": [
                    {
                    "curve": "CURVE_SECP256K1",
                    "pathFormat": "PATH_FORMAT_BIP32",
                    "path": "m/44'/60'/0'/0",
                    "addressFormat": "ADDRESS_FORMAT_ETHEREUM"
                    }
                ],
            },
        },
    })
    .to_string();

    // Sign the request with a stamp
    let mut headers = HeaderMap::new();
    let stamp = stamp(body.clone(), &api_key)?;
    headers.insert("X-Stamp", HeaderValue::from_str(&stamp)?);

    // Send the request out!
    let client = reqwest::Client::new();
    println!("Sending request body:\n{}\n", body.clone());
    println!("Associated X-Stamp header:\n{}\n", stamp.clone());
    let response = client
        .post(format!("{TURNKEY_API_HOST}{CREATE_SUB_ORGANIZATION_PATH}"))
        .headers(headers)
        .body(body)
        .send()
        .await?;

    println!("Status: {}", response.status());

    if response.status() == StatusCode::OK {
        let text = response.text().await?;
        // This should be valid JSON -- you can further parse information by digging in
        let json: serde_json::Value = serde_json::from_str(&text)?;
        let activity_result = json
            .pointer("/activity/result/createSubOrganizationResultV7")
            .unwrap();
        println!("Activity result\n{}\n", activity_result);
    } else {
        // Not a 200, just log the body for debugging
        let error_body = response.text().await?;
        println!("Response body\n{:?}\n", error_body);
    }

    Ok(())
}
