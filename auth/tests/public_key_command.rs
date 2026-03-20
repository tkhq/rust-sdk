use assert_cmd::Command;
use predicates::prelude::*;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use wiremock::matchers::{header_exists, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn public_key_prints_openssh_line_from_turnkey_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/public/v1/query/get_private_key"))
        .and(header_exists("X-Stamp"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "privateKey": {
                "privateKeyId": "pk-id",
                "publicKey": "6666666666666666666666666666666666666666666666666666666666666666",
                "privateKeyName": "git signer",
                "curve": "CURVE_ED25519",
                "addresses": [],
                "privateKeyTags": [],
                "createdAt": null,
                "updatedAt": null,
                "exported": false,
                "imported": false
            }
        })))
        .mount(&server)
        .await;

    let api_key = TurnkeyP256ApiKey::generate();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("public-key")
        .env("TURNKEY_ORGANIZATION_ID", "org-id")
        .env(
            "TURNKEY_API_PUBLIC_KEY",
            hex::encode(api_key.compressed_public_key()),
        )
        .env(
            "TURNKEY_API_PRIVATE_KEY",
            hex::encode(api_key.private_key()),
        )
        .env("TURNKEY_PRIVATE_KEY_ID", "pk-id")
        .env("TURNKEY_API_BASE_URL", server.uri());

    cmd.assert().success().stdout(predicate::str::contains(
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm",
    ));
}
