use coset::iana;
use passkey_types::{
    crypto::sha256,
    webauthn::{
        PublicKeyCredentialParameters, PublicKeyCredentialType, PublicKeyCredentialUserEntity,
    },
};
use std::error::Error;
use std::{env, vec};
use turnkey_client::generated::immutable::activity::v1::DeleteSubOrganizationIntent;
use turnkey_client::generated::{
    immutable::common::v1::{AddressFormat, Curve, PathFormat},
    CreateSubOrganizationIntentV7, RootUserParamsV4, WalletAccountParams, WalletParams,
};
use turnkey_client::generated::{AuthenticatorParamsV2, DeleteSubOrganizationRequest};
use turnkey_examples::load_api_key_from_env;
use turnkey_webauthn_stamper::WebAuthnStamper;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;

    let origin_url = Url::parse("https://example.com")?;
    let mut webauthn_stamper = WebAuthnStamper::new(origin_url);
    let challenge = sha256(b"passkey example").to_vec();
    let credential_params = PublicKeyCredentialParameters {
        ty: PublicKeyCredentialType::PublicKey,
        alg: iana::Algorithm::ES256,
    };
    let user_entity = PublicKeyCredentialUserEntity {
        id: b"user".to_vec().into(),
        name: "passkey user".into(),
        display_name: "User".into(),
    };
    let passkey = webauthn_stamper
        .create_passkey(&challenge, credential_params, user_entity)
        .await
        .unwrap();

    // Create the request body for our create_sub_organization request
    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    let client: turnkey_client::TurnkeyClient<_> = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?;
    let intent: CreateSubOrganizationIntentV7 = CreateSubOrganizationIntentV7 {
        sub_organization_name: "New sub-organization".to_string(),
        root_users: vec![RootUserParamsV4 {
            user_name: "Root User".to_string(),
            api_keys: vec![],
            user_email: None,
            user_phone_number: None,
            authenticators: vec![AuthenticatorParamsV2 {
                authenticator_name: "Test software passkey".to_string(),
                challenge: passkey.encoded_challenge,
                attestation: Some(passkey.attestation),
            }],
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
        verification_token: None,
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

    // delete the created sub-organization to clean up
    let url = format!("https://api.turnkey.com/public/v1/submit/delete_sub_organization");
    let request: DeleteSubOrganizationRequest = DeleteSubOrganizationRequest {
        r#type: "ACTIVITY_TYPE_DELETE_SUB_ORGANIZATION".to_string(),
        timestamp_ms: client.current_timestamp().to_string(),
        parameters: Some(DeleteSubOrganizationIntent {
            delete_without_export: Some(true),
        }),
        organization_id: create_res.sub_organization_id,
    };
    let request_str = serde_json::to_string(&request)?;
    let stamp = webauthn_stamper
        .stamp(request_str.as_bytes())
        .await
        .unwrap();

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("X-Stamp-Webauthn", serde_json::to_string(&stamp)?)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request)?)
        .send()
        .await?;
    assert!(res.status().is_success());

    println!("Deleted the created sub-organization");

    Ok(())
}
