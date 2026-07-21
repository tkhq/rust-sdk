//! The closed vocabulary of command outcomes.
//!
//! `Outcome` has exactly one variant per command, named after the command and
//! ordered to mirror `cli.rs`'s subcommand tree, so the two can be diffed at a
//! glance. Per-command message structs live in their own command modules; this
//! enum only aggregates and delegates. A command with multiple terminal shapes
//! (e.g. `deploy approve`) owns a command-local enum in its own module, and
//! the top-level variant here wraps it.
//!
//! `reason` strings are an external, stable contract: kebab-case, named
//! deliberately, never renamed after release.

use crate::commands::{app, deploy, keys, login};
use crate::output::Message;
use serde::{Serialize, Serializer};

/// One wide terminal outcome per command (the wide-event model).
///
/// Streaming messages (today: only `deploy debug-logs`'s per-line
/// `debug-log-line`) are emitted inline by their command and are not part of
/// this enum; the command still returns its terminal variant.
pub enum Outcome {
    Login(login::LoggedIn),
    ProfileDelete(login::ProfileDeleted),
    DeployApprove(deploy::approve::ApproveOutcome),
    DeployGetStatus(deploy::get_status::DeploymentRuntimeStatus),
    DeployProvisioningDetails(deploy::provisioning_details::ProvisioningDetails),
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
}

/// Apply `$body` to the message carried by whichever variant `$self` is.
macro_rules! with_message {
    ($self:expr, |$msg:ident| $body:expr) => {
        match $self {
            Outcome::Login($msg) => $body,
            Outcome::ProfileDelete($msg) => $body,
            Outcome::DeployApprove($msg) => $body,
            Outcome::DeployGetStatus($msg) => $body,
            Outcome::DeployProvisioningDetails($msg) => $body,
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
        }
    };
}

impl Serialize for Outcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        with_message!(self, |msg| msg.serialize(serializer))
    }
}

impl Message for Outcome {
    fn reason(&self) -> &'static str {
        with_message!(self, |msg| msg.reason())
    }

    fn human_message(&self) -> String {
        with_message!(self, |msg| msg.human_message())
    }

    fn to_json_string(&self) -> String {
        with_message!(self, |msg| msg.to_json_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::deploy::approve::ApproveOutcome;
    use std::collections::HashSet;

    /// One zero-value instance per terminal shape, in `Outcome`'s declaration
    /// order (command-local shapes expanded in their own declaration order).
    /// The registry tests only read reason strings, so `Default` fixtures
    /// suffice; realistic payloads are exercised by each command's own tests.
    fn all_terminal_shapes() -> Vec<Outcome> {
        vec![
            Outcome::Login(login::LoggedIn::default()),
            Outcome::ProfileDelete(login::ProfileDeleted::default()),
            Outcome::DeployApprove(ApproveOutcome::Posted(
                deploy::approve::ApprovalPosted::default(),
            )),
            Outcome::DeployApprove(ApproveOutcome::NotPosted(
                deploy::approve::ApprovalGenerated::default(),
            )),
            Outcome::DeployApprove(ApproveOutcome::DryRun(
                deploy::approve::ApprovalDryRun::default(),
            )),
            Outcome::DeployGetStatus(deploy::get_status::DeploymentRuntimeStatus::default()),
            Outcome::DeployProvisioningDetails(
                deploy::provisioning_details::ProvisioningDetails::default(),
            ),
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
        ]
    }

    /// Reasons that live outside `Outcome`: the `deploy debug-logs` streaming
    /// message and the error envelope reasons from `output.rs`.
    const NON_TERMINAL_REASONS: [&str; 3] =
        ["debug-log-line", "command-error", "missing-required-input"];

    // TODO - proc macro here
    #[test]
    fn all_reasons_are_unique() {
        let mut reasons: Vec<&str> = all_terminal_shapes().iter().map(Message::reason).collect();
        reasons.extend(NON_TERMINAL_REASONS);

        let unique: HashSet<&str> = reasons.iter().copied().collect();
        assert_eq!(
            unique.len(),
            reasons.len(),
            "duplicate reason strings in: {reasons:?}"
        );
    }

    #[test]
    fn reasons_are_kebab_case() {
        for outcome in all_terminal_shapes() {
            let reason = outcome.reason();
            assert!(
                reason
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "reason `{reason}` is not kebab-case"
            );
        }
    }
}
