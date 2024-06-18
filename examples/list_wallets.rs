use tkhq_rust_sdk::client::{self, GetWallets};
use tkhq_rust_sdk::gen::services::coordinator::public::v1 as api;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();
    let organization_id = std::env::var("TURNKEY_ORGANIZATION_ID").unwrap();
    let tk = client::TurnkeyClient::new_from_env().unwrap();
    let req = api::GetWalletsRequest { organization_id };
    let resp = tk.request::<GetWallets>(req).await.unwrap();
    println!("{:#?}", resp);
}
