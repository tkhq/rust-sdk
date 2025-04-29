use std::env;
use std::error::Error;
use tkhq_client::generated::GetWhoamiRequest;
use tkhq_examples::load_api_key_from_env;
// See <https://docs.turnkey.com/api-reference/sessions/who-am-i> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;
    let client = tkhq_client::TurnkeyClient::new(TURNKEY_API_HOST, api_key, None);

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
