use turnkey_api_key_stamper::{Stamp, StampHeader, StamperError};
use turnkey_client::{
    ActivityResult, TurnkeyClient,
    generated::{ActivityStatus, CreateTvcOperatorIntent, CreateTvcOperatorResult},
};

struct CustomStamp;
impl Stamp for CustomStamp {
    fn stamp(&self, _: &[u8]) -> Result<StampHeader, StamperError> {
        Ok(StampHeader {
            name: "X-Custom".into(),
            value: "unchanged".into(),
        })
    }
}

#[test]
fn previous_struct_literals_and_custom_ordinary_method_still_compile() {
    let StampHeader { name, value } = CustomStamp.stamp(b"body").unwrap();
    assert_eq!((name.as_str(), value.as_str()), ("X-Custom", "unchanged"));
    let previous = ActivityResult {
        result: CreateTvcOperatorResult {
            wallet_id: "wallet".into(),
            operator_id: "operator".into(),
            encrypt_public_key: "encrypt".into(),
            sign_public_key: "sign".into(),
        },
        activity_id: "id".into(),
        status: ActivityStatus::Completed,
        app_proofs: vec![],
    };
    assert_eq!(previous.result.wallet_id, "wallet");
    let client = TurnkeyClient::builder()
        .api_key(CustomStamp)
        .build()
        .unwrap();
    let future = client.create_tvc_operator(
        "org".into(),
        100,
        CreateTvcOperatorIntent {
            wallet_id: None,
            wallet_name: Some("wallet".into()),
            path: "p".into(),
            operator_name: "operator".into(),
        },
    );
    drop(future);
}
