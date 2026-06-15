use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use tvc::passkey::{
    WEBAUTHN_STAMP_HEADER_NAME, WebAuthnAssertion, WebAuthnStamper, derive_challenge,
};
use turnkey_api_key_stamper::Stamp as _;

#[test]
fn derives_turnkey_webauthn_challenge_from_raw_request_body() {
    let challenge = derive_challenge(b"{}");

    assert_eq!(
        challenge,
        "NDQxMzZmYTM1NWIzNjc4YTExNDZhZDE2ZjdlODY0OWU5NGZiNGZjMjFmZTc3ZTgzMTBjMDYwZjYxY2FhZmY4YQ"
    );
}

#[test]
fn webauthn_stamper_serializes_protojson_header_without_base64_wrapping() {
    let stamper = WebAuthnStamper::new_for_tests(WebAuthnAssertion {
        credential_id: "credential-id".to_string(),
        client_data_json: "client-data-json".to_string(),
        authenticator_data: "authenticator-data".to_string(),
        signature: "signature".to_string(),
    });

    let header = stamper.stamp(br#"{"organizationId":"org-id"}"#).unwrap();

    assert_eq!(header.name, WEBAUTHN_STAMP_HEADER_NAME);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&header.value).unwrap(),
        serde_json::json!({
            "credentialId": "credential-id",
            "clientDataJson": "client-data-json",
            "authenticatorData": "authenticator-data",
            "signature": "signature"
        })
    );
    assert!(
        !header.value.starts_with("ey"),
        "stamp must not be base64url-wrapped"
    );
}

#[test]
fn passkey_commands_reject_non_interactive_mode() {
    cargo_bin_cmd!("tvc")
        .env_clear()
        .arg("--non-interactive")
        .arg("login")
        .arg("--passkey")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "passkey authentication requires an interactive terminal",
        ));
}

#[test]
fn auth_passkey_help_lists_management_commands() {
    cargo_bin_cmd!("tvc")
        .arg("auth")
        .arg("passkey")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("register"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("remove"));
}
