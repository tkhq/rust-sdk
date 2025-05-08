use std::env;
use std::error::Error;
use tkhq_client::generated::GetWhoamiRequest;
use tkhq_examples::load_api_key_from_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;
    let client = tkhq_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?;

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
