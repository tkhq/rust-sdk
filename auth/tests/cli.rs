mod support;

use predicates::prelude::*;

#[test]
fn cli_help_lists_commands() {
    let env = support::AuthTestEnv::new();
    let mut cmd = env.command();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("git-sign"))
        .stdout(predicate::str::contains("public-key"));
}

#[test]
fn public_key_requires_turnkey_org_id() {
    let env = support::AuthTestEnv::new();
    let mut cmd = env.command_without_auth_env();
    cmd.arg("public-key");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("turnkey.organizationId"));
}
