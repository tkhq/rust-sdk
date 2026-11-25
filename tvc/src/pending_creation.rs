//! Save the original signed request's fingerprint and intent before each hosted
//! creation attempt. If the activity completes later, verify-result can verify the
//! result against that saved evidence before accepting any keys.

pub(crate) mod unverified;

use crate::{
    client::{AuthenticatedClient, build_client},
    commands::operator::create::save_hosted_operator,
    config::turnkey::{Config, config_dir},
    errors::MissingResource,
    operator::ensure_authenticated_org,
    outcome::Outcome,
    output::StdCtx,
    terminal::TerminalText,
};
use anyhow::{Context, Result, ensure};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;
use turnkey_client::{
    ActivityReceipt, ReceiptCreationError, SubmissionCheckpoint, SubmissionFailureCause,
    SubmissionOutcome, SubmissionPendingReason, SubmissionState, SubmittedActivityResult,
    TurnkeyClient, TurnkeyClientError, TurnkeyP256ApiKey,
    generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent, GetActivityRequest},
};
use turnkey_proofs::tvc::{
    MAX_QUORUM_RECIPIENTS, OperatorExpectation, P256PublicKey, QuorumExpectation, TvcProvenance,
    TvcProvenanceError,
};
use uuid::Uuid;

/// The original request and any local registration options needed after verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum CreationIntent {
    Operator {
        intent: CreateTvcOperatorIntent,
        org_alias: String,
        make_default: bool,
    },
    #[serde(rename = "quorum")]
    QuorumKey { intent: CreateTvcQuorumKeyIntent },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CreationKind {
    #[default]
    Operator,
    QuorumKey,
}

impl CreationKind {
    fn verify_result_command(self) -> &'static str {
        match self {
            Self::Operator => "tvc operator create verify-result",
            Self::QuorumKey => "tvc keys create-quorum-key verify-result",
        }
    }
}

impl CreationIntent {
    fn kind(&self) -> CreationKind {
        match self {
            Self::Operator { .. } => CreationKind::Operator,
            Self::QuorumKey { .. } => CreationKind::QuorumKey,
        }
    }
}

/// The local evidence and intent needed to verify the result of a completed creation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedCreation {
    version: u32,
    api_base_url: String,
    creation: CreationIntent,
    checkpoint: SubmissionCheckpoint,
}

/// A creation request with the expectations checked against its returned proofs.
pub(crate) struct PreparedCreation(PreparedIntent);

/// The exact request to send alongside the expectations used to verify its output.
enum PreparedIntent {
    Operator {
        intent: CreateTvcOperatorIntent,
        org_alias: String,
        make_default: bool,
        expected: OperatorExpectation,
    },
    Quorum {
        intent: CreateTvcQuorumKeyIntent,
        expected: QuorumExpectation,
    },
}

impl PreparedCreation {
    pub(crate) fn operator(
        organization_id: Uuid,
        intent: CreateTvcOperatorIntent,
        org_alias: String,
        make_default: bool,
    ) -> Result<Self> {
        let expected = OperatorExpectation::parse(
            organization_id.to_string(),
            intent.wallet_id.clone(),
            intent.path.clone(),
        )?;
        Ok(Self(PreparedIntent::Operator {
            intent,
            org_alias,
            make_default,
            expected,
        }))
    }

    pub(crate) fn quorum(
        organization_id: Uuid,
        threshold: u32,
        recipients: Vec<P256PublicKey>,
    ) -> Result<Self> {
        let intent = CreateTvcQuorumKeyIntent {
            threshold,
            operator_encrypt_keys: recipients.iter().map(ToString::to_string).collect(),
        };
        let expected = QuorumExpectation::new(organization_id.to_string(), threshold, recipients)?;
        Ok(Self(PreparedIntent::Quorum { intent, expected }))
    }
}

impl From<&PreparedCreation> for CreationIntent {
    fn from(creation: &PreparedCreation) -> Self {
        match &creation.0 {
            PreparedIntent::Operator {
                intent,
                org_alias,
                make_default,
                expected: _,
            } => Self::Operator {
                intent: intent.clone(),
                org_alias: org_alias.clone(),
                make_default: *make_default,
            },
            PreparedIntent::Quorum {
                intent,
                expected: _,
            } => Self::QuorumKey {
                intent: intent.clone(),
            },
        }
    }
}

/// Identifies one creation's receipt file, even when the service returns no activity ID.
struct Operation {
    id: Uuid,
    path: PathBuf,
}

impl Operation {
    fn open(id: Uuid) -> Result<Self> {
        Ok(Self::at(&config_dir()?.join("pending-creations"), id))
    }

    fn at(directory: &Path, id: Uuid) -> Self {
        Self {
            id,
            path: directory.join(format!("{id}.json")),
        }
    }

