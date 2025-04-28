use std::error::Error;
use std::{env, time, vec};
use tkhq_api_key_stamper::TurnkeyApiKey;
use tkhq_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
use tkhq_client::generated::{CreateWalletIntent, DeleteWalletsIntent, SignRawPayloadIntentV2};
use tkhq_client::{
    generated::{
        immutable::common::v1::{AddressFormat, Curve, PathFormat},
        WalletAccountParams,
    },
    RetryConfig,
};

// See <https://docs.turnkey.com/api-reference/organizations/create-sub-organization> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file from the example folder (examples/.env)
    let current_dir = env::current_dir()?; // should be the workspace root
    let env_path = current_dir.join("examples").join(".env");

    if env_path.exists() {
        dotenvy::from_path(&env_path)?;
    } else {
        println!("No .env file found at {:?}", env_path);
        println!("Continuing because env might already be populated with the right variables");
    }

    let api_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");
    let api_private_key =
        env::var("TURNKEY_API_PRIVATE_KEY").expect("cannot load TURNKEY_API_PRIVATE_KEY");

    let api_key = TurnkeyApiKey {
        private_key_hex: api_private_key,
        public_key_hex: api_public_key.clone(),
    };

    // Create the request body for our create_sub_organization request
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");
    let timestamp_ms = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let client = tkhq_client::TurnkeyClient::new(TURNKEY_API_HOST, api_key, RetryConfig::none());
    let intent = CreateWalletIntent {
        wallet_name: "New wallet".to_string(),
        accounts: vec![WalletAccountParams {
            curve: Curve::Secp256k1,
            path_format: PathFormat::Bip32,
            path: "m/44'/60'/0'/0".to_string(),
            address_format: AddressFormat::Ethereum,
        }],
        mnemonic_length: None, // Let that be the default
    };

    let res = client
        .create_wallet(organization_id.clone(), timestamp_ms, intent)
        .await?;

    assert_eq!(res.addresses.len(), 1);
    let eth_address = res.addresses.first().unwrap();
    let wallet_id = res.wallet_id;

    println!(
        "New ETH address created: {} (wallet ID: {})",
        eth_address, wallet_id
    );

    // Now we can sign something
    let signature_activity_result = client
        .sign_raw_payload(
            organization_id.clone(),
            timestamp_ms,
            SignRawPayloadIntentV2 {
                sign_with: eth_address.to_string(),
                payload: "hello from TKHQ".to_string(),
                encoding: PayloadEncoding::TextUtf8,
                hash_function: HashFunction::Keccak256,
            },
        )
        .await;

    let signature = signature_activity_result.unwrap();

    println!(
        "Produced a new signature: r={}, s={}, v={}",
        signature.r, signature.s, signature.v,
    );

    // Finally, delete the wallet. We don't need it, let's clean up!
    let delete_wallet_result = client
        .delete_wallets(
            organization_id,
            timestamp_ms,
            DeleteWalletsIntent {
                wallet_ids: vec![wallet_id.clone()],
                delete_without_export: Some(true),
            },
        )
        .await;

    let deleted_wallets = delete_wallet_result.unwrap().wallet_ids;
    assert_eq!(deleted_wallets.len(), 1);
    assert_eq!(deleted_wallets.first().unwrap().to_string(), wallet_id);

    println!("Deleted wallet {}", wallet_id);

    Ok(())
}
