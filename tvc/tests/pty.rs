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

/// Drive `tvc login` through the interactive new-org flow (empty config, so
/// no org selector appears) against a dead-port API base URL. Login persists
/// the profile and its generated API key before the final whoami request, so
/// the profile exists on disk even though the command exits nonzero when that
/// request fails.
fn pty_create_profile(home: &Path, org_id: &str, alias: &str) {
    let mut session = spawn_with_home(home, &["login", "--api-base-url", "http://127.0.0.1:1"]);

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

/// TVC-159: interactive `tvc login` folds duplicate profiles down to one per
/// organization before proceeding: prompt for the keeper, one confirmation,
/// full profile-delete cleanup for the losers, and active-profile repair onto
/// the keeper. Duplicate profiles can no longer be created through the CLI,
/// so this seeds the legacy on-disk state directly.
#[test]
fn login_consolidates_duplicate_profiles_interactively() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-b"),
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session
        .exp_string("Select the profile to keep for organization 'org-dup-test'")
        .unwrap();
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Permanently delete profile 'alias-b' and the key files on disk?")
        .unwrap();
    session.send_line("y").unwrap();

    session
        .exp_string("Deleted login profile 'alias-b' (org-dup-test).")
        .unwrap();
    session.exp_string("Removed key directory").unwrap();
    session
        .exp_string("IMPORTANT: The API key may still be registered")
        .unwrap();

    // Login proceeds against the consolidated config.
    session.exp_string("Select organization").unwrap();
    session.send_line("alias-a").unwrap();
    session
        .exp_string("Selected org: alias-a (org-dup-test)")
        .unwrap();
    session.exp_string("Using existing API key.").unwrap();
    session.exp_string("Verifying credentials...").unwrap();
    session.exp_eof().unwrap();

    assert!(!temp.path().join(".config/turnkey/orgs/alias-b").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(!saved.contains("alias-b"));
    assert!(saved.contains(r#"active_org = "alias-a""#));
    assert!(!saved.contains("default_alias"));
}

/// Declining the consolidation confirmation cancels login and leaves both
/// profiles and their key files untouched (nothing is mutated before the
/// single consent point).
#[test]
fn login_consolidation_decline_cancels_login() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(
        temp.path(),
        &[("alias-a", "org-dup-test"), ("alias-b", "org-dup-test")],
        Some("alias-b"),
    );
    common::write_profile_key_files(temp.path(), "alias-a");
    common::write_profile_key_files(temp.path(), "alias-b");

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session
        .exp_string("Select the profile to keep for organization 'org-dup-test'")
        .unwrap();
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Permanently delete profile 'alias-b' and the key files on disk?")
        .unwrap();
    session.send_line("n").unwrap();

    session
        .exp_string("operation cancelled by user: profile consolidation")
        .unwrap();
    session.exp_eof().unwrap();

    assert!(temp.path().join(".config/turnkey/orgs/alias-a").exists());
    assert!(temp.path().join(".config/turnkey/orgs/alias-b").exists());

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("alias-a"));
    assert!(saved.contains("alias-b"));
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

/// The interactive new-org flow (the only way to create a profile) works end
/// to end and persists the profile even though the final whoami request fails
/// against the dead-port URL.
#[test]
fn login_creates_first_profile_and_persists_it() {
    let temp = tempfile::TempDir::new().unwrap();
    pty_create_profile(temp.path(), "org-solo-test", "solo");

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains("[orgs.solo]"));
    assert!(saved.contains(r#"id = "org-solo-test""#));
}

/// Entering an organization ID that is already configured refuses to create a
/// second profile for it (one profile per organization, TVC-159) and names
/// the existing alias.
#[test]
fn login_new_org_refuses_already_configured_org_id() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-dup-test")], Some("alias-a"));

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Select organization").unwrap();
    session.send_line("new").unwrap();

    session.exp_string("Organization ID").unwrap();
    session.send_line("org-dup-test").unwrap();

    session
        .exp_string("Organization 'org-dup-test' is already configured as profile 'alias-a'.")
        .unwrap();
    session
        .exp_string("tvc profile delete --org alias-a")
        .unwrap();
    session.exp_eof().unwrap();

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert_eq!(saved.matches(r#"id = "org-dup-test""#).count(), 1);
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

/// Reusing an existing profile alias for a different organization refuses
/// instead of silently overwriting the profile.
#[test]
fn login_new_org_refuses_alias_already_in_use() {
    let temp = tempfile::TempDir::new().unwrap();
    common::write_profiles_config(temp.path(), &[("alias-a", "org-other-id")], Some("alias-a"));

    let mut session = spawn_with_home(temp.path(), &["login"]);

    session.exp_string("Select organization").unwrap();
    session.send_line("new").unwrap();

    session.exp_string("Organization ID").unwrap();
    session.send_line("org-fresh-id").unwrap();
    session.exp_string("Organization alias").unwrap();
    session.send_line("alias-a").unwrap();

    session
        .exp_string("Profile alias 'alias-a' is already in use for organization 'org-other-id'.")
        .unwrap();
    session.exp_eof().unwrap();

    let saved =
        std::fs::read_to_string(temp.path().join(".config/turnkey/tvc.config.toml")).unwrap();
    assert!(saved.contains(r#"id = "org-other-id""#));
    assert!(!saved.contains("org-fresh-id"));
}
