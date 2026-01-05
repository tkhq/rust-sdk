use std::env;
use std::error::Error;
use turnkey_api_key_stamper::Stamp;
use turnkey_client::generated::immutable::activity::v1 as immutable_activity;
use turnkey_client::generated::immutable::common::v1::{AddressFormat, Curve, PathFormat};
use turnkey_client::generated::DeleteWalletsIntent;
use turnkey_client::generated::WalletAccountParams;
use turnkey_client::ActivityResult;
use turnkey_examples::load_api_key_from_env;
use turnkey_proofs::{get_boot_proof_for_app_proof, verify};

async fn create_wallet(
    client: &turnkey_client::TurnkeyClient<impl Stamp>,
    organization_id: String,
) -> Result<ActivityResult<immutable_activity::CreateWalletResult>, Box<dyn Error>> {
    let intent = immutable_activity::CreateWalletIntent {
        wallet_name: "Test Wallet".to_string(),
        accounts: vec![WalletAccountParams {
            curve: Curve::Secp256k1,
            path_format: PathFormat::Bip32,
            path: "m/44'/60'/0'/0".to_string(),
            address_format: AddressFormat::Ethereum,
        }],
        mnemonic_length: None,
    };

    let activity_result = client
        .create_wallet(organization_id, client.current_timestamp(), intent)
        .await?;

    Ok(activity_result)
}

async fn delete_wallet(
    client: &turnkey_client::TurnkeyClient<impl Stamp>,
    organization_id: String,
    wallet_id: String,
) -> Result<(), Box<dyn Error>> {
    let activity_result = client
        .delete_wallets(
            organization_id,
            client.current_timestamp(),
            DeleteWalletsIntent {
                wallet_ids: vec![wallet_id.clone()],
                delete_without_export: Some(true),
            },
        )
        .await?;

    let deleted_wallets = activity_result.result.wallet_ids;
    assert_eq!(deleted_wallets.len(), 1);
    assert_eq!(deleted_wallets.first().unwrap().to_string(), wallet_id);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load API key
    let api_key = load_api_key_from_env()?;

    // Get organization ID from env
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    // Create our Turnkey client with app proofs enabled
    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?
        .with_app_proofs();

    // Step 1: Create a wallet
    println!("Creating wallet...");

    let activity_result = create_wallet(&client, organization_id.clone()).await?;

    println!(
        "Wallet created successfully! Activity ID: {}, Wallet ID: {}",
        activity_result.activity_id, activity_result.result.wallet_id
    );

    // Step 2: Now that we have the wallet creation activity and result, let's preemptively do cleanup: delete the wallet now before running verification logic so that cleanup happens no matter what
    println!("Preemptively cleaning up: deleting wallet...");

    delete_wallet(
        &client,
        organization_id.clone(),
        activity_result.result.wallet_id.clone(),
    )
    .await?;

    println!("Successfully deleted wallet");

    // Step 3: Get the app proof for this activity. This example creates one wallet account, so there will be at least one app proof.
    assert!(!activity_result.app_proofs.is_empty());

    // Step 4: For each app proof, fetch the corresponding boot proof and verify them
    for app_proof in activity_result.app_proofs {
        // Get boot proof for the app proof
        println!("Fetching boot proof for app proof...");

        let boot_proof_response =
            get_boot_proof_for_app_proof(&client, organization_id.clone(), &app_proof).await?;

        let boot_proof = boot_proof_response
            .boot_proof
            .ok_or("No boot proof found in response")?;

        println!(
            "Found boot proof for ephemeral key: {}",
            boot_proof.ephemeral_public_key_hex
        );

        // Verify the app proof and boot proof
        println!("Verifying app proof and boot proof...");

        verify(&app_proof, &boot_proof).map_err(|e| format!("Verification failed: {e:?}"))?;

        println!(
            "Verification successful! The wallet creation was performed in a secure Turnkey enclave."
        );
    }

    Ok(())
}