    /// Create the receipt directory using TVC's regular config parent.
    async fn prepare_directory(&self) -> Result<()> {
        let directory = self
            .path
            .parent()
            .expect("operation path has a parent")
            .to_owned();
        spawn_blocking(move || {
            if let Some(parent) = directory.parent() {
                fs::create_dir_all(parent)?;
            }
            match fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => File::open(directory.parent().expect("receipt directory has a parent"))?
                    .sync_all()?,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => (),
                Err(error) => return Err(error.into()),
            }
            let metadata = fs::symlink_metadata(directory)?;
            ensure!(
                metadata.is_dir() && metadata.permissions().mode() & 0o022 == 0,
                "saved creation directory must be a directory writable only by its owner"
            );
            Ok(())
        })
        .await
        .context("saved creation directory preparation task failed")?
    }

    /// Load the saved intent and fingerprints for result verification. Reject an
    /// unsupported receipt version or a receipt whose file or parent directory
    /// is a symlink or allows group or other users to write.
    async fn load(&self) -> Result<SavedCreation> {
        let directory = self.path.parent().expect("operation path has a parent");
        let parent = tokio::fs::symlink_metadata(directory)
            .await
            .with_context(|| {
                format!(
                    "failed to inspect saved creation directory {}",
                    directory.display()
                )
            })?;
        let file = tokio::fs::symlink_metadata(&self.path)
            .await
            .with_context(|| format!("failed to inspect saved creation {}", self.id))?;
        ensure!(
            parent.is_dir()
                && file.is_file()
                && (parent.permissions().mode() | file.permissions().mode()) & 0o022 == 0,
            "saved creation {} must be a regular file in a directory writable only by its owner",
            self.id
        );
        let saved: SavedCreation =
            crate::util::read_json_file(&self.path, "saved creation").await?;
        ensure!(
            saved.version == 1,
            "unsupported saved creation version {}",
            saved.version
        );
        Ok(saved)
    }

    // A failed update must leave the previous fingerprints available for recovery.
    async fn save(&self, saved: &impl Serialize) -> Result<()> {
        let path = self.path.clone();
        let contents = serde_json::to_vec_pretty(saved)?;
        spawn_blocking(move || {
            let directory = path.parent().expect("operation path has a parent");
            let mut file = NamedTempFile::new_in(directory)?;
            file.write_all(&contents)?;
            file.as_file().sync_all()?;
            file.persist(&path).map_err(|error| error.error)?;
            File::open(directory)?.sync_all()?;
            Ok(())
        })
        .await
        .context("saved creation write task failed")?
    }

    /// Submit the creation request, saving its intent and fingerprint before
    /// each send so the CLI can verify the result even after a restart.
    async fn submit(
        self,
        ctx: &mut StdCtx,
        config: Config,
        auth: CreationClient,
        creation: PreparedCreation,
        timestamp_ms: u128,
        verbose_proofs: bool,
    ) -> Result<Outcome> {
        self.prepare_directory()
            .await
            .context("failed to prepare the saved creation directory")?;
        let operation_id = self.id;
        let intent = CreationIntent::from(&creation);
        let kind = intent.kind();
        let api_base_url = auth.api_base_url.clone();
        let save = async move |checkpoint: &SubmissionCheckpoint| {
            let saved = SavedCreation {
                version: 1,
                api_base_url: api_base_url.clone(),
                creation: intent.clone(),
                checkpoint: checkpoint.clone(),
            };
            self.save(&saved).await.with_context(|| {
                format!(
                    "failed to save creation {} at {}",
                    self.id,
                    self.path.display()
                )
            })
        };

        let result = match &creation.0 {
            PreparedIntent::Operator { intent, .. } => auth
                .client()
                .create_tvc_operator_with_receipt(auth.org_id, timestamp_ms, intent.clone(), save)
                .await
                .map(receipt_outcome),
            PreparedIntent::Quorum { intent, .. } => auth
                .client()
                .create_tvc_quorum_key_with_receipt(auth.org_id, timestamp_ms, intent.clone(), save)
                .await
                .map(receipt_outcome),
        };

        let recovery = Recovery {
            kind,
            operation_id,
            activity_id: None,
            resumed: false,
        };
        let (outcome, recovery) = recovery.resolve(result)?;
        creation
            .finish(ctx, config, auth, outcome, recovery, verbose_proofs)
            .await
    }
}

/// By default, the CLI verifies that the Signer produced the returned keys for
/// the saved request. Callers who explicitly skip verification receive keys
/// labeled unverified.
enum CreationVerification {
    Verify(TvcProvenance<TurnkeyP256ApiKey>),
    Skip(TurnkeyClient<TurnkeyP256ApiKey>),
}

/// An authenticated creation client with the caller's chosen verification mode.
pub(crate) struct CreationClient {
    verification: CreationVerification,
    pub(crate) org_id: Uuid,
    pub(crate) api_base_url: String,
}

impl CreationClient {
    pub(crate) fn new(
        auth: AuthenticatedClient,
        dangerous_skip_verification: bool,
    ) -> Result<Self> {
        let AuthenticatedClient {
            client,
            org_id,
            api_base_url,
        } = auth;
        let org_id =
            Uuid::parse_str(&org_id).context("authenticated organization ID must be a UUID")?;
        let verification = if dangerous_skip_verification {
            CreationVerification::Skip(client)
        } else {
            CreationVerification::Verify(TvcProvenance::try_from((
                client.with_app_proofs(),
                api_base_url.as_str(),
            ))?)
        };
        Ok(Self {
            verification,
            org_id,
            api_base_url,
        })
    }

    fn client(&self) -> &TurnkeyClient<TurnkeyP256ApiKey> {
        match &self.verification {
            CreationVerification::Verify(provenance) => provenance.client(),
            CreationVerification::Skip(client) => client,
        }
    }

    pub(crate) fn ensure_configured_org(&self, configured_org_id: &str) -> Result<()> {
        let configured_id = Uuid::parse_str(configured_org_id)
            .context("configured organization ID must be a UUID")?;
        ensure_authenticated_org(self.org_id, configured_id)
    }
}

impl TryFrom<AuthenticatedClient> for CreationClient {
    type Error = anyhow::Error;

    fn try_from(auth: AuthenticatedClient) -> Result<Self> {
        Self::new(auth, false)
    }
}

/// Start a hosted creation with a fresh local operation ID for its receipt.
pub(crate) async fn submit(
    ctx: &mut StdCtx,
    config: Config,
    auth: CreationClient,
    creation: PreparedCreation,
    timestamp_ms: u128,
    verbose_proofs: bool,
) -> Result<Outcome> {
    let mut bytes = [0; 16];
    OsRng
        .try_fill_bytes(&mut bytes)
        .context("failed to generate local creation UUID")?;
    let operation = Operation::open(uuid::Builder::from_random_bytes(bytes).into_uuid())?;
    operation
        .submit(ctx, config, auth, creation, timestamp_ms, verbose_proofs)
        .await
}

pub(crate) async fn verify_result(
    ctx: &mut StdCtx,
    config: Config,
    operation_id: Uuid,
    activity_id: String,
    kind: CreationKind,
    verbose_proofs: bool,
) -> Result<Outcome> {
    let operation = Operation::open(operation_id)?;
    let saved = operation.load().await?;
    saved.check_kind(kind)?;
    let auth = build_client(&config).await?;
    saved
        .verify_result(ctx, config, &operation, auth, activity_id, verbose_proofs)
        .await
}

