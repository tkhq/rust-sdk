//! The closed vocabulary of command outcomes.
//!
//! `Outcome` has exactly one variant per terminal shape, and the variant name
//! IS the wire `reason`: serde's internal tagging stamps
//! `"reason": "<variant_in_snake_case>"` onto every serialized outcome, so
//! the vocabulary cannot drift from the type and two shapes cannot share a
//! reason without rustc rejecting the duplicate variant name. Do not add
//! per-variant `#[serde(rename)]` overrides — that equality is the guarantee.
//!
//! Payload structs live in their own command modules. A command with multiple
//! terminal shapes (e.g. `deploy approve`) owns a command-local enum plus a
//! `From` impl mapping it onto these variants.
//!
//! `reason` strings are stable snake_case discriminators; renaming a variant
//! is a breaking change to the JSON contract.

use crate::commands::deploy::approve::{
    ApprovalAlreadyPosted, ApprovalDryRun, ApprovalGenerated, ApprovalPosted,
};
use crate::commands::{app, deploy, keys, login, operator, secrets, version};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// One wide terminal outcome per command invocation (the wide-event model).
///
/// Streaming messages (today: only `deploy debug-logs`'s per-line
/// `debug_log_line`) are emitted inline by their command and are not part of
/// this enum; the command still returns its terminal variant.
#[derive(Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
#[cfg_attr(test, derive(strum::EnumIter))]
pub enum Outcome {
    LoggedIn(login::LoggedIn),
    OperatorCreated(operator::create::OperatorCreated),
    ProfileDeleted(login::ProfileDeleted),
    ManifestApprovalPosted(ApprovalPosted),
    ManifestApprovalGenerated(ApprovalGenerated),
    ManifestApprovalAlreadyPosted(ApprovalAlreadyPosted),
    ManifestApprovalDryRun(ApprovalDryRun),
    DeploymentRuntimeStatus(deploy::get_status::DeploymentRuntimeStatus),
    ProvisioningDetails(deploy::provisioning_details::ProvisioningDetails),
    ProvisioningShareCreated(deploy::provision::ProvisioningShareCreated),
    QuorumKeySharePosted(deploy::post_share::QuorumKeySharePosted),
    DeploymentStatus(deploy::status::DeploymentStatusReport),
    DeploymentCreated(deploy::create::DeploymentCreated),
    DeploymentConfigCreated(deploy::init::DeploymentConfigCreated),
    DebugLogsFetched(deploy::debug_logs::DebugLogsFetched),
    DeploymentDeleted(deploy::delete::DeploymentDeleted),
    DeploymentRestored(deploy::restore::DeploymentRestored),
    AppStatus(app::status::AppStatusReport),
    AppsListed(app::list::AppsListed),
    AppCreated(app::create::AppCreated),
    AppConfigCreated(app::init::AppConfigCreated),
    LiveDeploymentSet(app::set_live_deploy::LiveDeploymentSet),
    AppDeleted(app::delete::AppDeleted),
    OperatorKeyBackedUp(keys::backup_operator_key::OperatorKeyBackedUp),
    QuorumKeyCreated(keys::create_quorum_key::QuorumKeyCreated),
    QuorumKeyGenerated(keys::generate_local_quorum_key::QuorumKeyGenerated),
    QuorumKeyConfigCreated(keys::init_local_quorum_key::QuorumKeyConfigCreated),
    ReEncryptedShareGenerated(keys::re_encrypt_local_share::ReEncryptedShareGenerated),
    SecretImported(secrets::import::SecretImported),
    SecretExported(secrets::export::SecretExported),
    SecretsListed(secrets::list::SecretsListed),
    Version(version::CliVersion),
}

impl Display for Outcome {
    /// Each payload renders itself; the terminal outcome just delegates. An
    /// empty rendering means the outcome is machine-only.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Outcome::LoggedIn(msg) => msg.fmt(f),
            Outcome::OperatorCreated(msg) => msg.fmt(f),
            Outcome::ProfileDeleted(msg) => msg.fmt(f),
            Outcome::ManifestApprovalPosted(msg) => msg.fmt(f),
            Outcome::ManifestApprovalGenerated(msg) => msg.fmt(f),
            Outcome::ManifestApprovalAlreadyPosted(msg) => msg.fmt(f),
            Outcome::ManifestApprovalDryRun(msg) => msg.fmt(f),
            Outcome::DeploymentRuntimeStatus(msg) => msg.fmt(f),
            Outcome::ProvisioningDetails(msg) => msg.fmt(f),
            Outcome::ProvisioningShareCreated(msg) => msg.fmt(f),
            Outcome::QuorumKeySharePosted(msg) => msg.fmt(f),
            Outcome::DeploymentStatus(msg) => msg.fmt(f),
            Outcome::DeploymentCreated(msg) => msg.fmt(f),
            Outcome::DeploymentConfigCreated(msg) => msg.fmt(f),
            Outcome::DebugLogsFetched(msg) => msg.fmt(f),
            Outcome::DeploymentDeleted(msg) => msg.fmt(f),
            Outcome::DeploymentRestored(msg) => msg.fmt(f),
            Outcome::AppStatus(msg) => msg.fmt(f),
            Outcome::AppsListed(msg) => msg.fmt(f),
            Outcome::AppCreated(msg) => msg.fmt(f),
            Outcome::AppConfigCreated(msg) => msg.fmt(f),
            Outcome::LiveDeploymentSet(msg) => msg.fmt(f),
            Outcome::AppDeleted(msg) => msg.fmt(f),
            Outcome::OperatorKeyBackedUp(msg) => msg.fmt(f),
            Outcome::QuorumKeyCreated(msg) => msg.fmt(f),
            Outcome::QuorumKeyGenerated(msg) => msg.fmt(f),
            Outcome::QuorumKeyConfigCreated(msg) => msg.fmt(f),
            Outcome::ReEncryptedShareGenerated(msg) => msg.fmt(f),
            Outcome::SecretImported(msg) => msg.fmt(f),
            Outcome::SecretExported(msg) => msg.fmt(f),
            Outcome::SecretsListed(msg) => msg.fmt(f),
            Outcome::Version(msg) => msg.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    /// Reasons that live outside `Outcome`: the `deploy debug-logs` streaming
    /// message and the error envelope reasons.
    const NON_TERMINAL_REASONS: [&str; 3] =
        ["debug_log_line", "command_error", "missing_required_input"];

    // TODO - proc macro here
    #[test]
    fn terminal_reasons_do_not_collide_with_non_terminal_reasons() {
        for outcome in Outcome::iter() {
            let value = serde_json::to_value(&outcome)
                .expect("every outcome payload must serialize as a JSON map");
            let reason = value["reason"]
                .as_str()
                .expect("every serialized outcome must carry a `reason` tag");

            assert!(
                !NON_TERMINAL_REASONS.contains(&reason),
                "terminal reason `{reason}` collides with a non-terminal reason"
            );
        }
    }

    #[test]
    fn reasons_are_snake_case() {
        let terminal_reasons = Outcome::iter().map(|outcome| {
            serde_json::to_value(outcome)
                .expect("every outcome payload must serialize as a JSON map")["reason"]
                .as_str()
                .expect("every serialized outcome must carry a `reason` tag")
                .to_string()
        });

        for reason in terminal_reasons.chain(NON_TERMINAL_REASONS.map(String::from)) {
            assert!(
                !reason.is_empty()
                    && reason.split('_').all(|word| {
                        !word.is_empty()
                            && word
                                .chars()
                                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                    }),
                "reason `{reason}` is not snake_case"
            );
        }
    }
}
