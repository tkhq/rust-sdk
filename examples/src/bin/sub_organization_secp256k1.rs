use std::error::Error;
use std::{env, vec};

use turnkey_client::generated::DeleteSubOrganizationIntent;
use turnkey_client::generated::{
    immutable::common::v1::{AddressFormat, ApiKeyCurve, Curve, PathFormat},
    ApiKeyParamsV2, CreateSubOrganizationIntentV7, RootUserParamsV4, WalletAccountParams,
    WalletParams,
};

use turnkey_examples::load_api_key_from_env;

use turnkey_client::TurnkeySecp256k1ApiKey;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;

    // Sub-org root user gets a SECP256K1 API key:
    let sub_organization_api_key = TurnkeySecp256k1ApiKey::generate();

    // Build intent
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("missing TURNKEY_ORGANIZATION_ID");

    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?;

    let intent = CreateSubOrganizationIntentV7 {
        sub_organization_name: "New sub-organization (secp256k1 root user)".to_string(),
        root_users: vec![RootUserParamsV4 {
            user_name: "Root User (secp256k1)".to_string(),
            api_keys: vec![ApiKeyParamsV2 {
                api_key_name: "Test API Key (k1)".to_string(),
                public_key: hex::encode(sub_organization_api_key.compressed_public_key()),
                curve_type: ApiKeyCurve::Secp256k1,
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
            mnemonic_length: None,
        }),
        // Defaults:
        disable_email_recovery: None,
        disable_email_auth: None,
        disable_sms_auth: None,
        disable_otp_email_auth: None,
    };

    let create_res = client
        .create_sub_organization(organization_id, client.current_timestamp(), intent)
        .await?;

    assert_eq!(create_res.root_user_ids.len(), 1);

    println!(
        "New sub-organization created: {} (root user ID: {})",
        create_res.sub_organization_id,
        create_res.root_user_ids.first().unwrap()
    );

    // Now let's cleanup and delete our sub-organization
    // This needs to be done by the sub-organization user, authenticated by our fresh secp256k1 API key
    let sub_organization_client = turnkey_client::TurnkeyClient::builder()
        .api_key(sub_organization_api_key)
        .build()?;

    let delete_res = sub_organization_client
        .delete_sub_organization(
            create_res.sub_organization_id.clone(),
            client.current_timestamp(),
            DeleteSubOrganizationIntent {
                delete_without_export: Some(true),
            },
        )
        .await?;

    assert_eq!(
        delete_res.sub_organization_uuid,
        create_res.sub_organization_id
    );
    println!("Sub-organization cleaned up and deleted");
    Ok(())
}