/// Look up an activity in the authenticated organization without using a local receipt.
pub(crate) async fn lookup_unverified(
    ctx: &mut StdCtx,
    config: &Config,
    activity_id: String,
    kind: CreationKind,
    verbose_proofs: bool,
) -> Result<Outcome> {
    let auth = build_client(config).await?;
    let org_id =
        Uuid::parse_str(&auth.org_id).context("authenticated organization ID must be a UUID")?;
    let response = auth
        .client
        .get_activity(GetActivityRequest {
            organization_id: auth.org_id,
            activity_id: activity_id.clone(),
        })
        .await?;
    let activity = response
        .activity
        .ok_or_else(|| MissingResource::new("activity", &activity_id))?;
    let outcome = unverified::result(activity, kind, org_id, &activity_id)?;
    crate::proofs::present_skipped(ctx.shell(), verbose_proofs);
    Ok(outcome)
}

type ReceiptResult =
    Result<SubmissionOutcome<ActivityReceipt>, ReceiptCreationError<anyhow::Error>>;

fn receipt_outcome<T>(
    outcome: SubmissionOutcome<SubmittedActivityResult<T>>,
) -> SubmissionOutcome<ActivityReceipt> {
    match outcome {
        SubmissionOutcome::Completed(submitted) => {
            SubmissionOutcome::Completed(submitted.into_parts().1)
        }
        SubmissionOutcome::Pending { checkpoint, reason } => {
            SubmissionOutcome::Pending { checkpoint, reason }
        }
    }
}

impl SavedCreation {
    fn check_kind(&self, kind: CreationKind) -> Result<()> {
        ensure!(
            self.creation.kind() == kind,
            "this saved operation belongs to `{}`",
            self.creation.kind().verify_result_command(),
        );
        Ok(())
    }

    async fn verify_result(
        self,
        ctx: &mut StdCtx,
        config: Config,
        operation: &Operation,
        auth: AuthenticatedClient,
        activity_id: String,
        verbose_proofs: bool,
    ) -> Result<Outcome> {
        let Self {
            version: _,
            api_base_url,
            creation,
            checkpoint,
        } = self;
        ensure!(
            auth.api_base_url == api_base_url,
            "saved creation belongs to a different API environment"
        );
        let auth = CreationClient::try_from(auth)?;
        let organization_id = checkpoint.organization_id();
        ensure!(
            auth.org_id == organization_id,
            "saved creation belongs to a different organization"
        );

        let kind = creation.kind();
        let creation = match creation {
            CreationIntent::Operator {
                intent,
                org_alias,
                make_default,
            } => PreparedCreation::operator(organization_id, intent, org_alias, make_default)?,
            CreationIntent::QuorumKey { intent } => {
                // Bound the number of curve point parses from the saved intent.
                ensure!(
                    intent.operator_encrypt_keys.len() <= MAX_QUORUM_RECIPIENTS,
                    TvcProvenanceError::InvalidExpectation(
                        "require 2 <= threshold <= recipients <= 255",
                    )
                );
                let recipients = intent
                    .operator_encrypt_keys
                    .iter()
                    .map(|key| {
                        key.parse::<P256PublicKey>().map_err(|_| {
                            TvcProvenanceError::InvalidExpectation("invalid recipient public key")
                        })
                    })
                    .collect::<std::result::Result<_, _>>()?;
                PreparedCreation::quorum(organization_id, intent.threshold, recipients)?
            }
        };
        let result = auth
            .client()
            .resume_submission(checkpoint, Some(activity_id.clone()))
            .await
            .map_err(ReceiptCreationError::Client);
        let recovery = Recovery {
            kind,
            operation_id: operation.id,
            activity_id: Some(activity_id),
            resumed: true,
        };
        let (outcome, recovery) = recovery.resolve(result)?;
        creation
            .finish(ctx, config, auth, outcome, recovery, verbose_proofs)
            .await
    }
}

/// The IDs and command name used to guide the user back to their saved receipt.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Recovery {
    #[serde(skip)]
    kind: CreationKind,
    operation_id: Uuid,
    activity_id: Option<String>,
    #[serde(skip)]
    resumed: bool,
}

impl Recovery {
    fn resumed_activity(&self) -> Option<crate::proofs::ResumedActivity<'_>> {
        let activity_id = self.activity_id.as_deref()?;
        self.resumed.then_some(crate::proofs::ResumedActivity {
            operation_id: self.operation_id,
            activity_id,
        })
    }

    fn resolve(self, result: ReceiptResult) -> Result<(SubmissionOutcome<ActivityReceipt>, Self)> {
        let activity_id = match &result {
            Ok(SubmissionOutcome::Pending { checkpoint, .. }) => checkpoint.activity_id(),
            Ok(SubmissionOutcome::Completed(receipt)) => receipt.checkpoint().activity_id(),
            Err(error) => error.activity_id(),
        };
        let recovery = Self {
            activity_id: self.activity_id.or_else(|| activity_id.map(str::to_owned)),
            ..self
        };
        match result {
            Ok(outcome) => Ok((outcome, recovery)),
            Err(source) => Err(CreationFailure { source, recovery }.into()),
        }
    }
}

fn activity_id_argument(candidate: Option<&str>) -> Option<String> {
    let id = candidate.filter(|id| !id.is_empty() && TerminalText(*id).single_line() == *id)?;
    if id
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        && !id.starts_with('-')
    {
        Some(format!("--activity-id {id}"))
    } else {
        Some(format!("--activity-id='{}'", id.replace('\'', "'\\''")))
    }
}

impl Display for Recovery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let argument = activity_id_argument(self.activity_id.as_deref());
        write!(
            f,
            "Verify with: {} {} {}",
            self.kind.verify_result_command(),
            self.operation_id,
            argument.as_deref().unwrap_or("--activity-id <candidate>")
        )?;
        if argument.is_none() {
            write!(
                f,
                "\nReplace <candidate> with the activity ID from Turnkey."
            )?;
        }
        Ok(())
    }
}

/// Keeps the SDK error available for classification and shows recovery once.
#[derive(Debug)]
pub(crate) struct CreationFailure {
    source: ReceiptCreationError<anyhow::Error>,
    recovery: Recovery,
}

