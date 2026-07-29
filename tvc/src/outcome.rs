//! The closed vocabulary of command outcomes.
//!
//! `Outcome` has exactly one variant per command, named after the command.
//! Per-command message structs live in their own command modules; this
//! enum only aggregates and delegates. A command with multiple terminal shapes
//! (e.g. `deploy approve`) owns a command-local enum in its own module, and
//! the top-level variant here wraps it.
//!
//! `reason` strings are stable snake_case discriminators

use crate::commands::deploy::approve::ApproveOutcome;
use crate::commands::{app, deploy, keys, login, operator, version};
use crate::output::Message;
use serde::{Serialize, Serializer};

/// One wide terminal outcome per command (the wide-event model).
///
/// Streaming messages (today: only `deploy debug-logs`'s per-line
/// `debug_log_line`) are emitted inline by their command and are not part of
/// this enum; the command still returns its terminal variant.
pub enum Outcome {
    Login(login::LoggedIn),
    OperatorCreate(operator::create::OperatorCreated),
    ProfileDelete(login::ProfileDeleted),
    DeployApprove(ApproveOutcome),
    DeployGetStatus(deploy::get_status::DeploymentRuntimeStatus),
    DeployProvisioningDetails(deploy::provisioning_details::ProvisioningDetails),
    DeployProvision(deploy::provision::ProvisioningShareCreated),
    DeployPostShare(deploy::post_share::QuorumKeySharePosted),
    DeployStatus(deploy::status::DeploymentStatusReport),
    DeployCreate(deploy::create::DeploymentCreated),
    DeployInit(deploy::init::DeploymentConfigCreated),
    DeployDebugLogs(deploy::debug_logs::DebugLogsFetched),
    DeployDelete(deploy::delete::DeploymentDeleted),
    DeployRestore(deploy::restore::DeploymentRestored),
    AppStatus(app::status::AppStatusReport),
    AppList(app::list::AppsListed),
    AppCreate(app::create::AppCreated),
    AppInit(app::init::AppConfigCreated),
    AppSetLiveDeploy(app::set_live_deploy::LiveDeploymentSet),
    AppDelete(app::delete::AppDeleted),
    KeysCreateQuorumKey(keys::create_quorum_key::QuorumKeyCreated),
    KeysGenerateQuorumKey(keys::generate_local_quorum_key::QuorumKeyGenerated),
    KeysInitQuorumKey(keys::init_local_quorum_key::QuorumKeyConfigCreated),
    KeysReEncryptShare(keys::re_encrypt_local_share::ReEncryptedShareGenerated),
    Version(version::CliVersion),
}

/// Apply `$body` to the message carried by whichever variant `$self` is.
macro_rules! with_message {
    ($self:expr, |$msg:ident| $body:expr) => {
        match $self {
            Outcome::Login($msg) => $body,
            Outcome::OperatorCreate($msg) => $body,
            Outcome::ProfileDelete($msg) => $body,
            Outcome::DeployApprove($msg) => $body,
            Outcome::DeployGetStatus($msg) => $body,
            Outcome::DeployProvisioningDetails($msg) => $body,
            Outcome::DeployProvision($msg) => $body,
            Outcome::DeployPostShare($msg) => $body,
            Outcome::DeployStatus($msg) => $body,
            Outcome::DeployCreate($msg) => $body,
            Outcome::DeployInit($msg) => $body,
            Outcome::DeployDebugLogs($msg) => $body,
            Outcome::DeployDelete($msg) => $body,
            Outcome::DeployRestore($msg) => $body,
            Outcome::AppStatus($msg) => $body,
            Outcome::AppList($msg) => $body,
            Outcome::AppCreate($msg) => $body,
            Outcome::AppInit($msg) => $body,
            Outcome::AppSetLiveDeploy($msg) => $body,
            Outcome::AppDelete($msg) => $body,
            Outcome::KeysCreateQuorumKey($msg) => $body,
            Outcome::KeysGenerateQuorumKey($msg) => $body,
            Outcome::KeysInitQuorumKey($msg) => $body,
            Outcome::KeysReEncryptShare($msg) => $body,
            Outcome::Version($msg) => $body,
        }
    };
}

impl Serialize for Outcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        with_message!(self, |msg| msg.serialize(serializer))
    }
}

