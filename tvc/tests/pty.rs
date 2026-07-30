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

/// Drive `tvc login` through the interactive new-org flow against a dead-port
/// API base URL. Login persists the profile and its generated API key before
/// the final whoami request, so the profile exists on disk even though the
/// command exits nonzero when that request fails.
///
/// `pick_new_in_selector` is required once profiles exist: login then opens an
/// org selector first, and typing "new" filters it down to the
/// "[new] Add a new organization" entry (no fixture string here contains
/// "new") which Enter selects.
fn pty_create_profile(home: &Path, org_id: &str, alias: &str, pick_new_in_selector: bool) {
    let mut session = spawn_with_home(home, &["login", "--api-base-url", "http://127.0.0.1:1"]);

    if pick_new_in_selector {
        session.exp_string("Select organization").unwrap();
        session.send_line("new").unwrap();
    }

    session.exp_string("Organization ID").unwrap();
    session.send_line(org_id).unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line(alias).unwrap();

    session
        .exp_string(&format!("Selected org: {alias} ({org_id})"))
        .unwrap();
    session.exp_string("API Key Generated!").unwrap();
    session.exp_string("Press Enter when done...").unwrap();
    session.send_line("").unwrap();

    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();
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

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Organization ID").unwrap();
    session.send_line("").unwrap();
    session.exp_string("Organization ID is required").unwrap();
    session.exp_eof().unwrap();
}

/// TVC-159: `login --org <org-id>` with several profiles registered for that
/// organization ID prompts for which profile to use instead of resolving to
/// an arbitrary one. The duplicate state is created through the CLI itself,
/// proving the fix handles profiles that predate it.
#[test]
fn login_with_duplicate_org_id_prompts_for_profile() {
    let temp = tempfile::TempDir::new().unwrap();
    pty_create_profile(temp.path(), "org-dup-test", "alias-a", false);
    pty_create_profile(temp.path(), "org-dup-test", "alias-b", true);

    let mut session = spawn_with_home(temp.path(), &["login", "--org", "org-dup-test"]);

    session
        .exp_string("Select profile for organization 'org-dup-test'")
        .unwrap();
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Selected org: alias-a (org-dup-test)")
        .unwrap();
    session.exp_string("Using existing API key.").unwrap();
    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();
}

/// `profile delete --org <org-id>` with several profiles registered for that
/// organization ID prompts for which profile to delete instead of deleting an
/// arbitrary one (TVC-159), and only the chosen profile is removed.
#[test]
fn profile_delete_with_duplicate_org_id_prompts_for_profile() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-a"),
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["profile", "delete", "--org", "org-dup-test"]);

    session.exp_string("Select profile to delete").unwrap();
    session.send_line("alias-b").unwrap();

    session
        .exp_string("Permanently delete profile 'alias-b' (org-dup-test)")
        .unwrap();
    session.send_line("y").unwrap();

    session
        .exp_string("Deleted login profile 'alias-b' (org-dup-test).")
        .unwrap();
    session.exp_eof().unwrap();

    assert!(!temp.path().join(".config/turnkey/orgs/alias-b").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("alias-b"));
    assert!(saved.contains("alias-a"));
}