impl Display for CreationFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let (state, activity_failed) = match &self.source {
            ReceiptCreationError::Client(TurnkeyClientError::SubmissionFailed(failure)) => {
                write!(f, "{}.", failure.cause())?;
                (
                    failure.submission_state(),
                    *failure.cause() == SubmissionFailureCause::ActivityNotCompleted,
                )
            }
            ReceiptCreationError::Client(error) => {
                return write!(f, "Creation failed: {error}\n{}", self.recovery);
            }
            ReceiptCreationError::CheckpointSave { source, state, .. } => {
                write!(f, "submission checkpoint persistence failed.\n{source:#}")?;
                (*state, false)
            }
        };
        match state {
            SubmissionState::NeverSubmitted => write!(f, "\nNo remote creation request was sent."),
            SubmissionState::MayHaveSubmitted => {
                write!(
                    f,
                    "\nNo keys were accepted or saved. Operation ID: {}",
                    self.recovery.operation_id
                )?;
                if activity_failed {
                    return Ok(());
                }
                write!(
                    f,
                    "\nRemote creation may have succeeded. Do not rerun creation blindly.\n{}",
                    self.recovery
                )
            }
        }
    }
}

impl std::error::Error for CreationFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.source {
            ReceiptCreationError::Client(error) => Some(error),
            ReceiptCreationError::CheckpointSave { source, .. } => Some(source.as_ref()),
        }
    }
}

impl PreparedCreation {
    /// Reports pending creations and verifies completed keys against the original request.
    /// When verification is explicitly skipped, labels completed results as unverified.
    async fn finish(
        self,
        ctx: &mut StdCtx,
        config: Config,
        auth: CreationClient,
        outcome: SubmissionOutcome<ActivityReceipt>,
        recovery: Recovery,
        verbose_proofs: bool,
    ) -> Result<Outcome> {
        let provenance = match auth.verification {
            CreationVerification::Verify(provenance) => provenance,
            CreationVerification::Skip(_) => {
                let outcome = match outcome {
                    SubmissionOutcome::Pending { checkpoint, reason } => {
                        unverified::pending(recovery.kind, checkpoint.activity_id(), reason)
                    }
                    SubmissionOutcome::Completed(receipt) => {
                        let (activity, checkpoint) = receipt.into_parts();
                        let activity_id = checkpoint
                            .activity_id()
                            .context("missing or invalid activity ID")?;
                        unverified::result(activity, recovery.kind, auth.org_id, activity_id)?
                    }
                };
                crate::proofs::present_skipped(ctx.shell(), verbose_proofs);
                return Ok(outcome);
            }
        };
        match outcome {
            SubmissionOutcome::Pending { reason, .. } => {
                Ok(Outcome::CreationPending(CreationPending {
                    recovery,
                    status: match reason {
                        SubmissionPendingReason::Processing => "processing",
                        SubmissionPendingReason::ConsensusNeeded => "consensus_needed",
                        SubmissionPendingReason::AuthenticatorsNeeded => "authenticators_needed",
                    }
                    .to_owned(),
                }))
            }
            SubmissionOutcome::Completed(receipt) => match self.0 {
                PreparedIntent::Operator {
                    intent,
                    org_alias,
                    make_default,
                    expected,
                } => {
                    let organization_id = receipt.checkpoint().organization_id();
                    let acceptance = provenance.accept_operator(expected, receipt).await;
                    let verified = crate::proofs::present(
                        ctx.shell(),
                        acceptance,
                        verbose_proofs,
                        recovery.resumed_activity(),
                    )
                    .context("proof acceptance failed; no keys were saved")
                    .with_context(|| recovery.to_string())?;
                    let hosted = verified.try_into().with_context(|| recovery.to_string())?;
                    save_hosted_operator(
                        config,
                        &org_alias,
                        organization_id,
                        &auth.api_base_url,
                        intent.operator_name,
                        make_default,
                        hosted,
                    )
                    .await
                    .map(Outcome::OperatorCreated)
                    .with_context(|| recovery.to_string())
                }
                PreparedIntent::Quorum {
                    intent: _,
                    expected,
                } => {
                    let acceptance = provenance.accept_quorum(expected, receipt).await;
                    let verified = crate::proofs::present(
                        ctx.shell(),
                        acceptance,
                        verbose_proofs,
                        recovery.resumed_activity(),
                    )
                    .context("proof acceptance failed; no keys were accepted")
                    .with_context(|| recovery.to_string())?;
                    Ok(Outcome::QuorumKeyCreated(verified.into()))
                }
            },
        }
    }
}

/// A creation is waiting for the service, additional approvals, or authentication.
#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct CreationPending {
    #[serde(flatten)]
    recovery: Recovery,
    status: String,
}

