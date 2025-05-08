use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use turnkey_examples::load_api_key_from_env;
use turnkey_proofs::parse_and_verify_aws_nitro_attestation;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetAttestationRequest {
    organization_id: String,
    // One of ("signer", "ump", "evm-parser", "tls-fetcher", "notarizer")
    enclave_type: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetAttestationResponse {
    // base64 encoded string
    attestation_document: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = load_api_key_from_env()?;

    let organization_id =
        env::var("TURNKEY_ORGANIZATION_ID").expect("cannot load TURNKEY_ORGANIZATION_ID");

    let client = turnkey_client::TurnkeyClient::builder()
        .api_key(api_key)
        .build()?;

    let res: GetAttestationResponse = client
        .process_request(
            &GetAttestationRequest {
                organization_id,
                enclave_type: "signer".to_string(),
            },
            "/public/v1/query/get_attestation".to_string(),
        )
        .await?;

    let attestation = parse_and_verify_aws_nitro_attestation(res.attestation_document)
        .expect("cannot parse and verify attestation document");

    println!("Successfully fetched and verified attestation document from Turnkey enclave");
    println!("PCR0: {}", hex::encode(attestation.pcrs.get(&0).unwrap()));
    println!("PCR1: {}", hex::encode(attestation.pcrs.get(&1).unwrap()));
    println!("PCR2: {}", hex::encode(attestation.pcrs.get(&2).unwrap()));
    println!("PCR3: {}", hex::encode(attestation.pcrs.get(&3).unwrap()));
    println!("user_data: {}", hex::encode(attestation.user_data.unwrap()));
    println!(
        "public_key: {}",
        hex::encode(attestation.public_key.unwrap())
    );

    Ok(())
}
