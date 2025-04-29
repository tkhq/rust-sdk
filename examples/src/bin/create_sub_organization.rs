use std::error::Error;
use std::{env, vec};
use tkhq_client::generated::{
    immutable::common::v1::{AddressFormat, ApiKeyCurve, Curve, PathFormat},
    ApiKeyParamsV2, CreateSubOrganizationIntentV7, RootUserParamsV4, WalletAccountParams,
    WalletParams,
};
use tkhq_examples::{current_time_ms, load_api_key_from_env};

// See <https://docs.turnkey.com/api-reference/organizations/create-sub-organization> for documentation
const TURNKEY_API_HOST: &str = "https://api.turnkey.com";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;

    // We'll reuse the public key we have in our env, for convenience.
    // In a real scenario this will be the public key associated with an end user, or a passkey, etc.
    let suborg_public_key =
        env::var("TURNKEY_API_PUBLIC_KEY").expect("cannot load TURNKEY_API_PUBLIC_KEY");

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
                public_key: suborg_public_key,
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

    let res = client
        .create_sub_organization(organization_id, current_time_ms(), intent)
        .await?;

    assert_eq!(res.root_user_ids.len(), 1);

    println!(
        "New sub-organization created: {} (root user ID: {})",
        res.sub_organization_id,
        res.root_user_ids.first().unwrap()
    );

    Ok(())
}
