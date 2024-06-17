use std::{env, process};

use tkhq_rust_sdk::client::{self, CreateWallet};
use tkhq_rust_sdk::gen::external::activity::v1::CreateWalletRequest;
use tkhq_rust_sdk::gen::immutable::activity::v1::{
    ActivityType, CreateWalletIntent, WalletAccountParams,
};
use tkhq_rust_sdk::gen::immutable::common::v1::{AddressFormat, Curve, PathFormat};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: create_wallet NAME");
        process::exit(1);
    }
    let wallet_name = args[1].clone();

    let organization_id = env::var("TURNKEY_ORGANIZATION_ID").unwrap();

    let tk = client::TurnkeyClient::new_from_env().unwrap();

    let timestamp_ms = tk.request_timestamp_ms();
    let req = CreateWalletRequest {
        organization_id,
        r#type: ActivityType::CreateWallet.as_str_name().to_owned(),
        timestamp_ms,
        parameters: Some(CreateWalletIntent {
            wallet_name,
            accounts: vec![WalletAccountParams {
                curve: Curve::Secp256k1.into(),
                path_format: PathFormat::Bip32.into(),
                path: "m/44'/60'/0'/0/0".to_owned(),
                address_format: AddressFormat::Ethereum.into(),
            }],
            mnemonic_length: None,
        }),
    };
    let resp = tk.request::<CreateWallet>(req).await.unwrap();
    println!("{:#?}", resp);
}
