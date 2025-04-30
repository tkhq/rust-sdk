use std::error::Error;
use std::{env, vec};
use tkhq_api_key_stamper::TurnkeyP256ApiKey;
use tkhq_client::generated::DeleteSubOrganizationIntent;
use tkhq_client::generated::{
    immutable::common::v1::{AddressFormat, ApiKeyCurve, Curve, PathFormat},
    ApiKeyParamsV2, CreateSubOrganizationIntentV7, RootUserParamsV4, WalletAccountParams,
    WalletParams,
};
use tkhq_client::TurnkeyClient;
use tkhq_examples::{current_time_ms, load_api_key_from_env};

// See <https://docs.turnkey.com/api-reference/organizations/create-sub-organization> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;

    // In a real scenario this will be the public key associated with an end user, or a passkey, etc.
    // We generate a brand new API key to simulate this.
    let sub_organization_api_key = TurnkeyP256ApiKey::generate();

    // Create the request body for our create_sub_organization request
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    let client = tkhq_client::TurnkeyClient::new(TURNKEY_API_HOST, api_key, None);
    let intent = CreateSubOrganizationIntentV7 {
        sub_organization_name: "New sub-organization".to_string(),
        root_users: vec![RootUserParamsV4 {
            user_name: "Root User".to_string(),
            api_keys: vec![ApiKeyParamsV2 {
                api_key_name: "Test API Key".to_string(),
                public_key: hex::encode(sub_organization_api_key.compressed_public_key()),
                curve_type: ApiKeyCurve::P256,
                expiration_seconds: None,
            }],
            user_email: None,
            user_phone_number: None,
            authenticators: vec![],
            oauth_providers: vec![],
        }],
        root_quorum_threshold: 1,
        wallet: Some(WalletParams {
            wallet_name: "New wallet".to_string(),
            accounts: vec![WalletAccountParams {
                curve: Curve::Secp256k1,
                path_format: PathFormat::Bip32,
                path: "m/44'/60'/0'/0".to_string(),
                address_format: AddressFormat::Ethereum,
            }],
            mnemonic_length: None, // Let that be the default
        }),
        // Defaults
        disable_email_recovery: None,
        disable_email_auth: None,
        disable_sms_auth: None,
        disable_otp_email_auth: None,
    };

    let create_res = client
        .create_sub_organization(organization_id, current_time_ms(), intent)
        .await?;

    assert_eq!(create_res.root_user_ids.len(), 1);

    println!(
        "New sub-organization created: {} (root user ID: {})",
        create_res.sub_organization_id,
        create_res.root_user_ids.first().unwrap()
    );

    // Now let's cleanup and delete our sub-organization
    // This needs to be done by the sub-organization user, authenticated by our fresh API key
    let sub_organization_client =
        TurnkeyClient::new(TURNKEY_API_HOST, sub_organization_api_key, None);
    let delete_res = sub_organization_client
        .delete_sub_organization(
            create_res.sub_organization_id.clone(),
            current_time_ms(),
            DeleteSubOrganizationIntent {
                delete_without_export: Some(true),
            },
        )
        .await?;

    assert_eq!(
        delete_res.sub_organization_uuid,
        create_res.sub_organization_id
    );

    println!("Sub-organization cleaned up and deleted",);

    Ok(())
}
