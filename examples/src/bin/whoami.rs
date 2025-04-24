use std::env;
use std::error::Error;
use tkhq_api_key_stamper::TurnkeyApiKey;
use tkhq_client::generated::GetWhoamiRequest;
use tkhq_client::RetryConfig;
// See <https://docs.turnkey.com/api-reference/sessions/who-am-i> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";

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

    let client = tkhq_client::TurnkeyClient::new(TURNKEY_API_HOST, api_key, RetryConfig::none());
    let req = GetWhoamiRequest {
        organization_id: env::var("TURNKEY_ORGANIZATION_ID").unwrap(),
    };

    let res = client.get_whoami(req).await?;
    println!(
        "Organization: \"{}\" ({})",
        res.organization_name, res.organization_id
    );
    println!("User: \"{}\" ({})", res.username, res.user_id);

    Ok(())
}
