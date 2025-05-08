use std::error::Error;
use std::{env, vec};
use turnkey_client::generated::immutable::common::v1::{HashFunction, PayloadEncoding};
use turnkey_client::generated::{
    immutable::common::v1::{AddressFormat, Curve, PathFormat},
    WalletAccountParams,
};
use turnkey_client::generated::{
    CreateWalletIntent, DeleteWalletsIntent, ExportWalletIntent, SignRawPayloadIntentV2,
};
use turnkey_enclave_encrypt::{ExportClient, QuorumPublicKey};
use turnkey_examples::load_api_key_from_env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load API key
    let api_key = load_api_key_from_env()?;

    // Get organization ID from env
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    // Create our Turnkey client
    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?;

    // Create a new wallet in the organization
    let create_wallet_result = client
        .create_wallet(
            organization_id.clone(),
            client.current_timestamp(),
            CreateWalletIntent {
                wallet_name: "New wallet".to_string(),
                accounts: vec![WalletAccountParams {
                    curve: Curve::Secp256k1,
                    path_format: PathFormat::Bip32,
                    path: "m/44'/60'/0'/0".to_string(),
                    address_format: AddressFormat::Ethereum,
                }],
                mnemonic_length: None, // Let that be the default
            },
        )
        .await?;

    assert_eq!(create_wallet_result.addresses.len(), 1);
    let eth_address = create_wallet_result.addresses.first().unwrap();
    let wallet_id = create_wallet_result.wallet_id;

    println!(
        "New ETH address created: {} (wallet ID: {})",
        eth_address, wallet_id
    );

    // Now we can sign something
    let signature_result = client
        .sign_raw_payload(
            organization_id.clone(),
            client.current_timestamp(),
            SignRawPayloadIntentV2 {
                sign_with: eth_address.to_string(),
                payload: "hello from TKHQ".to_string(),
                encoding: PayloadEncoding::TextUtf8,
                hash_function: HashFunction::Keccak256,
            },
        )
        .await?;

    println!(
        "Produced a new signature: r={}, s={}, v={}",
        signature_result.r, signature_result.s, signature_result.v,
    );

    // Export our wallet using `ExportClient`
    let mut export_client = ExportClient::new(&QuorumPublicKey::production_signer());
    let export_wallet_result = client
        .export_wallet(
            organization_id.clone(),
            client.current_timestamp(),
            ExportWalletIntent {
                wallet_id: wallet_id.clone(),
                target_public_key: export_client.target_public_key()?,
                language: None,
            },
        )
        .await?;

    let export_bundle = export_wallet_result.export_bundle;
    let mnemonic_phrase =
        export_client.decrypt_wallet_mnemonic_phrase(export_bundle, organization_id.clone())?;

    assert_eq!(export_wallet_result.wallet_id, wallet_id);
    println!(
        "Wallet successfully exported: {} (Mnemonic phrase: {})",
        export_wallet_result.wallet_id,
        first_and_last_word(&mnemonic_phrase)
    );

    // Finally, delete the wallet. We don't need it, let's clean up!
    let delete_wallet_result = client
        .delete_wallets(
            organization_id,
            client.current_timestamp(),
            DeleteWalletsIntent {
                wallet_ids: vec![wallet_id.clone()],
                delete_without_export: Some(false),
            },
        )
        .await;

    let deleted_wallets = delete_wallet_result.unwrap().wallet_ids;
    assert_eq!(deleted_wallets.len(), 1);
    assert_eq!(deleted_wallets.first().unwrap().to_string(), wallet_id);

    println!("Deleted wallet {}", wallet_id);

    Ok(())
}

// Simple convenience function to display the first and last word of a mnemonic phrase
// Technically we could just print the whole mnemonic out...but something about that feels wrong. I'm coy.
fn first_and_last_word(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    format!("{} ... {}", words.first().unwrap(), words.last().unwrap())
}
