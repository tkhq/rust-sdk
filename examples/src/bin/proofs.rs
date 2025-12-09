use std::env;
use std::error::Error;
use turnkey_api_key_stamper::Stamp;
use turnkey_client::generated::external::activity::v1 as external_activity;
use turnkey_client::generated::immutable::activity::v1 as immutable_activity;
use turnkey_client::generated::immutable::common::v1::{AddressFormat, Curve, PathFormat};
use turnkey_client::generated::DeleteWalletsIntent;
use turnkey_client::generated::WalletAccountParams;
use turnkey_client::TurnkeyClientError;
use turnkey_examples::load_api_key_from_env;
use turnkey_proofs::{get_boot_proof_for_app_proof, verify};

async fn create_wallet(
    client: &turnkey_client::TurnkeyClient<impl Stamp>,
    organization_id: String,
) -> Result<(external_activity::Activity, String), Box<dyn Error>> {
    let create_wallet_request = external_activity::CreateWalletRequest {
        r#type: "ACTIVITY_TYPE_CREATE_WALLET".to_string(),
        timestamp_ms: client.current_timestamp().to_string(),
        parameters: Some(immutable_activity::CreateWalletIntent {
            wallet_name: "Test Wallet".to_string(),
            accounts: vec![WalletAccountParams {
                curve: Curve::Secp256k1,
                path_format: PathFormat::Bip32,
                path: "m/44'/60'/0'/0".to_string(),
                address_format: AddressFormat::Ethereum,
            }],
            mnemonic_length: None,
        }),
        organization_id,
        generate_app_proofs: client.generate_app_proofs(),
    };

    let activity = client
        .process_activity(
            &create_wallet_request,
            "/public/v1/submit/create_wallet".to_string(),
        )
        .await?;

    let inner = activity
        .result
        .as_ref()
        .ok_or_else(|| TurnkeyClientError::MissingResult)?
        .inner
        .as_ref()
        .ok_or_else(|| TurnkeyClientError::MissingInnerResult)?;

    let create_wallet_result = match inner {
        immutable_activity::result::Inner::CreateWalletResult(res) => res,
        other => {
            return Err(
                TurnkeyClientError::UnexpectedInnerActivityResult(serde_json::to_string(&other)?)
                    .into(),
            )
        }
    };

    let wallet_id = create_wallet_result.wallet_id.clone();

    Ok((activity, wallet_id))
}

async fn delete_wallet(
    client: &turnkey_client::TurnkeyClient<impl Stamp>,
    organization_id: String,
    wallet_id: String,
) -> Result<(), Box<dyn Error>> {
    let delete_wallet_result = client
        .delete_wallets(
            organization_id,
            client.current_timestamp(),
            DeleteWalletsIntent {
                wallet_ids: vec![wallet_id.clone()],
                delete_without_export: Some(true),
            },
        )
        .await?;

    let deleted_wallets = delete_wallet_result.wallet_ids;
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

    let (activity, wallet_id) = create_wallet(&client, organization_id.clone()).await?;

    println!(
        "Wallet created successfully! Activity ID: {}, Wallet ID: {}",
        activity.id, wallet_id
    );

    // Step 2: Now that we have the wallet creation activity and result, let's preemptively do cleanup: delete the wallet now before running verification logic so that cleanup happens no matter what
    println!("Preemptively cleaning up: deleting wallet...");

    delete_wallet(&client, organization_id.clone(), wallet_id.clone()).await?;

    println!("Successfully deleted wallet");

    // Step 3: Get the app proof for this activity. This example creates one wallet account, so there will be at least one app proof.
    assert!(!activity.app_proofs.is_empty());
    let app_proof = activity.app_proofs.first().unwrap();

    // Step 4: Get boot proof for the app proof
    println!("Fetching boot proof for app proof...");

    let boot_proof_response =
        get_boot_proof_for_app_proof(&client, organization_id.clone(), app_proof).await?;

    let boot_proof = boot_proof_response
        .boot_proof
        .ok_or("No boot proof found in response")?;

    println!(
        "Found boot proof for ephemeral key: {}",
        boot_proof.ephemeral_public_key_hex
    );

    // Step 5: Verify the app proof and boot proof
    println!("Verifying app proof and boot proof...");

    verify(app_proof, &boot_proof).map_err(|e| format!("Verification failed: {:?}", e))?;

    println!(
        "Verification successful! The wallet creation was performed in a secure Turnkey enclave."
    );

    Ok(())
}
