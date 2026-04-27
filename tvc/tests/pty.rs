//! PTY-based integration tests.
//!
//! Drives the real `tvc` binary through a pseudo-terminal so inquire's TTY
//! code path is exercised end-to-end.
//!
//! Gated `#[cfg(unix)]` because rexpect uses Unix PTYs; Windows users hit
//! inquire via ConPTY in production, but we don't test that surface here.

#![cfg(unix)]

use rexpect::session::PtySession;

/// Default per-step timeout. Generous enough for CI-runner cold cargo builds
/// of the binary; tight enough to fail fast if an `exp_string` mismatches.
const TIMEOUT_MS: u64 = 10_000;

fn spawn(args: &str) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} {args}");
    rexpect::spawn(&cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"))
}

/// `tvc deploy approve` walks all four section confirmations in order and
/// emits the signed approval JSON when the user accepts every section.
///
/// Replaces the deleted `tests/deploy_approve.rs::approve_interactive_prompts`
/// integration test which used piped stdin.
#[test]
fn approve_walks_all_four_sections_with_yeses() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("MANIFEST APPROVAL").unwrap();
    session.exp_string("NAMESPACE").unwrap();
    session.exp_string("turnkey-prod").unwrap();
    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ENCLAVE (AWS Nitro)").unwrap();
    session.exp_string("Approve enclave configuration?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("PIVOT BINARY").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("MANIFEST SET").unwrap();
    session.exp_string("operator-alice").unwrap();
    session.exp_string("Approve manifest set?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ALL SECTIONS APPROVED").unwrap();
    session.exp_string("\"signature\"").unwrap();
    session.exp_eof().unwrap();
}

/// Rejecting at the third section (pivot) bails immediately with the exact
/// "approval cancelled by user" string and never reaches the manifest-set
/// section.
///
/// Replaces the deleted `tests/deploy_approve.rs::approve_interactive_reject`.
#[test]
fn approve_bails_when_user_rejects_pivot() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Approve enclave configuration?").unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("n").unwrap();

    session.exp_string("approval cancelled by user").unwrap();
    session.exp_eof().unwrap();
}

/// Submitting an empty Organization ID at the new-org prompt errors with the
/// exact bail string.
///
/// Replaces the deleted `tests/login.rs::login_empty_org_id_fails`.
#[test]
fn login_with_empty_org_id_bails() {
    let temp = tempfile::TempDir::new().unwrap();

    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} login");

    let mut session = rexpect::session::spawn_command(
        {
            let mut c = std::process::Command::new(bin);
            c.arg("login").env("HOME", temp.path());
            c
        },
        Some(TIMEOUT_MS),
    )
    .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"));

    session.exp_string("Organization ID").unwrap();
    session.send_line("").unwrap();
    session.exp_string("Organization ID is required").unwrap();
    session.exp_eof().unwrap();
}
