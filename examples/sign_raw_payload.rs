use std::{env, process};

use tkhq_rust_sdk::client::{self, SignRawPayload};
use tkhq_rust_sdk::gen::external::activity::v1::SignRawPayloadRequest;
use tkhq_rust_sdk::gen::immutable::activity::v1::result::Inner;
use tkhq_rust_sdk::gen::immutable::activity::v1::{ActivityType, SignRawPayloadIntentV2};
use tkhq_rust_sdk::gen::immutable::common::v1::{HashFunction, PayloadEncoding};

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: sign_raw_payload WALLET_ADDRESS PAYLOAD");
        process::exit(1);
    }
    let wallet_address = args[1].clone();
    let payload = args[1].clone();

    let organization_id = env::var("TURNKEY_ORGANIZATION_ID").unwrap();

    let tk = client::TurnkeyClient::new_from_env().unwrap();

    let timestamp_ms = tk.request_timestamp_ms();
    let req = SignRawPayloadRequest {
        r#type: ActivityType::SignRawPayloadV2.as_str_name().to_owned(),
        timestamp_ms,
        organization_id,
        parameters: Some(SignRawPayloadIntentV2 {
            sign_with: wallet_address,
            payload,
            encoding: PayloadEncoding::TextUtf8 as i32,
            hash_function: HashFunction::Keccak256 as i32,
        }),
    };
    let resp = tk.request::<SignRawPayload>(req).await.unwrap();
    let activity = resp.activity.unwrap();
    match activity.result {
        Some(tkhq_rust_sdk::gen::immutable::activity::v1::Result {
            inner: Some(Inner::SignRawPayloadResult(result)),
        }) => {
            println!("signature: {:?}", result);
        }
        _ => {
            panic!("failed to decode CreateWalletResult");
        }
    }
}