impl Display for CreationPending {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(activity_id) = &self.recovery.activity_id {
            writeln!(
                f,
                "Activity ID (untrusted lookup hint): {}",
                TerminalText(activity_id).single_line()
            )?;
        }
        writeln!(
            f,
            "Creation is pending ({}). No keys have been accepted.",
            self.status
        )?;
        self.recovery.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ColorChoice, ErrorMessage, MessageFormat, Shell, TestShell};
    use CreationKind::{Operator, QuorumKey};
    use serde_json::json;
    use std::{io::Read, os::unix::fs::PermissionsExt};
    use turnkey_client::{TurnkeyClient, TurnkeyP256ApiKey};
    use turnkey_proofs::tvc::TvcProvenanceError;
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{method, path},
    };

    const ID: &str = "11111111-1111-4111-8111-111111111111";
    const ORGANIZATION_ID: Uuid = uuid::uuid!("aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    const API: &str = "https://api.turnkey.com";

    fn context() -> StdCtx {
        StdCtx::new(
            Shell::standard(MessageFormat::Human, ColorChoice::Never),
            true,
        )
    }

    fn auth(server: &MockServer) -> AuthenticatedClient {
        AuthenticatedClient {
            client: TurnkeyClient::builder()
                .api_key(TurnkeyP256ApiKey::generate())
                .base_url(server.uri())
                .build()
                .unwrap()
                .with_app_proofs(),
            org_id: ORGANIZATION_ID.to_string(),
            api_base_url: API.into(),
        }
    }

    fn creation(kind: CreationKind) -> CreationIntent {
        if kind == QuorumKey {
            let public = qos_p256::P256Pair::generate()
                .unwrap()
                .public_key()
                .to_bytes();
            CreationIntent::QuorumKey {
                intent: CreateTvcQuorumKeyIntent {
                    threshold: 2,
                    operator_encrypt_keys: vec![
                        hex::encode(&public[..65]),
                        hex::encode(&public[65..]),
                    ],
                },
            }
        } else {
            CreationIntent::Operator {
                intent: CreateTvcOperatorIntent {
                    operator_name: "original operator".into(),
                    wallet_name: Some("original wallet".into()),
                    wallet_id: None,
                    path: "m/0".into(),
                },
                org_alias: "original alias".into(),
                make_default: true,
            }
        }
    }

    fn prepared_creation(kind: CreationKind) -> PreparedCreation {
        let organization_id = ORGANIZATION_ID;
        match creation(kind) {
            CreationIntent::Operator {
                intent,
                org_alias,
                make_default,
            } => PreparedCreation::operator(organization_id, intent, org_alias, make_default),
            CreationIntent::QuorumKey { intent } => {
                let recipients = intent
                    .operator_encrypt_keys
                    .iter()
                    .map(|key| key.parse().unwrap())
                    .collect();
                PreparedCreation::quorum(organization_id, intent.threshold, recipients)
            }
        }
        .unwrap()
    }

    fn checkpoint(kind: CreationKind, activity_id: Option<&str>) -> SubmissionCheckpoint {
        serde_json::from_value(json!({
            "version":1, "organization_id":ORGANIZATION_ID,
            "kind": if kind == QuorumKey { "CreateTvcQuorumKey" } else { "CreateTvcOperator" },
            "fingerprints":["ab".repeat(32)], "activity_id":activity_id
        }))
        .unwrap()
    }

    fn saved(kind: CreationKind) -> SavedCreation {
        SavedCreation {
            version: 1,
            api_base_url: API.into(),
            creation: creation(kind),
            checkpoint: checkpoint(kind, Some("activity-a")),
        }
    }

    fn submit_endpoint(kind: CreationKind) -> &'static str {
        match kind {
            Operator => "/public/v1/submit/create_tvc_operator",
            QuorumKey => "/public/v1/submit/create_tvc_quorum_key",
        }
    }

    fn response(kind: CreationKind, id: &str, status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({"activity":{
            "id":id, "organizationId":ORGANIZATION_ID, "status":status, "fingerprint":"untrusted-server-fingerprint",
            "type": if kind == QuorumKey { "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY" } else { "ACTIVITY_TYPE_CREATE_TVC_OPERATOR" },
            "appProofs":[], "result": if kind == QuorumKey {
                json!({"createTvcQuorumKeyResult":{"quorumKeyId":"quorum-id", "quorumPublicKey":"untrusted-key", "shareIds":["share-a","share-b"]}})
            } else {
                json!({"createTvcOperatorResult":{"walletId":"wallet-id", "operatorId":"operator-id", "encryptPublicKey":"untrusted-encrypt", "signPublicKey":"untrusted-sign"}})
            }
        }}))
    }

    #[tokio::test]
    async fn replacing_a_receipt_preserves_readers_of_the_previous_version() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(
            &directory.path().join("pending-creations"),
            ID.parse().unwrap(),
        );
        operation.prepare_directory().await.unwrap();
        let original = saved(QuorumKey);
        operation.save(&original).await.unwrap();

        let mut previous = File::open(&operation.path).unwrap();
        let replacement = SavedCreation {
            checkpoint: checkpoint(QuorumKey, Some("activity-b")),
            ..saved(QuorumKey)
        };
        operation.save(&replacement).await.unwrap();
        let mut contents = Vec::new();
        previous.read_to_end(&mut contents).unwrap();
        assert_eq!(contents, serde_json::to_vec_pretty(&original).unwrap());
        assert_eq!(
            operation.load().await.unwrap().checkpoint.activity_id(),
            Some("activity-b")
        );
        assert_eq!(
            fs::metadata(operation.path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700,
        );
        assert_eq!(
            fs::metadata(&operation.path).unwrap().permissions().mode() & 0o777,
            0o600,
        );
    }

    async fn assert_version_one_disk_shape(
        original: SavedCreation,
        creation: serde_json::Value,
        checkpoint_kind: &str,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&original).await.unwrap();
        let written: serde_json::Value =
            serde_json::from_slice(&fs::read(&operation.path).unwrap()).unwrap();
        assert_eq!(
            written,
            json!({
                "version": 1,
                "api_base_url": API,
                "creation": creation,
                "checkpoint": {
                    "version": 1,
                    "organization_id": ORGANIZATION_ID,
                    "kind": checkpoint_kind,
                    "fingerprints": ["ab".repeat(32)],
                    "activity_id": "activity-a"
                }
            })
        );
        assert_eq!(
            serde_json::to_value(operation.load().await.unwrap()).unwrap(),
            written
        );
    }

    #[tokio::test]
    async fn operator_record_has_the_version_one_disk_shape() {
        assert_version_one_disk_shape(
            saved(Operator),
            json!({
                "kind": "operator",
                "org_alias": "original alias",
                "make_default": true,
                "intent": {
                    "operatorName": "original operator",
                    "walletName": "original wallet",
                    "walletId": null,
                    "path": "m/0"
                }
            }),
            "CreateTvcOperator",
        )
        .await;
    }

    #[tokio::test]
    async fn quorum_record_has_the_version_one_disk_shape() {
        let original = saved(QuorumKey);
        let CreationIntent::QuorumKey { intent } = &original.creation else {
            panic!("expected quorum intent");
        };
        let creation = json!({
            "kind": "quorum",
            "intent": {"threshold": 2, "operatorEncryptKeys": intent.operator_encrypt_keys}
        });
        assert_version_one_disk_shape(original, creation, "CreateTvcQuorumKey").await;
    }

    async fn assert_record_rejected(record: serde_json::Value, expected: &str) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&record).await.unwrap();
        let error = operation.load().await.unwrap_err();
        assert!(
            error
                .chain()
                .any(|cause| cause.to_string().contains(expected)),
            "{error:#}"
        );
    }

    #[tokio::test]
    async fn record_without_version_is_rejected() {
        assert_record_rejected(json!({}), "missing field `version`").await;
    }

    #[tokio::test]
    async fn record_with_unsupported_version_is_rejected() {
        let record = SavedCreation {
            version: 2,
            ..saved(Operator)
        };
        assert_record_rejected(
            serde_json::to_value(record).unwrap(),
            "unsupported saved creation version 2",
        )
        .await;
    }

    #[tokio::test]
    async fn record_without_checkpoint_fingerprints_is_rejected() {
        let mut record = serde_json::to_value(saved(Operator)).unwrap();
        record["checkpoint"]["fingerprints"] = json!([]);
        assert_record_rejected(record, "invalid submission checkpoint").await;
    }

    #[tokio::test]
    async fn group_writable_records_are_rejected_before_loading_origin_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(Operator)).await.unwrap();
        fs::set_permissions(&operation.path, fs::Permissions::from_mode(0o664)).unwrap();
        assert!(operation.load().await.is_err());
        fs::set_permissions(&operation.path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(operation.load().await.is_ok());
    }

    #[tokio::test]
    async fn symlinked_records_are_rejected_before_loading_origin_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(Operator)).await.unwrap();
        let target = directory.path().join("target.json");
        fs::rename(&operation.path, &target).unwrap();
        std::os::unix::fs::symlink(&target, &operation.path).unwrap();
        assert!(operation.load().await.is_err());
    }

    async fn assert_save_before_post_and_restart_without_reposting(
        kind: CreationKind,
        command: &str,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        let record_path = operation.path.clone();
        Mock::given(method("POST"))
            .and(path(submit_endpoint(kind)))
            .respond_with(move |_: &Request| {
                let durable: SavedCreation =
                    serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
                assert_eq!(durable.checkpoint.sent_fingerprints().count(), 1);
                response(kind, "activity-a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")
            })
            .expect(1)
            .mount(&server)
            .await;
        let outcome = operation
            .submit(
                &mut context(),
                Config::default(),
                CreationClient::try_from(auth(&server)).unwrap(),
                prepared_creation(kind),
                100,
                false,
            )
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            json!({
                "reason":"creation_pending", "operationId":ID, "activityId":"activity-a", "status":"consensus_needed"
            })
        );
        assert!(
            outcome
                .to_string()
                .contains(&format!("{command} {ID} --activity-id activity-a"))
        );
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let before = fs::read(&operation.path).unwrap();
        Mock::given(method("POST"))
            .and(path("/public/v1/query/get_activity"))
            .respond_with(response(kind, "activity-b", "ACTIVITY_STATUS_PENDING"))
            .expect(1)
            .mount(&server)
            .await;
        let loaded = operation.load().await.unwrap();
        let outcome = loaded
            .verify_result(
                &mut context(),
                Config::default(),
                &operation,
                auth(&server),
                "activity-b".into(),
                false,
            )
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&outcome).unwrap()["status"],
            "processing"
        );
        assert!(
            outcome
                .to_string()
                .contains(&format!("{command} {ID} --activity-id activity-b"))
        );
        assert_eq!(fs::read(&operation.path).unwrap(), before);
        server.verify().await;
    }

    #[tokio::test]
    async fn operator_creation_saves_before_post_and_restarts_without_reposting() {
        assert_save_before_post_and_restart_without_reposting(
            Operator,
            "tvc operator create verify-result",
        )
        .await;
    }

    #[tokio::test]
    async fn quorum_creation_saves_before_post_and_restarts_without_reposting() {
        assert_save_before_post_and_restart_without_reposting(
            QuorumKey,
            "tvc keys create-quorum-key verify-result",
        )
        .await;
    }

    #[tokio::test]
    async fn storage_failure_keeps_io_cause_and_prevents_the_creation_post() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        std::fs::create_dir(&operation.path).unwrap();
        let server = MockServer::start().await;
        let error = operation
            .submit(
                &mut context(),
                Config::default(),
                CreationClient::try_from(auth(&server)).unwrap(),
                prepared_creation(Operator),
                100,
                false,
            )
            .await
            .err()
            .unwrap();
        let failure = error.downcast_ref::<CreationFailure>().unwrap();
        assert!(matches!(
            failure.source,
            ReceiptCreationError::CheckpointSave {
                state: SubmissionState::NeverSubmitted,
                checkpoint: None,
                ..
            }
        ));
        assert!(error.chain().any(|cause| cause.is::<io::Error>()));
        assert_eq!(
            crate::errors::classify(&error).code,
            crate::errors::ErrorCode::CommandError
        );
        assert!(
            error
                .to_string()
                .contains("No remote creation request was sent")
        );
        assert!(!error.to_string().contains("Verify with"));
        assert!(server.received_requests().await.unwrap().is_empty());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn retry_save_failure_reports_the_original_activity_and_io_error() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let record_path = operation.path.clone();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(submit_endpoint(QuorumKey)))
            .respond_with(move |_: &Request| {
                // Make the retry write fail after the first request reached the service.
                fs::rename(&record_path, record_path.with_extension("first")).unwrap();
                fs::create_dir(&record_path).unwrap();
                response(QuorumKey, "activity-a", "ACTIVITY_STATUS_PENDING")
            })
            .expect(1)
            .mount(&server)
            .await;

        let error = operation
            .submit(
                &mut context(),
                Config::default(),
                CreationClient::try_from(auth(&server)).unwrap(),
                prepared_creation(QuorumKey),
                100,
                true,
            )
            .await
            .err()
            .unwrap();
        let failure = error.downcast_ref::<CreationFailure>().unwrap();
        let ReceiptCreationError::CheckpointSave {
            state: SubmissionState::MayHaveSubmitted,
            checkpoint: Some(checkpoint),
            ..
        } = &failure.source
        else {
            panic!("expected the previous submission checkpoint")
        };
        assert_eq!(checkpoint.activity_id(), Some("activity-a"));
        assert!(error.chain().any(|cause| cause.is::<io::Error>()));
        assert!(error.to_string().contains(&format!(
            "tvc keys create-quorum-key verify-result {ID} --activity-id activity-a"
        )));
        server.verify().await;
    }

    #[tokio::test]
    async fn wrong_organization_is_rejected_before_network() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        let wrong_org = AuthenticatedClient {
            org_id: Uuid::from_u128(2).to_string(),
            ..auth(&server)
        };
        assert!(
            saved(Operator)
                .verify_result(
                    &mut context(),
                    Config::default(),
                    &operation,
                    wrong_org,
                    "activity-a".into(),
                    false,
                )
                .await
                .is_err()
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn wrong_environment_is_rejected_before_network() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        let wrong_environment = AuthenticatedClient {
            api_base_url: "https://api.preprod.turnkey.engineering".into(),
            ..auth(&server)
        };
        assert!(
            saved(Operator)
                .verify_result(
                    &mut context(),
                    Config::default(),
                    &operation,
                    wrong_environment,
                    "activity-a".into(),
                    false
                )
                .await
                .is_err()
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    async fn assert_invalid_saved_intent(
        kind: CreationKind,
        field: &str,
        value: serde_json::Value,
        expected: &str,
    ) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        let mut persisted = serde_json::to_value(saved(kind)).unwrap();
        persisted["creation"]["intent"][field] = value;
        operation.save(&persisted).await.unwrap();
        let error = operation
            .load()
            .await
            .unwrap()
            .verify_result(
                &mut context(),
                Config::default(),
                &operation,
                auth(&server),
                "activity-a".into(),
                false,
            )
            .await
            .err()
            .unwrap();
        assert!(matches!(
            error.downcast_ref::<TvcProvenanceError>(),
            Some(TvcProvenanceError::InvalidExpectation(reason)) if *reason == expected
        ));
        assert_eq!(
            crate::errors::classify(&error).code,
            crate::errors::ErrorCode::InvalidInput
        );
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&std::fs::read(&operation.path).unwrap())
                .unwrap(),
            persisted
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn saved_operator_with_empty_path_fails_preflight() {
        assert_invalid_saved_intent(
            Operator,
            "path",
            json!(""),
            "organization, wallet and path must be nonempty",
        )
        .await;
    }

    #[tokio::test]
    async fn saved_quorum_with_malformed_recipient_fails_preflight() {
        assert_invalid_saved_intent(
            QuorumKey,
            "operatorEncryptKeys",
            json!(["malformed", "keys"]),
            "invalid recipient public key",
        )
        .await;
    }

    #[tokio::test]
    async fn saved_quorum_with_too_many_recipients_fails_preflight_before_key_parsing() {
        assert_invalid_saved_intent(
            QuorumKey,
            "operatorEncryptKeys",
            json!(vec!["malformed"; 256]),
            "require 2 <= threshold <= recipients <= 255",
        )
        .await;
    }

    async fn mock_completed_without_proofs(
        server: &MockServer,
        kind: CreationKind,
        endpoint: &str,
    ) {
        Mock::given(method("POST"))
            .and(path(endpoint))
            .respond_with(response(kind, "activity-a", "ACTIVITY_STATUS_COMPLETED"))
            .expect(1)
            .mount(server)
            .await;
    }

    async fn assert_missing_proof_error(
        error: anyhow::Error,
        operation: &Operation,
        expected_activity_id: Option<&str>,
    ) {
        let rejection = error
            .chain()
            .find_map(|cause| cause.downcast_ref::<turnkey_proofs::tvc::Rejected>())
            .expect("typed rejection remains in the error chain");
        assert_eq!(
            rejection.stage().check,
            turnkey_proofs::tvc::ProofCheck::ClaimSetAndIntent
        );
        assert_eq!(rejection.stage().envelope_index, None);
        assert!(error.chain().any(|cause| matches!(
            cause.downcast_ref::<TvcProvenanceError>(),
            Some(TvcProvenanceError::ProofUnavailable)
        )));
        assert!(
            ErrorMessage::from_error(&error)
                .to_string()
                .contains("claim set and caller intent")
        );
        assert_eq!(
            crate::errors::classify(&error).code,
            crate::errors::ErrorCode::ProofVerificationFailed
        );
        assert_eq!(
            operation.load().await.unwrap().checkpoint.activity_id(),
            expected_activity_id
        );
        assert!(error.to_string().contains("--activity-id activity-a"));
    }

    async fn assert_create_rejects_missing_proofs(kind: CreationKind) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        mock_completed_without_proofs(&server, kind, submit_endpoint(kind)).await;
        let error = operation
            .submit(
                &mut context(),
                Config::default(),
                CreationClient::try_from(auth(&server)).unwrap(),
                prepared_creation(kind),
                100,
                false,
            )
            .await
            .err()
            .unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        assert_missing_proof_error(error, &operation, None).await;
        server.verify().await;
    }

    async fn assert_verify_result_rejects_missing_proofs(kind: CreationKind) {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        mock_completed_without_proofs(&server, kind, "/public/v1/query/get_activity").await;
        operation.save(&saved(kind)).await.unwrap();
        let error = operation
            .load()
            .await
            .unwrap()
            .verify_result(
                &mut context(),
                Config::default(),
                &operation,
                auth(&server),
                "activity-a".into(),
                false,
            )
            .await
            .err()
            .unwrap();
        assert_missing_proof_error(error, &operation, Some("activity-a")).await;
        server.verify().await;
    }

    #[tokio::test]
    async fn operator_create_rejects_missing_proofs_and_retains_receipt() {
        assert_create_rejects_missing_proofs(Operator).await;
    }

    #[tokio::test]
    async fn quorum_create_rejects_missing_proofs_and_retains_receipt() {
        assert_create_rejects_missing_proofs(QuorumKey).await;
    }

    #[tokio::test]
    async fn operator_verify_result_rejects_missing_proofs_and_retains_receipt() {
        assert_verify_result_rejects_missing_proofs(Operator).await;
    }

    #[tokio::test]
    async fn quorum_verify_result_rejects_missing_proofs_and_retains_receipt() {
        assert_verify_result_rejects_missing_proofs(QuorumKey).await;
    }

    #[tokio::test]
    async fn lookup_failure_reports_one_cause_and_keeps_the_explicit_candidate() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(Operator)).await.unwrap();
        let before = fs::read(&operation.path).unwrap();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/public/v1/query/get_activity"))
            .respond_with(ResponseTemplate::new(503).set_body_string("private coordinator detail"))
            .expect(1)
            .mount(&server)
            .await;

        let error = operation
            .load()
            .await
            .unwrap()
            .verify_result(
                &mut context(),
                Config::default(),
                &operation,
                auth(&server),
                "activity-b".into(),
                false,
            )
            .await
            .err()
            .unwrap();
        let output = serde_json::to_value(ErrorMessage::from_error(&error)).unwrap();
        assert_eq!(output["httpStatus"], 503);
        assert_eq!(output["code"], "api_error");
        let message = output["message"].as_str().unwrap();
        assert_eq!(message.matches("HTTP status 503").count(), 1);
        assert!(message.contains(&format!(
            "tvc operator create verify-result {ID} --activity-id activity-b"
        )));
        assert!(!message.contains("private coordinator detail"));
        assert_eq!(fs::read(&operation.path).unwrap(), before);
        server.verify().await;
    }

    #[test]
    fn verify_result_command_shows_placeholder_for_missing_activity_id() {
        let recovery = Recovery {
            kind: CreationKind::Operator,
            operation_id: ID.parse().unwrap(),
            activity_id: None,
            resumed: false,
        };
        assert!(recovery.to_string().contains("--activity-id <candidate>"));
    }

    #[test]
    fn verify_result_command_quotes_service_supplied_activity_id() {
        let recovery = Recovery {
            kind: CreationKind::Operator,
            operation_id: ID.parse().unwrap(),
            activity_id: Some("label' $(example)".into()),
            resumed: false,
        };
        assert!(
            recovery
                .to_string()
                .ends_with(r#"--activity-id='label'\'' $(example)'"#)
        );
    }

    async fn assert_verbose_lookup_preserves_rejection(kind: CreationKind, format: MessageFormat) {
        let original = serde_json::to_value(saved(kind)).unwrap();
        let ephemeral_key = hex::encode(
            qos_p256::P256Pair::generate()
                .unwrap()
                .public_key()
                .to_bytes(),
        );
        let observe = async |verbose| {
            let directory = tempfile::tempdir().unwrap();
            let operation = Operation::at(directory.path(), ID.parse().unwrap());
            operation.save(&original).await.unwrap();
            let server = MockServer::start().await;
            let result = if kind == QuorumKey {
                json!({"createTvcQuorumKeyResult": {"quorumKeyId":ID,"quorumPublicKey":"returned-key","shareIds":[]}})
            } else {
                json!({"createTvcOperatorResult": {"operatorId":ID,"walletId":ID,"encryptPublicKey":"returned-encryption","signPublicKey":"returned-signing"}})
            };
            Mock::given(method("POST")).and(path("/public/v1/query/get_activity"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({"activity": {
                        "id":"activity-a", "organizationId":ORGANIZATION_ID, "status":"ACTIVITY_STATUS_COMPLETED", "fingerprint":"untrusted-lookup-fingerprint",
                        "type": if kind == QuorumKey {"ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY"} else {"ACTIVITY_TYPE_CREATE_TVC_OPERATOR"},
                        "result":result,
                        "appProofs":[{"scheme":"SIGNATURE_SCHEME_EPHEMERAL_KEY_P256","publicKey":ephemeral_key,
                            "signature":"00".repeat(64), "proofPayload":json!({"type":if kind == QuorumKey {"APP_PROOF_TYPE_TVC_QUORUM_KEY_CREATE"} else {"APP_PROOF_TYPE_ADDRESS_DERIVATION"}}).to_string()}]
                    }}))).expect(1).mount(&server).await;
            Mock::given(method("POST"))
                .and(path("/public/v1/query/get_boot_proof"))
                .respond_with(ResponseTemplate::new(503))
                .expect(1)
                .mount(&server)
                .await;
            let mut ctx = StdCtx::new(Shell::standard(format, ColorChoice::Never), true);
            let error = operation
                .load()
                .await
                .unwrap()
                .verify_result(
                    &mut ctx,
                    Config::default(),
                    &operation,
                    auth(&server),
                    "activity-a".into(),
                    verbose,
                )
                .await
                .err()
                .unwrap();
            let rejected = error
                .chain()
                .find_map(|cause| cause.downcast_ref::<turnkey_proofs::tvc::Rejected>())
                .unwrap_or_else(|| panic!("expected proof rejection, got {error:#}"));
            assert_eq!(
                rejected.stage(),
                turnkey_proofs::tvc::AcceptanceStage {
                    check: turnkey_proofs::tvc::ProofCheck::BootProofLookup,
                    envelope_index: Some(0),
                }
            );
            let requests: Vec<_> = server
                .received_requests()
                .await
                .unwrap()
                .into_iter()
                .map(|request| {
                    (
                        request.url.path().to_owned(),
                        serde_json::from_slice::<serde_json::Value>(&request.body).unwrap(),
                    )
                })
                .collect();
            assert_eq!(
                requests
                    .iter()
                    .map(|(path, _)| path.as_str())
                    .collect::<Vec<_>>(),
                [
                    "/public/v1/query/get_activity",
                    "/public/v1/query/get_boot_proof"
                ]
            );
            let mut shell = TestShell::with_json_formatter();
            shell.emit(&ErrorMessage::from_error(&error)).unwrap();
            let saved_bytes = std::fs::read(&operation.path).unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&saved_bytes).unwrap(),
                original
            );
            assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
            server.verify().await;
            (
                requests,
                saved_bytes,
                serde_json::to_value(ErrorMessage::from_error(&error)).unwrap(),
                shell.into_stdout(),
            )
        };
        assert_eq!(observe(false).await, observe(true).await);
    }

    #[tokio::test]
    async fn operator_human_verbose_lookup_preserves_requests_receipt_and_rejection() {
        assert_verbose_lookup_preserves_rejection(Operator, MessageFormat::Human).await;
    }

    #[tokio::test]
    async fn operator_json_verbose_lookup_preserves_requests_receipt_and_rejection() {
        assert_verbose_lookup_preserves_rejection(Operator, MessageFormat::Json).await;
    }

    #[tokio::test]
    async fn quorum_human_verbose_lookup_preserves_requests_receipt_and_rejection() {
        assert_verbose_lookup_preserves_rejection(QuorumKey, MessageFormat::Human).await;
    }

    #[tokio::test]
    async fn quorum_json_verbose_lookup_preserves_requests_receipt_and_rejection() {
        assert_verbose_lookup_preserves_rejection(QuorumKey, MessageFormat::Json).await;
    }
}
