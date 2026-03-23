mod support;

use predicates::prelude::*;

#[tokio::test]
async fn public_key_prints_openssh_line_from_turnkey_key() {
    let env = support::AuthTestEnv::new();
    let server = support::start_mock_turnkey_server().await;
    support::mount_get_private_key_mock(
        &server,
        "6666666666666666666666666666666666666666666666666666666666666666",
    )
    .await;

    let mut cmd = env.turnkey_command(&server);
    cmd.arg("public-key");

    cmd.assert().success().stdout(predicate::str::contains(
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm",
    ));
}
