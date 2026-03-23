mod support;

use predicates::prelude::*;

#[tokio::test]
async fn git_sign_writes_verifiable_sshsig_file() {
    let env = support::AuthTestEnv::new();
    let fixture = support::create_ssh_fixture(env.path(), b"hello world");
    let raw_signature = support::extract_raw_signature(&fixture.key_path, &fixture.payload_path);
    let server = support::start_mock_turnkey_server().await;
    support::mount_get_private_key_mock(
        &server,
        &hex::encode(&fixture.parsed_public_key.public_key),
    )
    .await;
    support::mount_sign_raw_payload_mock(&server, &raw_signature).await;

    let mut cmd = env.turnkey_command(&server);
    cmd.arg("git-sign")
        .arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&fixture.public_key_path)
        .arg(&fixture.payload_path);

    cmd.assert().success();

    let signature_path = fixture.payload_path.with_extension("txt.sig");
    assert!(signature_path.exists(), "signature file should be created");
    support::verify_signature(
        &fixture.allowed_signers_path,
        &fixture.public_key_line,
        &fixture.payload_path,
        &signature_path,
    );
}

#[tokio::test]
async fn direct_ssh_signer_invocation_writes_verifiable_sshsig_file() {
    let env = support::AuthTestEnv::new();
    let fixture = support::create_ssh_fixture(env.path(), b"hello world");
    let raw_signature = support::extract_raw_signature(&fixture.key_path, &fixture.payload_path);
    let server = support::start_mock_turnkey_server().await;
    support::mount_get_private_key_mock(
        &server,
        &hex::encode(&fixture.parsed_public_key.public_key),
    )
    .await;
    support::mount_sign_raw_payload_mock(&server, &raw_signature).await;

    let mut cmd = env.turnkey_command(&server);
    cmd.arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&fixture.public_key_path)
        .arg(&fixture.payload_path);

    cmd.assert().success();

    let signature_path = fixture.payload_path.with_extension("txt.sig");
    assert!(signature_path.exists(), "signature file should be created");
    support::verify_signature(
        &fixture.allowed_signers_path,
        &fixture.public_key_line,
        &fixture.payload_path,
        &signature_path,
    );
}

#[tokio::test]
async fn git_sign_rejects_public_key_that_does_not_match_configured_turnkey_key() {
    let env = support::AuthTestEnv::new();
    let fixture = support::create_ssh_fixture(env.path(), b"hello world");
    let server = support::start_mock_turnkey_server().await;
    support::mount_get_private_key_mock(
        &server,
        "1111111111111111111111111111111111111111111111111111111111111111",
    )
    .await;

    let mut cmd = env.turnkey_command(&server);
    cmd.arg("git-sign")
        .arg("-Y")
        .arg("sign")
        .arg("-n")
        .arg("git")
        .arg("-f")
        .arg(&fixture.public_key_path)
        .arg(&fixture.payload_path);

    cmd.assert().failure().stderr(predicate::str::contains(
        "does not match the configured Turnkey key",
    ));

    let signature_path = fixture.payload_path.with_extension("txt.sig");
    assert!(
        !signature_path.exists(),
        "signature file should not be created"
    );
}
