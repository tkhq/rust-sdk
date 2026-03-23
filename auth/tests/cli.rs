use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_lists_commands() {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_auth"));
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("config"));
}
