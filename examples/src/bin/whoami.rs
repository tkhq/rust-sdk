use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::json;
use std::env;
use std::error::Error;
use tkhq_api_key_stamper::{stamp, TurnkeyApiKey};

// See <https://docs.turnkey.com/api-reference/sessions/who-am-i> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";
const WHOAMI_PATH: &str = "/public/v1/query/whoami";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let current_dir = env::current_dir()?; // should be the workspace root
    dotenvy::from_path(current_dir.join("examples").join(".env"))?;

    // Create our API key from env
    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");
    let api_key = TurnkeyApiKey {
        private_key_hex: api_private_key,
        public_key_hex: api_public_key,
    };

    // Create the request body for our whoami request
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");
    let body = json!({
        "organizationId": organization_id,
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
        .post(format!("{TURNKEY_API_HOST}{WHOAMI_PATH}"))
        .headers(headers)
        .body(body)
        .send()
        .await?;

    println!("Status: {}", response.status());

    let text = response.text().await?;
    println!("Response body\n{}", text);

    Ok(())
}