impl Message for Outcome {
    /// The single source of truth for the reason vocabulary. Every terminal
    /// reason string is a literal in  this one match, so uniqueness is auditable
    /// at a glance (and, in a follow-up, enforceable by a proc-macro).
    fn reason(&self) -> &'static str {
        match self {
            Outcome::Login(_) => "logged_in",
            Outcome::OperatorCreate(_) => "operator_created",
            Outcome::ProfileDelete(_) => "profile_deleted",
            Outcome::DeployApprove(ApproveOutcome::Posted(_)) => "manifest_approval_posted",
            Outcome::DeployApprove(ApproveOutcome::NotPosted(_)) => "manifest_approval_generated",
            Outcome::DeployApprove(ApproveOutcome::DryRun(_)) => "manifest_approval_dry_run",
            Outcome::DeployGetStatus(_) => "deployment_runtime_status",
            Outcome::DeployProvisioningDetails(_) => "provisioning_details",
            Outcome::DeployProvision(_) => "provisioning_share_created",
            Outcome::DeployPostShare(_) => "quorum_key_share_posted",
            Outcome::DeployStatus(_) => "deployment_status",
            Outcome::DeployCreate(_) => "deployment_created",
            Outcome::DeployInit(_) => "deployment_config_created",
            Outcome::DeployDebugLogs(_) => "debug_logs_fetched",
            Outcome::DeployDelete(_) => "deployment_deleted",
            Outcome::DeployRestore(_) => "deployment_restored",
            Outcome::AppStatus(_) => "app_status",
            Outcome::AppList(_) => "apps_listed",
            Outcome::AppCreate(_) => "app_created",
            Outcome::AppInit(_) => "app_config_created",
            Outcome::AppSetLiveDeploy(_) => "live_deployment_set",
            Outcome::AppDelete(_) => "app_deleted",
            Outcome::KeysCreateQuorumKey(_) => "quorum_key_created",
            Outcome::KeysGenerateQuorumKey(_) => "quorum_key_generated",
            Outcome::KeysInitQuorumKey(_) => "quorum_key_config_created",
            Outcome::KeysReEncryptShare(_) => "re_encrypted_share_generated",
            Outcome::Version(_) => "version",
        }
    }

    fn human_message(&self) -> String {
        // Each inner payload renders itself via `Display`; the terminal outcome
        // just delegates. An empty rendering means the outcome is machine-only.
        with_message!(self, |msg| msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::deploy::approve::{
        ApprovalDryRun, ApprovalGenerated, ApprovalPosted, ApproveOutcome,
    };
    use std::collections::HashSet;

    /// One zero-value instance per terminal shape, in `Outcome`'s declaration
    /// order (command-local shapes expanded in their own declaration order).
    /// The registry tests only read reason strings, so `Default` fixtures
    /// suffice; realistic payloads are exercised by each command's own tests.
    fn all_terminal_shapes() -> Vec<Outcome> {
        vec![
            Outcome::Login(login::LoggedIn::default()),
            Outcome::OperatorCreate(operator::create::OperatorCreated::default()),
            Outcome::ProfileDelete(login::ProfileDeleted::default()),
            Outcome::DeployApprove(ApproveOutcome::Posted(ApprovalPosted::default())),
            Outcome::DeployApprove(ApproveOutcome::NotPosted(ApprovalGenerated::default())),
            Outcome::DeployApprove(ApproveOutcome::DryRun(ApprovalDryRun::default())),
            Outcome::DeployGetStatus(deploy::get_status::DeploymentRuntimeStatus::default()),
            Outcome::DeployProvisioningDetails(
                deploy::provisioning_details::ProvisioningDetails::default(),
            ),
            Outcome::DeployProvision(deploy::provision::ProvisioningShareCreated::default()),
            Outcome::DeployPostShare(deploy::post_share::QuorumKeySharePosted::default()),
            Outcome::DeployStatus(deploy::status::DeploymentStatusReport::default()),
            Outcome::DeployCreate(deploy::create::DeploymentCreated::default()),
            Outcome::DeployInit(deploy::init::DeploymentConfigCreated::default()),
            Outcome::DeployDebugLogs(deploy::debug_logs::DebugLogsFetched::default()),
            Outcome::DeployDelete(deploy::delete::DeploymentDeleted::default()),
            Outcome::DeployRestore(deploy::restore::DeploymentRestored::default()),
            Outcome::AppStatus(app::status::AppStatusReport::default()),
            Outcome::AppList(app::list::AppsListed::default()),
            Outcome::AppCreate(app::create::AppCreated::default()),
            Outcome::AppInit(app::init::AppConfigCreated::default()),
            Outcome::AppSetLiveDeploy(app::set_live_deploy::LiveDeploymentSet::default()),
            Outcome::AppDelete(app::delete::AppDeleted::default()),
            Outcome::KeysCreateQuorumKey(keys::create_quorum_key::QuorumKeyCreated::default()),
            Outcome::KeysGenerateQuorumKey(
                keys::generate_local_quorum_key::QuorumKeyGenerated::default(),
            ),
            Outcome::KeysInitQuorumKey(
                keys::init_local_quorum_key::QuorumKeyConfigCreated::default(),
            ),
            Outcome::KeysReEncryptShare(
                keys::re_encrypt_local_share::ReEncryptedShareGenerated::default(),
            ),
            Outcome::Version(version::CliVersion::default()),
        ]
    }

    /// Reasons that live outside `Outcome`: the `deploy debug-logs` streaming
    /// message and the error envelope reasons
    const NON_TERMINAL_REASONS: [&str; 3] =
        ["debug_log_line", "command_error", "missing_required_input"];

    fn all_reasons() -> Vec<&'static str> {
        let mut reasons: Vec<&str> = all_terminal_shapes().iter().map(Message::reason).collect();
        reasons.extend(NON_TERMINAL_REASONS);
        reasons
    }

    // TODO - proc macro here
    #[test]
    fn all_reasons_are_unique() {
        let reasons = all_reasons();

        let unique: HashSet<&str> = reasons.iter().copied().collect();
        assert_eq!(
            unique.len(),
            reasons.len(),
            "duplicate reason strings in: {reasons:?}"
        );
    }

    #[test]
    fn reasons_are_snake_case() {
        for reason in all_reasons() {
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
