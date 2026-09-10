use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use turnkey_enclave_encrypt::QuorumPublicKey;
use turnkey_examples::{load_api_key_from_env, load_base_url_from_env};

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
        .base_url(load_base_url_from_env())
        .build()?;

    // Secrets are encrypted to a Turnkey enclave, so every call is checked
    // against the signer's quorum public key.
    let signer = QuorumPublicKey::production_signer();

    // Static properties are plaintext metadata: policies can read them, the
    // secret value itself stays encrypted end to end.
    let mut static_properties = BTreeMap::new();
    static_properties.insert("environment".to_string(), "demo".to_string());

    // Secret names are unique within an organization, so stamp this run.
    let run = client.current_timestamp();
    let api_token = "sk-demo-0000000000";
    let webhook_secret = "whsec-demo-1111111111";

    // Import a secret. `import_secret` runs the whole flow: it initializes the
    // import to get a signed enclave target, encrypts to it, and submits.
    let api_token_id = client
        .import_secret(
            organization_id.clone(),
            Some(format!("demo-api-token-{run}")),
            api_token.to_string(),
            static_properties.clone(),
            &signer,
        )
        .await?
        .result;

    let webhook_secret_id = client
        .import_secret(
            organization_id.clone(),
            Some(format!("demo-webhook-secret-{run}")),
            webhook_secret.to_string(),
            static_properties,
            &signer,
        )
        .await?
        .result;

    println!("Imported secrets: {api_token_id}, {webhook_secret_id}");

    // List secret metadata: IDs, names, static properties, creation times.
    // Values never ride this path.
    let listing = client
        .list_secrets(turnkey_client::generated::ListSecretsRequest {
            organization_id: organization_id.clone(),
            pagination_options: None,
        })
        .await?;
    println!("Organization has {} secrets", listing.secrets.len());

    // Export both in one activity. Plaintexts come back in the order the IDs
    // were given, each decrypted with its own single-use transport key.
    let plaintexts = client
        .export_secret(
            organization_id,
            &[&api_token_id, &webhook_secret_id],
            &signer,
        )
        .await?
        .result;

    // What came back out is what went in, in the order the IDs were given.
    assert_eq!(plaintexts, vec![api_token, webhook_secret]);

    for (secret_id, plaintext) in [&api_token_id, &webhook_secret_id].iter().zip(&plaintexts) {
        println!("Exported {secret_id}: {plaintext}");
    }

    Ok(())
}
