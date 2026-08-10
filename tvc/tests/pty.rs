//! PTY-based integration tests.
//!
//! Drives the real `tvc` binary through a pseudo-terminal so `inquire`'s TTY
//! code path is exercised end-to-end.
//!
//! Gated `#[cfg(unix)]` because `rexpect` uses Unix PTYs; Windows users hit
//! inquire via ConPTY in production, but we don't test that surface here.

#![cfg(unix)]

mod common;

use rexpect::session::PtySession;
use std::path::Path;
use std::process::Command;

/// Default per-step timeout. Generous enough for CI-runner cold cargo builds
/// of the binary; tight enough to fail fast if an `exp_string` mismatches.
const TIMEOUT_MS: u64 = 10_000;

fn spawn(args: &str) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");
    let cmd = format!("{bin} {args}");
    rexpect::spawn(&cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {cmd}"))
}

/// Spawn the binary in a PTY with `HOME` pointed at an isolated directory and
/// ambient `TVC_*` variables scrubbed so developer shells can't leak into the
/// prompts under test.
fn spawn_with_home(home: &Path, args: &[&str]) -> PtySession {
    let bin = env!("CARGO_BIN_EXE_tvc");

    let mut cmd = Command::new(bin);
    cmd.args(args)
        .env("HOME", home)
        .env_remove("TVC_ORG")
        .env_remove("TVC_API_BASE_URL")
        .env_remove("TVC_NON_INTERACTIVE");

    rexpect::session::spawn_command(cmd, Some(TIMEOUT_MS))
        .unwrap_or_else(|e| panic!("spawn failed: {e}\n  cmd: {bin} {}", args.join(" ")))
}

/// `tvc deploy approve` walks all five section confirmations in order and
/// emits the signed approval JSON when the user accepts every section.
///
/// Replaces the deleted `tests/deploy_approve.rs::approve_interactive_prompts`
/// integration test which used piped stdin.
#[test]
fn approve_walks_all_sections_with_yeses() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed-path fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("MANIFEST APPROVAL").unwrap();
    session.exp_string("NAMESPACE").unwrap();
    session.exp_string("turnkey-prod").unwrap();
    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ENCLAVE (AWS Nitro)").unwrap();
    session
        .exp_string("Approve enclave configuration?")
        .unwrap();
    session.send_line("y").unwrap();

    session.exp_string("PIVOT BINARY").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("MANIFEST SET").unwrap();
    session.exp_string("operator-alice").unwrap();
    session.exp_string("Approve manifest set?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("SHARE SET").unwrap();
    session.exp_string("Approve share set?").unwrap();
    session.send_line("y").unwrap();

    session.exp_string("ALL SECTIONS APPROVED").unwrap();
    session.exp_string(r#""signature""#).unwrap();
    session.exp_eof().unwrap();
}

/// Rejecting at the third section (pivot) bails immediately with the exact
/// "operation cancelled by user: approval" string and never reaches the
/// manifest-set section.
#[test]
fn approve_bails_when_user_rejects_pivot() {
    let mut session = spawn(
        "deploy approve \
         --manifest fixtures/manifest.json \
         --operator-seed-path fixtures/seed.hex \
         --skip-post",
    );

    session.exp_string("Approve namespace?").unwrap();
    session.send_line("y").unwrap();
    session
        .exp_string("Approve enclave configuration?")
        .unwrap();
    session.send_line("y").unwrap();
    session.exp_string("Approve pivot binary?").unwrap();
    session.send_line("n").unwrap();

    session
        .exp_string("operation cancelled by user: approval")
        .unwrap();
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

/// Interactive `keys backup-operator-key` prompts for the destination and
/// reports the copy.
#[test]
fn keys_backup_operator_key_prompts_for_destination() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-backup")], Some("alias-a"));
    common::write_profile_key_files(temp.path(), "alias-a");
    let destination = temp.path().join("operator-backup.json");

    let mut session = spawn_with_home(temp.path(), &["keys", "backup-operator-key"]);

    session.exp_string("Backup file path").unwrap();
    session.send_line(destination.to_str().unwrap()).unwrap();

    session.exp_string("Operator key backed up!").unwrap();
    session.exp_eof().unwrap();

    assert!(destination.exists());
}
