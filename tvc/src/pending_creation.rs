//! Save the original signed request's fingerprint and intent before each hosted
//! creation attempt. If the activity completes later, resume can verify the
//! result against that saved evidence before accepting any keys.

use crate::{
    client::{AuthenticatedClient, build_client},
    commands::operator::create::save_hosted_operator,
    config::turnkey::{Config, config_dir, config_file_path},
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
    future::ready,
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tempfile::NamedTempFile;
use turnkey_client::{
    ActivityReceipt, CheckpointSaveError, SubmissionCheckpoint, SubmissionFailureCause,
    SubmissionOutcome, SubmissionPendingReason, SubmissionState, SubmittedActivityResult,
    TurnkeyClientError, TurnkeyP256ApiKey,
    generated::{CreateTvcOperatorIntent, CreateTvcQuorumKeyIntent},
};
use turnkey_proofs::tvc::{OperatorExpectation, P256PublicKey, QuorumExpectation, TvcProvenance};
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
    fn resume_command(self) -> &'static str {
        match self {
            Self::Operator => "tvc operator create resume",
            Self::QuorumKey => "tvc keys create-quorum-key resume",
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

/// The local evidence and intent needed to verify a completed creation on resume.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SavedCreation {
    version: u32,
    api_base_url: String,
    creation: CreationIntent,
    checkpoint: SubmissionCheckpoint,
    // Accept receipts from earlier builds of the proof-enforcing CLI.
    #[serde(rename = "timestamp_ms", default, skip_serializing)]
    _legacy_timestamp_ms: Option<String>,
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
pub(crate) struct Operation {
    id: Uuid,
    path: PathBuf,
}

impl Operation {
    pub(crate) fn open(id: Uuid) -> Result<Self> {
        Ok(Self::at(&config_dir()?.join("pending-creations"), id))
    }

    fn at(directory: &Path, id: Uuid) -> Self {
        Self {
            id,
            path: directory.join(format!("{id}.json")),
        }
    }

    /// Create the receipt directory using TVC's regular config parent.
    fn prepare_directory(&self) -> Result<()> {
        let directory = self.path.parent().expect("operation path has a parent");
        if let Some(parent) = directory.parent() {
            fs::create_dir_all(parent)?;
        }
        match fs::DirBuilder::new().mode(0o700).create(directory) {
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
    }

    pub(crate) async fn load(&self) -> Result<SavedCreation> {
        let directory = self.path.parent().expect("operation path has a parent");
        let parent = fs::symlink_metadata(directory).with_context(|| {
            format!(
                "failed to inspect saved creation directory {}",
                directory.display()
            )
        })?;
        let file = fs::symlink_metadata(&self.path)
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

    fn save(&self, saved: &impl Serialize) -> Result<()> {
        self.write_record(saved).with_context(|| {
            format!(
                "failed to save creation {} at {}",
                self.id,
                self.path.display()
            )
        })
    }

    // A failed update must leave the previous fingerprints available for recovery.
    fn write_record(&self, saved: &impl Serialize) -> Result<()> {
        let directory = self.path.parent().expect("operation path has a parent");
        let contents = serde_json::to_vec_pretty(saved)?;
        let mut file = NamedTempFile::new_in(directory)?;
        file.write_all(&contents)?;
        file.as_file().sync_all()?;
        file.persist(&self.path).map_err(|error| error.error)?;
        File::open(directory)?.sync_all()?;
        Ok(())
    }
}

/// An authenticated TVC API client with the matching Signer policy selected.
pub(crate) struct CreationClient {
    client: TvcProvenance<TurnkeyP256ApiKey>,
    pub(crate) org_id: Uuid,
    pub(crate) api_base_url: String,
}

impl CreationClient {
    pub(crate) fn ensure_configured_org(&self, configured_org_id: &str) -> Result<()> {
        let configured_id = Uuid::parse_str(configured_org_id)
            .context("configured organization ID must be a UUID")?;
        ensure_authenticated_org(&self.org_id.to_string(), &configured_id.to_string())
    }
}

pub(crate) async fn build_creation_client(config: &Config) -> Result<CreationClient> {
    with_provenance(build_client(config).await?)
}

fn with_provenance(auth: AuthenticatedClient) -> Result<CreationClient> {
    let AuthenticatedClient {
        client,
        org_id,
        api_base_url,
    } = auth;
    let org_id =
        Uuid::parse_str(&org_id).context("authenticated organization ID must be a UUID")?;
    let client = TvcProvenance::try_from((client.with_app_proofs(), api_base_url.as_str()))?;
    Ok(CreationClient {
        client,
        org_id,
        api_base_url,
    })
}

pub(crate) async fn submit(
    ctx: &mut StdCtx,
    auth: CreationClient,
    creation: PreparedCreation,
    timestamp_ms: u128,
) -> Result<Outcome> {
    let mut bytes = [0; 16];
    OsRng
        .try_fill_bytes(&mut bytes)
        .context("failed to generate local creation UUID")?;
    let operation = Operation::open(uuid::Builder::from_random_bytes(bytes).into_uuid())?;
    operation.submit(ctx, auth, creation, timestamp_ms).await
}

impl Operation {
    async fn submit(
        &self,
        ctx: &mut StdCtx,
        auth: CreationClient,
        creation: PreparedCreation,
        timestamp_ms: u128,
    ) -> Result<Outcome> {
        self.prepare_directory()
            .context("failed to prepare the saved creation directory")?;
        let CreationClient {
            client: verifier,
            org_id,
            api_base_url,
        } = auth;
        let intent = CreationIntent::from(&creation);
        let kind = intent.kind();
        let storage_failure = Arc::new(Mutex::new(None));
        let save = {
            let storage_failure = Arc::clone(&storage_failure);
            let api_base_url = api_base_url.clone();
            let operation = Self {
                id: self.id,
                path: self.path.clone(),
            };
            move |checkpoint: &SubmissionCheckpoint| {
                let saved = SavedCreation {
                    version: 1,
                    api_base_url: api_base_url.clone(),
                    creation: intent.clone(),
                    checkpoint: checkpoint.clone(),
                    _legacy_timestamp_ms: None,
                };
                ready(operation.save(&saved).map_err(|error| {
                    *storage_failure
                        .lock()
                        .expect("checkpoint save failed without panicking") = Some(error);
                    CheckpointSaveError
                }))
            }
        };

        let result = match &creation.0 {
            PreparedIntent::Operator { intent, .. } => verifier
                .client()
                .create_tvc_operator_with_receipt(org_id, timestamp_ms, intent.clone(), save)
                .await
                .map(receipt_outcome),
            PreparedIntent::Quorum { intent, .. } => verifier
                .client()
                .create_tvc_quorum_key_with_receipt(org_id, timestamp_ms, intent.clone(), save)
                .await
                .map(receipt_outcome),
        };

        let storage_failure = storage_failure
            .lock()
            .expect("checkpoint state poisoned")
            .take();
        let recovery = Recovery {
            kind,
            operation_id: self.id,
            activity_id: None,
        };
        creation
            .finish(
                ctx,
                verifier,
                &api_base_url,
                result,
                recovery,
                storage_failure,
            )
            .await
    }
}

type ReceiptResult = std::result::Result<SubmissionOutcome<ActivityReceipt>, TurnkeyClientError>;

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
    pub(crate) fn check_kind(&self, kind: CreationKind) -> Result<()> {
        ensure!(
            self.creation.kind() == kind,
            "this saved operation belongs to `{}`",
            self.creation.kind().resume_command(),
        );
        Ok(())
    }

    pub(crate) async fn resume(
        self,
        ctx: &mut StdCtx,
        operation: &Operation,
        auth: AuthenticatedClient,
        activity_id: String,
    ) -> Result<Outcome> {
        let Self {
            version: _,
            api_base_url,
            creation,
            checkpoint,
            _legacy_timestamp_ms: _,
        } = self;
        ensure!(
            auth.api_base_url == api_base_url,
            "saved creation belongs to a different API environment"
        );
        let auth = with_provenance(auth)?;
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
                let expected = QuorumExpectation::parse(
                    organization_id.to_string(),
                    intent.threshold,
                    &intent.operator_encrypt_keys,
                )?;
                PreparedCreation(PreparedIntent::Quorum { intent, expected })
            }
        };
        let verifier = auth.client;
        let result = verifier
            .client()
            .resume_submission(checkpoint, Some(activity_id.clone()))
            .await;
        let recovery = Recovery {
            kind,
            operation_id: operation.id,
            activity_id: Some(activity_id),
        };
        creation
            .finish(ctx, verifier, &api_base_url, result, recovery, None)
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
}

impl Display for Recovery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let candidate = self
            .activity_id
            .as_deref()
            .filter(|id| TerminalText(*id).single_line() == *id);
        let argument = match candidate {
            Some(id)
                if id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                    && !id.starts_with('-') =>
            {
                format!("--activity-id {id}")
            }
            Some(id) => format!("--activity-id='{}'", id.replace('\'', "'\\''")),
            None => "--activity-id <candidate>".to_owned(),
        };
        write!(
            f,
            "Resume with: {} {} {argument}",
            self.kind.resume_command(),
            self.operation_id
        )?;
        if candidate.is_none() {
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
    source: TurnkeyClientError,
    storage_failure: Option<anyhow::Error>,
    recovery: Recovery,
}

impl Display for CreationFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let TurnkeyClientError::SubmissionFailed(failure) = &self.source else {
            return write!(f, "Creation failed: {}\n{}", self.source, self.recovery);
        };
        write!(f, "{}.", failure.cause())?;
        if let Some(storage) = &self.storage_failure {
            write!(f, "\n{storage:#}")?;
        }
        match failure.submission_state() {
            SubmissionState::NeverSubmitted => write!(f, "\nNo remote creation request was sent."),
            SubmissionState::MayHaveSubmitted => {
                write!(
                    f,
                    "\nNo keys were accepted or saved. Operation ID: {}",
                    self.recovery.operation_id
                )?;
                if *failure.cause() == SubmissionFailureCause::ActivityNotCompleted {
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
        Some(&self.source)
    }
}

impl PreparedCreation {
    async fn finish(
        self,
        ctx: &mut StdCtx,
        verifier: TvcProvenance<TurnkeyP256ApiKey>,
        api_base_url: &str,
        result: ReceiptResult,
        recovery: Recovery,
        storage_failure: Option<anyhow::Error>,
    ) -> Result<Outcome> {
        let activity_id = match &result {
            Ok(SubmissionOutcome::Pending { checkpoint, .. }) => checkpoint.activity_id(),
            Ok(SubmissionOutcome::Completed(receipt)) => receipt.checkpoint().activity_id(),
            Err(TurnkeyClientError::SubmissionFailed(failure)) => failure.activity_id(),
            Err(_) => None,
        };
        let recovery = Recovery {
            activity_id: recovery
                .activity_id
                .or_else(|| activity_id.map(str::to_owned)),
            ..recovery
        };
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(source) => {
                return Err(CreationFailure {
                    source,
                    storage_failure,
                    recovery,
                }
                .into());
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
                    let acceptance = verifier.accept_operator(expected, receipt).await;
                    crate::proofs::present_status(ctx.shell(), &acceptance);
                    let accepted = acceptance
                        .context("proof acceptance failed; no keys were saved")
                        .with_context(|| recovery.to_string())?;
                    let hosted = accepted
                        .into_value()
                        .try_into()
                        .with_context(|| recovery.to_string())?;
                    save_hosted_operator(
                        &config_file_path()?,
                        &org_alias,
                        organization_id,
                        api_base_url,
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
                    let acceptance = verifier.accept_quorum(expected, receipt).await;
                    crate::proofs::present_status(ctx.shell(), &acceptance);
                    let accepted = acceptance
                        .context("proof acceptance failed; no keys were accepted")
                        .with_context(|| recovery.to_string())?;
                    Ok(Outcome::QuorumKeyCreated(accepted.into_value().into()))
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
    use crate::output::{ColorChoice, ErrorMessage, MessageFormat, Shell};
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
            Shell::standard(MessageFormat::Json, ColorChoice::Never),
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

    fn creation(quorum: bool) -> CreationIntent {
        if quorum {
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

    fn prepared_creation(quorum: bool) -> PreparedCreation {
        let organization_id = ORGANIZATION_ID;
        match creation(quorum) {
            CreationIntent::Operator {
                intent,
                org_alias,
                make_default,
            } => PreparedCreation::operator(organization_id, intent, org_alias, make_default),
            CreationIntent::QuorumKey { intent } => {
                let recipients = intent
                    .operator_encrypt_keys
                    .iter()
                    .map(|key| P256PublicKey::try_from(key.as_str()).unwrap())
                    .collect();
                PreparedCreation::quorum(organization_id, intent.threshold, recipients)
            }
        }
        .unwrap()
    }

    fn checkpoint(quorum: bool, activity_id: Option<&str>) -> SubmissionCheckpoint {
        serde_json::from_value(json!({
            "version":1, "organization_id":ORGANIZATION_ID,
            "kind": if quorum { "CreateTvcQuorumKey" } else { "CreateTvcOperator" },
            "fingerprints":["ab".repeat(32)], "activity_id":activity_id
        }))
        .unwrap()
    }

    fn saved(quorum: bool) -> SavedCreation {
        SavedCreation {
            version: 1,
            api_base_url: API.into(),
            _legacy_timestamp_ms: None,
            creation: creation(quorum),
            checkpoint: checkpoint(quorum, Some("activity-a")),
        }
    }

    fn response(quorum: bool, id: &str, status: &str) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(json!({"activity":{
            "id":id, "organizationId":ORGANIZATION_ID, "status":status, "fingerprint":"untrusted-server-fingerprint",
            "type": if quorum { "ACTIVITY_TYPE_CREATE_TVC_QUORUM_KEY" } else { "ACTIVITY_TYPE_CREATE_TVC_OPERATOR" },
            "appProofs":[], "result": if quorum {
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
        operation.prepare_directory().unwrap();
        let original = saved(true);
        operation.save(&original).unwrap();

        let mut previous = File::open(&operation.path).unwrap();
        let replacement = SavedCreation {
            checkpoint: checkpoint(true, Some("activity-b")),
            ..saved(true)
        };
        operation.save(&replacement).unwrap();
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

    #[tokio::test]
    async fn saved_record_has_the_version_one_disk_shape() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        for quorum in [false, true] {
            let original = saved(quorum);
            operation.save(&original).unwrap();
            let written: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&operation.path).unwrap()).unwrap();
            let creation = if quorum {
                let CreationIntent::QuorumKey { intent } = operation.load().await.unwrap().creation
                else {
                    panic!("expected quorum intent");
                };
                json!({
                    "kind": "quorum",
                    "intent": {
                        "threshold": 2,
                        "operatorEncryptKeys": intent.operator_encrypt_keys
                    }
                })
            } else {
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
                })
            };
            assert_eq!(
                written,
                json!({
                    "version": 1,
                    "api_base_url": API,
                    "creation": creation,
                    "checkpoint": {
                        "version": 1,
                        "organization_id": ORGANIZATION_ID,
                        "kind": if quorum { "CreateTvcQuorumKey" } else { "CreateTvcOperator" },
                        "fingerprints": ["ab".repeat(32)],
                        "activity_id": "activity-a"
                    }
                })
            );
            assert_eq!(
                serde_json::to_value(operation.load().await.unwrap()).unwrap(),
                written
            );
            let mut legacy = written;
            legacy["timestamp_ms"] = json!("100");
            operation.save(&legacy).unwrap();
            assert!(operation.load().await.is_ok());
        }
    }

    #[tokio::test]
    async fn malformed_and_unsupported_records_are_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(false)).unwrap();
        let bad_version = {
            let mut value = serde_json::to_value(saved(false)).unwrap();
            value["version"] = json!(2);
            value
        };
        let bad_checkpoint = {
            let mut value = serde_json::to_value(saved(false)).unwrap();
            value["checkpoint"]["fingerprints"] = json!([]);
            value
        };
        for (bad, expected) in [
            (json!({}), "missing field `version`"),
            (bad_version, "unsupported saved creation version 2"),
            (bad_checkpoint, "invalid submission checkpoint"),
        ] {
            std::fs::write(&operation.path, serde_json::to_vec(&bad).unwrap()).unwrap();
            let error = operation.load().await.unwrap_err();
            assert!(
                error
                    .chain()
                    .any(|cause| cause.to_string().contains(expected)),
                "{error:#}"
            );
        }
    }

    #[tokio::test]
    async fn writable_or_nonregular_records_are_rejected_before_loading_origin_evidence() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(false)).unwrap();
        fs::set_permissions(&operation.path, fs::Permissions::from_mode(0o664)).unwrap();
        assert!(operation.load().await.is_err());
        fs::set_permissions(&operation.path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(operation.load().await.is_ok());
        let target = directory.path().join("target.json");
        fs::rename(&operation.path, &target).unwrap();
        std::os::unix::fs::symlink(&target, &operation.path).unwrap();
        assert!(operation.load().await.is_err());
    }

    #[test]
    fn submission_http_classification_survives_an_attached_storage_failure() {
        let error: anyhow::Error = CreationFailure {
            source: TurnkeyClientError::UnexpectedHttpStatus(403, "{}".into()),
            storage_failure: Some(io::Error::from(io::ErrorKind::PermissionDenied).into()),
            recovery: Recovery {
                kind: CreationKind::Operator,
                operation_id: ID.parse().unwrap(),
                activity_id: None,
            },
        }
        .into();
        assert_eq!(
            crate::errors::classify(&error),
            crate::errors::Classification {
                code: crate::errors::ErrorCode::Unauthorized,
                http_status: Some(403),
            }
        );
    }

    #[tokio::test]
    async fn both_creations_save_before_post_and_restart_without_reposting() {
        for quorum in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let operation = Operation::at(directory.path(), ID.parse().unwrap());
            let server = MockServer::start().await;
            let record_path = operation.path.clone();
            Mock::given(method("POST"))
                .and(path(if quorum {
                    "/public/v1/submit/create_tvc_quorum_key"
                } else {
                    "/public/v1/submit/create_tvc_operator"
                }))
                .respond_with(move |_: &Request| {
                    let durable: SavedCreation =
                        serde_json::from_slice(&std::fs::read(&record_path).unwrap()).unwrap();
                    assert_eq!(durable.checkpoint.sent_fingerprints().count(), 1);
                    response(quorum, "activity-a", "ACTIVITY_STATUS_CONSENSUS_NEEDED")
                })
                .expect(1)
                .mount(&server)
                .await;
            let outcome = operation
                .submit(
                    &mut context(),
                    with_provenance(auth(&server)).unwrap(),
                    prepared_creation(quorum),
                    100,
                )
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(&outcome).unwrap(),
                json!({
                    "reason":"creation_pending", "operationId":ID, "activityId":"activity-a", "status":"consensus_needed"
                })
            );
            let command = if quorum {
                "tvc keys create-quorum-key resume"
            } else {
                "tvc operator create resume"
            };
            assert!(
                outcome
                    .to_string()
                    .contains(&format!("{command} {ID} --activity-id activity-a"))
            );
            let before = fs::read(&operation.path).unwrap();
            Mock::given(method("POST"))
                .and(path("/public/v1/query/get_activity"))
                .respond_with(response(quorum, "activity-b", "ACTIVITY_STATUS_PENDING"))
                .expect(1)
                .mount(&server)
                .await;
            let operation = Operation::at(directory.path(), ID.parse().unwrap());
            let loaded = operation.load().await.unwrap();
            let outcome = loaded
                .resume(
                    &mut context(),
                    &operation,
                    auth(&server),
                    "activity-b".into(),
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
                with_provenance(auth(&server)).unwrap(),
                prepared_creation(false),
                100,
            )
            .await
            .err()
            .unwrap();
        let failure = error.downcast_ref::<CreationFailure>().unwrap();
        let TurnkeyClientError::SubmissionFailed(submission) = &failure.source else {
            panic!("expected typed submission failure")
        };
        assert_eq!(
            submission.submission_state(),
            SubmissionState::NeverSubmitted
        );
        assert!(
            failure
                .storage_failure
                .as_ref()
                .unwrap()
                .chain()
                .any(|cause| cause.is::<io::Error>())
        );
        assert!(
            error
                .to_string()
                .contains("No remote creation request was sent")
        );
        assert!(!error.to_string().contains("Resume with"));
        assert!(server.received_requests().await.unwrap().is_empty());
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn wrong_organization_and_environment_are_rejected_before_network() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        let wrong_org = AuthenticatedClient {
            org_id: Uuid::from_u128(2).to_string(),
            ..auth(&server)
        };
        assert!(
            saved(false)
                .resume(&mut context(), &operation, wrong_org, "activity-a".into())
                .await
                .is_err()
        );
        let wrong_environment = AuthenticatedClient {
            api_base_url: "https://api.preprod.turnkey.engineering".into(),
            ..auth(&server)
        };
        assert!(
            saved(false)
                .resume(
                    &mut context(),
                    &operation,
                    wrong_environment,
                    "activity-a".into()
                )
                .await
                .is_err()
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn invalid_saved_intent_fails_preflight() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        let server = MockServer::start().await;
        for (quorum, field, value, expected) in [
            (
                false,
                "path",
                json!(""),
                "organization, wallet and path must be nonempty",
            ),
            (
                true,
                "operatorEncryptKeys",
                json!(["malformed", "keys"]),
                "invalid recipient public key",
            ),
            (
                true,
                "operatorEncryptKeys",
                json!(vec!["malformed"; 256]),
                "require 2 <= threshold <= recipients <= 255",
            ),
        ] {
            let mut persisted = serde_json::to_value(saved(quorum)).unwrap();
            persisted["creation"]["intent"][field] = value;
            operation.save(&persisted).unwrap();
            let error = operation
                .load()
                .await
                .unwrap()
                .resume(
                    &mut context(),
                    &operation,
                    auth(&server),
                    "activity-a".into(),
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
                serde_json::from_slice::<serde_json::Value>(
                    &std::fs::read(&operation.path).unwrap()
                )
                .unwrap(),
                persisted
            );
        }
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn initial_and_resumed_completion_reject_missing_proofs_and_retain_the_receipt() {
        for quorum in [false, true] {
            for resumed in [false, true] {
                let directory = tempfile::tempdir().unwrap();
                let operation = Operation::at(directory.path(), ID.parse().unwrap());
                let server = MockServer::start().await;
                let endpoint = if resumed {
                    "/public/v1/query/get_activity"
                } else if quorum {
                    "/public/v1/submit/create_tvc_quorum_key"
                } else {
                    "/public/v1/submit/create_tvc_operator"
                };
                Mock::given(method("POST"))
                    .and(path(endpoint))
                    .respond_with(response(quorum, "activity-a", "ACTIVITY_STATUS_COMPLETED"))
                    .expect(1)
                    .mount(&server)
                    .await;
                let client = auth(&server);
                let result = if resumed {
                    operation.save(&saved(quorum)).unwrap();
                    operation
                        .load()
                        .await
                        .unwrap()
                        .resume(&mut context(), &operation, client, "activity-a".into())
                        .await
                } else {
                    operation
                        .submit(
                            &mut context(),
                            with_provenance(client).unwrap(),
                            prepared_creation(quorum),
                            100,
                        )
                        .await
                };
                let error = result.err().unwrap();
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
                    resumed.then_some("activity-a")
                );
                assert!(error.to_string().contains("--activity-id activity-a"));
                server.verify().await;
            }
        }
    }

    #[tokio::test]
    async fn lookup_failure_reports_one_cause_and_keeps_the_explicit_candidate() {
        let directory = tempfile::tempdir().unwrap();
        let operation = Operation::at(directory.path(), ID.parse().unwrap());
        operation.save(&saved(false)).unwrap();
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
            .resume(
                &mut context(),
                &operation,
                auth(&server),
                "activity-b".into(),
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
            "tvc operator create resume {ID} --activity-id activity-b"
        )));
        assert!(!message.contains("private coordinator detail"));
        assert_eq!(fs::read(&operation.path).unwrap(), before);
        server.verify().await;
    }

    #[test]
    fn resume_commands_quote_service_supplied_ids_and_handle_missing_ids() {
        let recovery = Recovery {
            kind: CreationKind::Operator,
            operation_id: ID.parse().unwrap(),
            activity_id: None,
        };
        assert!(recovery.to_string().contains("--activity-id <candidate>"));
        let recovery = Recovery {
            activity_id: Some("label' $(example)".into()),
            ..recovery
        };
        assert!(
            recovery
                .to_string()
                .ends_with(r#"--activity-id='label'\'' $(example)'"#)
        );
    }
}
