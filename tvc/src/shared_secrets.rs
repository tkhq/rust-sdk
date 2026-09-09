//! Encrypted Secrets transfer with durable, identity-bound export recovery.
use crate::{
    shared_auth::ResolvedAuth,
    shared_operations::{OperationOutput, query, submit_bytes},
};
use anyhow::{Context, Result, bail, ensure};
use clap::Subcommand;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use turnkey_enclave_encrypt::{ExportClient, ImportClient, QuorumPublicKey};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

const SUITE: &str = "TRANSPORT_ENCRYPTION_SUITE_ENCLAVE_ENCRYPT_V1";
const MAX_SECRET_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Subcommand)]
pub enum SecretCommand {
    /// List metadata, one page at a time. Payloads are never returned.
    List {
        #[arg(long, default_value_t=50, value_parser=clap::value_parser!(u32).range(1..=100))]
        limit: u32,
        #[arg(long)]
        cursor: Option<Uuid>,
    },
    /// Encrypt bytes from a file or stdin before submitting an import.
    Import {
        #[arg(long)]
        name: String,
        #[arg(long)]
        input_file: PathBuf,
        /// JSON map of nonsecret, policy-visible string properties.
        #[arg(long)]
        static_properties_file: Option<PathBuf>,
        /// Continue a previously approved initialization without submitting another.
        #[arg(long)]
        init_activity_id: Option<Uuid>,
    },
    /// Export to a NEW protected file. Save recovery state before submitting.
    Export {
        id: Uuid,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        state_file: PathBuf,
        #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u64).range(1..=3600))]
        timeout: u64,
    },
    /// Recover an export by inspecting its activity; never resubmits it.
    Resume {
        #[arg(long)]
        state_file: PathBuf,
        #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u64).range(1..=3600))]
        timeout: u64,
    },
}

pub enum PreparedSecret {
    List {
        limit: u32,
        cursor: Option<Uuid>,
    },
    Import {
        name: String,
        plaintext: Zeroizing<Vec<u8>>,
        properties: std::collections::BTreeMap<String, String>,
        init_activity_id: Option<Uuid>,
    },
    Export {
        id: Uuid,
        output: PathBuf,
        state_file: PathBuf,
        timeout: u64,
    },
    Resume {
        state: ExportState,
        state_file: PathBuf,
        timeout: u64,
    },
}

impl SecretCommand {
    pub fn prepare(self) -> Result<PreparedSecret> {
        Ok(match self {
            Self::List { limit, cursor } => PreparedSecret::List { limit, cursor },
            Self::Import {
                name,
                input_file,
                static_properties_file,
                init_activity_id,
            } => {
                ensure!(
                    !name.trim().is_empty() && name.len() <= 256,
                    "secret name must contain 1–256 bytes"
                );
                let properties = match static_properties_file {
                    Some(path) => {
                        serde_json::from_slice::<std::collections::BTreeMap<String, String>>(
                            &fs::read(path).context("read static properties")?,
                        )
                        .map_err(|_| {
                            anyhow::anyhow!(
                                "static properties must be a JSON object of nonsecret string values"
                            )
                        })?
                    }
                    None => Default::default(),
                };
                let mut plaintext = Zeroizing::new(Vec::new());
                if input_file.as_os_str() == "-" {
                    std::io::stdin()
                        .take(MAX_SECRET_BYTES + 1)
                        .read_to_end(&mut plaintext)
                        .context("read secret from stdin")?;
                } else {
                    File::open(input_file)
                        .context("open secret input")?
                        .take(MAX_SECRET_BYTES + 1)
                        .read_to_end(&mut plaintext)
                        .context("read secret input")?;
                }
                ensure!(
                    plaintext.len() as u64 <= MAX_SECRET_BYTES,
                    "secret exceeds the 1 MiB CLI input limit"
                );
                PreparedSecret::Import {
                    name,
                    plaintext,
                    properties,
                    init_activity_id,
                }
            }
            Self::Export {
                id,
                output,
                state_file,
                timeout,
            } => {
                let output = new_path(output)?;
                let state_file = new_path(state_file)?;
                ensure!(
                    output != state_file,
                    "output and recovery state must be different files"
                );
                PreparedSecret::Export {
                    id,
                    output,
                    state_file,
                    timeout,
                }
            }
            Self::Resume {
                state_file,
                timeout,
            } => {
                let state_file = absolute_path(state_file)?;
                let state = ExportState::read(&state_file)?;
                PreparedSecret::Resume {
                    state,
                    state_file,
                    timeout,
                }
            }
        })
    }
}

fn absolute_path(path: PathBuf) -> Result<PathBuf> {
    let name = path.file_name().context("destination must name a file")?;
    ensure!(
        path.as_os_str() != "-",
        "stdout is not a protected destination"
    );
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(parent
        .canonicalize()
        .context("resolve destination directory")?
        .join(name))
}
fn new_path(path: PathBuf) -> Result<PathBuf> {
    let path = absolute_path(path)?;
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(path),
        Err(e) => Err(e).context("inspect destination"),
        Ok(_) => bail!("destination already exists; choose a new file"),
    }
}

/// Persisted schemas are private credential material, never command output.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportState {
    version: u8,
    organization_id: Uuid,
    api_base_url: String,
    identity_public_key: String,
    secret_id: Uuid,
    target_public_key: String,
    proposal: String,
    fingerprint: String,
    output: PathBuf,
    key_material: Option<String>,
    activity_id: Option<String>,
    completed: bool,
}
impl Drop for ExportState {
    fn drop(&mut self) {
        if let Some(key) = &mut self.key_material {
            key.zeroize();
        }
    }
}
impl ExportState {
    fn read(path: &Path) -> Result<Self> {
        let encoded = read_private(path, 1024 * 1024)?;
        let state: Self = serde_json::from_slice(&encoded)
            .map_err(|_| anyhow::anyhow!("invalid recovery state schema"))?;
        ensure!(state.version == 1, "unsupported recovery state version");
        ensure!(
            state.output.is_absolute(),
            "recovery output path must be absolute"
        );
        ensure!(
            state.fingerprint == fingerprint(&state.proposal),
            "recovery proposal fingerprint mismatch"
        );
        let proposal: Value = serde_json::from_str(&state.proposal)
            .map_err(|_| anyhow::anyhow!("invalid recovery proposal"))?;
        let expected = envelope(
            "EXPORT_SECRETS",
            &state.organization_id.to_string(),
            proposal
                .get("timestampMs")
                .and_then(Value::as_str)
                .context("missing proposal timestamp")?,
            json!({"secrets":[{"secretId":state.secret_id,"targetPublicKey":state.target_public_key,"encryptionSuite":SUITE}]}),
        );
        ensure!(
            proposal == expected,
            "recovery proposal does not match its context"
        );
        ensure!(
            state.completed == state.key_material.is_none(),
            "invalid recovery completion state"
        );
        Ok(state)
    }
    fn save(&self, path: &Path, create: bool) -> Result<()> {
        #[cfg(not(unix))]
        bail!("protected Secrets files currently require a Unix filesystem");
        let encoded = Zeroizing::new(serde_json::to_vec(self).context("encode recovery state")?);
        let mut temporary =
            tempfile::NamedTempFile::new_in(path.parent().context("recovery directory missing")?)
                .context("create recovery temporary file")?;
        temporary
            .write_all(&encoded)
            .context("write recovery state")?;
        temporary
            .as_file()
            .sync_all()
            .context("persist recovery state")?;
        if create {
            temporary.persist_noclobber(path).map_err(|_| {
                anyhow::anyhow!("recovery destination already exists or cannot be created")
            })?;
        } else {
            temporary
                .persist(path)
                .map_err(|_| anyhow::anyhow!("cannot replace recovery state"))?;
        }
        File::open(path.parent().context("recovery directory missing")?)?
            .sync_all()
            .context("persist recovery directory")?;
        Ok(())
    }
}

fn fingerprint(body: &str) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(body.as_bytes())))
}
fn envelope(kind: &str, org: &str, timestamp: &str, parameters: Value) -> Value {
    json!({"type":format!("ACTIVITY_TYPE_{kind}"),"timestampMs":timestamp,"organizationId":org,"parameters":parameters})
}
fn timestamp() -> Result<String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string())
}

impl PreparedSecret {
    pub async fn run(self, auth: ResolvedAuth) -> Result<OperationOutput> {
        self.run_with_key(
            &auth.org_id,
            &auth.api_base_url,
            &auth.stamper,
            &QuorumPublicKey::production_signer(),
        )
        .await
    }
    async fn run_with_key(
        self,
        org: &str,
        endpoint: &str,
        stamper: &turnkey_api_key_stamper::TurnkeyP256ApiKey,
        quorum: &QuorumPublicKey,
    ) -> Result<OperationOutput> {
        match self {
            Self::List { limit, cursor } => {
                let response = query("secret.list", "/public/v1/query/list_secrets", &json!({"organizationId":org,"paginationOptions":{"limit":limit.to_string(),"after":cursor.map(|v|v.to_string()).unwrap_or_default()}}), endpoint, stamper).await;
                if response.failed() {
                    return Ok(response);
                }
                let decoded: turnkey_client::generated::ListSecretsResponse =
                    serde_json::from_value(
                        response.data().context("missing list response")?.clone(),
                    )
                    .map_err(|_| anyhow::anyhow!("invalid Secrets metadata response"))?;
                let next = if decoded.secrets.len() == limit as usize {
                    decoded.secrets.last().map(|s| s.secret_id.clone())
                } else {
                    None
                };
                Ok(OperationOutput::result(
                    "secret.list",
                    json!({"items":decoded.secrets,"nextCursor":next}),
                ))
            }
            Self::Import {
                name,
                plaintext,
                properties,
                init_activity_id,
            } => {
                let init_body = serde_json::to_string(&envelope(
                    "INIT_IMPORT_SECRETS",
                    org,
                    &timestamp()?,
                    json!({"encryptionSuite":SUITE,"numSecrets":1}),
                ))?;
                let init_fingerprint = fingerprint(&init_body);
                let init = if let Some(id) = init_activity_id {
                    query(
                        "secret.import",
                        "/public/v1/query/get_activity",
                        &json!({"organizationId":org,"activityId":id}),
                        endpoint,
                        stamper,
                    )
                    .await
                } else {
                    submit_bytes(
                        "secret.import",
                        "/public/v1/submit/init_import_secrets",
                        init_body,
                        endpoint,
                        stamper,
                    )
                    .await
                };
                if init.failed() {
                    return Ok(init.with_data(json!({"phase":"init-import","fingerprint":if init_activity_id.is_none(){Some(init_fingerprint.as_str())}else{None}})));
                }
                let activity = init
                    .data()
                    .and_then(|d| d.get("activity"))
                    .context("missing initialization activity")?;
                ensure!(
                    activity["organizationId"] == org
                        && activity["type"] == "ACTIVITY_TYPE_INIT_IMPORT_SECRETS",
                    "initialization activity context mismatch"
                );
                if let Some(id) = init_activity_id {
                    ensure!(
                        activity["id"] == id.to_string(),
                        "initialization activity ID mismatch"
                    );
                    let intent = activity
                        .pointer("/intent/initImportSecretsIntent")
                        .context("missing initialization intent")?;
                    ensure!(
                        intent["encryptionSuite"] == SUITE && intent["numSecrets"] == 1,
                        "initialization intent mismatch"
                    );
                } else {
                    ensure!(
                        activity["fingerprint"] == init_fingerprint,
                        "initialization fingerprint mismatch"
                    );
                }
                if activity["status"] != "ACTIVITY_STATUS_COMPLETED" {
                    let out = OperationOutput::result(
                        "secret.import",
                        json!({"phase":"init-import","activity":{"id":activity["id"],"status":activity["status"]},"nextStep":"After approval, repeat import with --init-activity-id and the original input file."}),
                    );
                    return Ok(match activity["status"].as_str() {
                        Some("ACTIVITY_STATUS_REJECTED" | "ACTIVITY_STATUS_FAILED") => out.fail("activity_failed", "Initialization was denied or failed; no secret was imported."),
                        Some("ACTIVITY_STATUS_CREATED" | "ACTIVITY_STATUS_PENDING" | "ACTIVITY_STATUS_CONSENSUS_NEEDED" | "ACTIVITY_STATUS_AUTHENTICATORS_NEEDED") => out,
                        _ => out.fail("unknown_status", "Unrecognized initialization status; inspect the activity before continuing."),
                    });
                }
                let targets = activity
                    .pointer("/result/initImportSecretsResult/enclaveTargetMessages")
                    .and_then(Value::as_array)
                    .context("missing initialization target bundle")?;
                ensure!(
                    targets.len() == 1,
                    "expected exactly one initialization target"
                );
                let bundle = targets[0]
                    .as_str()
                    .context("invalid initialization target bundle")?;
                let (payload, target) = ImportClient::new(quorum)
                    .encrypt_secret_with_bundle(&plaintext, bundle, org)
                    .context("verify initialization bundle and encrypt secret")?;
                let body = serde_json::to_string(&envelope(
                    "IMPORT_SECRETS",
                    org,
                    &timestamp()?,
                    json!({"secrets":[{"name":name,"secretPayload":payload,"targetPublicKey":target,"encryptionSuite":SUITE,"staticProperties":properties.into_iter().map(|(key,value)|json!({"key":key,"value":value})).collect::<Vec<_>>()}]}),
                ))?;
                let expected_fingerprint = fingerprint(&body);
                let result = submit_bytes(
                    "secret.import",
                    "/public/v1/submit/import_secrets",
                    body,
                    endpoint,
                    stamper,
                )
                .await;
                if result.failed() {
                    return Ok(result
                        .with_data(json!({"phase":"import","fingerprint":expected_fingerprint})));
                }
                let activity = result
                    .data()
                    .and_then(|d| d.get("activity"))
                    .context("missing import activity")?;
                ensure!(
                    activity["organizationId"] == org
                        && activity["type"] == "ACTIVITY_TYPE_IMPORT_SECRETS"
                        && activity["fingerprint"] == expected_fingerprint,
                    "import activity context mismatch"
                );
                let ids = activity.pointer("/result/importSecretsResult/secretIds");
                if activity["status"] == "ACTIVITY_STATUS_COMPLETED" {
                    ensure!(
                        ids.and_then(Value::as_array).is_some_and(|v| v.len() == 1
                            && v[0].as_str().is_some_and(|s| Uuid::parse_str(s).is_ok())),
                        "completed import omitted secret ID"
                    );
                }
                Ok(OperationOutput::result(
                    "secret.import",
                    json!({"phase":"import","secretIds":ids,"activity":{"id":activity["id"],"status":activity["status"]}}),
                ))
            }
            Self::Export {
                id,
                output,
                state_file,
                timeout,
            } => {
                let mut ikm = Zeroizing::new([0u8; 32]);
                OsRng.fill_bytes(ikm.as_mut());
                let target_public_key =
                    ExportClient::dangerous_from_bytes(ikm.as_ref(), quorum).target_public_key()?;
                let proposal = serde_json::to_string(&envelope(
                    "EXPORT_SECRETS",
                    org,
                    &timestamp()?,
                    json!({"secrets":[{"secretId":id,"targetPublicKey":target_public_key,"encryptionSuite":SUITE}]}),
                ))?;
                let mut state = ExportState {
                    version: 1,
                    organization_id: Uuid::parse_str(org)?,
                    api_base_url: endpoint.to_owned(),
                    identity_public_key: hex::encode(stamper.compressed_public_key()),
                    secret_id: id,
                    target_public_key,
                    fingerprint: fingerprint(&proposal),
                    proposal,
                    output,
                    key_material: Some(hex::encode(ikm.as_ref())),
                    activity_id: None,
                    completed: false,
                };
                let _lock = StateLock::acquire(&state_file)?;
                state.save(&state_file, true)?;
                let response = submit_bytes(
                    "secret.export",
                    "/public/v1/submit/export_secrets",
                    state.proposal.clone(),
                    endpoint,
                    stamper,
                )
                .await;
                if let Some(activity) = response.data().and_then(|v| v.get("activity")) {
                    bind_activity(&mut state, activity)?;
                    state.save(&state_file, false)?;
                }
                // Even on an ambiguous response, recover only by observation, never replay.
                if response.failed() {
                    return Ok(response.with_data(json!({"secretId":state.secret_id,"stateFile":state_file,"output":state.output,"fingerprint":state.fingerprint,"completed":false})));
                }
                recover(
                    state,
                    state_file,
                    timeout,
                    endpoint,
                    stamper,
                    quorum,
                    "secret.export",
                )
                .await
            }
            Self::Resume {
                state: _,
                state_file,
                timeout,
            } => {
                let _lock = StateLock::acquire(&state_file)?;
                let state = ExportState::read(&state_file)?;
                ensure!(
                    state.organization_id.to_string() == org
                        && state.api_base_url == endpoint
                        && state.identity_public_key
                            == hex::encode(stamper.compressed_public_key()),
                    "recovery requires the original organization, API endpoint, and credential identity"
                );
                recover(
                    state,
                    state_file,
                    timeout,
                    endpoint,
                    stamper,
                    quorum,
                    "secret.resume",
                )
                .await
            }
        }
    }
}

fn state_output(
    state: &ExportState,
    path: &Path,
    command: &'static str,
    activity: Option<&Value>,
) -> OperationOutput {
    let mut data = json!({"secretId":state.secret_id,"stateFile":path,"output":state.output,"fingerprint":state.fingerprint,"completed":state.completed});
    if let Some(activity) = activity {
        data["activity"] = json!({"id":activity["id"],"status":activity["status"]});
    }
    OperationOutput::result(command, data)
}
fn bind_activity(state: &mut ExportState, activity: &Value) -> Result<()> {
    let id = activity
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .context("missing export activity ID")?;
    ensure!(
        activity["organizationId"] == state.organization_id.to_string()
            && activity["type"] == "ACTIVITY_TYPE_EXPORT_SECRETS"
            && activity["fingerprint"] == state.fingerprint,
        "export activity does not match persisted proposal"
    );
    if let Some(expected) = &state.activity_id {
        ensure!(expected == id, "export activity ID changed");
    }
    state.activity_id = Some(id.to_owned());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn recover(
    mut state: ExportState,
    state_file: PathBuf,
    timeout: u64,
    endpoint: &str,
    stamper: &turnkey_api_key_stamper::TurnkeyP256ApiKey,
    quorum: &QuorumPublicKey,
    command: &'static str,
) -> Result<OperationOutput> {
    if state.completed {
        private_file(&state.output).context("completed export output is missing or no longer protected; recovery key was already removed")?;
        return Ok(state_output(&state, &state_file, command, None));
    }
    let ikm = Zeroizing::new(
        hex::decode(
            state
                .key_material
                .as_ref()
                .context("missing recovery key")?,
        )
        .map_err(|_| anyhow::anyhow!("invalid recovery key"))?,
    );
    ensure!(ikm.len() == 32, "invalid recovery key size");
    let mut decryptor = ExportClient::dangerous_from_bytes(&*ikm, quorum);
    ensure!(
        decryptor.target_public_key()? == state.target_public_key,
        "recovery key does not match export target"
    );
    let deadline = Instant::now() + Duration::from_secs(timeout);
    let mut last = None;
    loop {
        let activity = if let Some(id) = &state.activity_id {
            let response = query(
                command,
                "/public/v1/query/get_activity",
                &json!({"organizationId":state.organization_id,"activityId":id}),
                endpoint,
                stamper,
            )
            .await;
            if response.failed() {
                return Ok(
                    state_output(&state, &state_file, command, last.as_ref()).fail(
                        "query_failed",
                        "Could not inspect export activity. Recovery state is retained.",
                    ),
                );
            }
            Some(
                response
                    .data()
                    .and_then(|v| v.get("activity"))
                    .context("missing export activity")?
                    .clone(),
            )
        } else {
            let mut after = String::new();
            let mut seen = std::collections::HashSet::new();
            let mut found = None;
            loop {
                let response = query(command, "/public/v1/query/list_activities", &json!({"organizationId":state.organization_id,"filterByType":["ACTIVITY_TYPE_EXPORT_SECRETS"],"paginationOptions":{"limit":"100","after":after}}), endpoint, stamper).await;
                if response.failed() {
                    return Ok(state_output(&state, &state_file, command, None).fail(
                        "query_failed",
                        "Could not find export activity. Recovery state is retained.",
                    ));
                }
                let activities = response
                    .data()
                    .and_then(|v| v.get("activities"))
                    .and_then(Value::as_array)
                    .context("invalid activity list")?;
                if let Some(matched) = activities
                    .iter()
                    .find(|a| a["fingerprint"] == state.fingerprint)
                {
                    found = Some(matched.clone());
                    break;
                }
                if activities.len() < 100 || Instant::now() >= deadline {
                    break;
                }
                after = activities
                    .last()
                    .and_then(|v| v["id"].as_str())
                    .context("missing pagination cursor")?
                    .to_owned();
                ensure!(
                    seen.insert(after.clone()),
                    "activity pagination did not advance"
                );
            }
            found
        };
        if let Some(activity) = activity {
            bind_activity(&mut state, &activity)?;
            state.save(&state_file, false)?;
            match activity["status"]
                .as_str()
                .context("missing export status")?
            {
                "ACTIVITY_STATUS_COMPLETED" => {
                    let bundles = activity
                        .pointer("/result/exportSecretsResult/secretPayloads")
                        .and_then(Value::as_array)
                        .context("completed export omitted payload")?;
                    ensure!(bundles.len() == 1, "expected exactly one exported payload");
                    let bundle = bundles[0].as_str().context("invalid export payload")?;
                    let plaintext = Zeroizing::new(
                        decryptor
                            .decrypt_secret(bundle, &state.organization_id.to_string())
                            .context("verify and decrypt export bundle")?,
                    );
                    // A crash may have committed output before the state update. Reuse only
                    // an identical protected file; never overwrite any existing destination.
                    if fs::symlink_metadata(&state.output).is_ok() {
                        let existing = read_private(&state.output, MAX_SECRET_BYTES + 1)?;
                        ensure!(
                            *existing == *plaintext,
                            "existing output differs from the authenticated export; choose no replacement and inspect recovery state"
                        );
                    } else {
                        let mut temporary = tempfile::NamedTempFile::new_in(
                            state.output.parent().context("missing output parent")?,
                        )?;
                        temporary
                            .write_all(&plaintext)
                            .context("write exported secret")?;
                        temporary
                            .as_file()
                            .sync_all()
                            .context("persist exported secret")?;
                        temporary.persist_noclobber(&state.output).map_err(|_| {
                            anyhow::anyhow!("output already exists or cannot be created")
                        })?;
                        File::open(state.output.parent().context("missing output parent")?)?
                            .sync_all()
                            .context("persist output directory")?;
                    }
                    File::open(&state.output)?
                        .sync_all()
                        .context("persist verified output")?;
                    File::open(state.output.parent().context("missing output parent")?)?
                        .sync_all()
                        .context("persist output directory")?;
                    state.completed = true;
                    if let Some(mut key) = state.key_material.take() {
                        key.zeroize();
                    }
                    state.save(&state_file, false)?;
                    return Ok(state_output(&state, &state_file, command, Some(&activity)));
                }
                "ACTIVITY_STATUS_REJECTED" | "ACTIVITY_STATUS_FAILED" => {
                    return Ok(
                        state_output(&state, &state_file, command, Some(&activity)).fail(
                            "activity_failed",
                            "Export was denied or failed. No plaintext file was written.",
                        ),
                    );
                }
                "ACTIVITY_STATUS_CREATED"
                | "ACTIVITY_STATUS_PENDING"
                | "ACTIVITY_STATUS_CONSENSUS_NEEDED"
                | "ACTIVITY_STATUS_AUTHENTICATORS_NEEDED" => (),
                _ => bail!("unrecognized export activity status; recovery state is retained"),
            }
            last = Some(activity);
        }
        if Instant::now() >= deadline {
            return Ok(state_output(&state, &state_file, command, last.as_ref()).fail("wait_timeout", "Export is not complete. Run secret resume with the saved state file; no submission was replayed."));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

struct StateLock(PathBuf);
impl StateLock {
    fn acquire(path: &Path) -> Result<Self> {
        let mut lock = path.as_os_str().to_owned();
        lock.push(".lock");
        let lock = PathBuf::from(lock);
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options.open(&lock).context("recovery state is locked; if a previous process crashed, remove its .lock file only after confirming it has stopped")?;
        Ok(Self(lock))
    }
}
impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn private_file(path: &Path) -> Result<File> {
    #[cfg(not(unix))]
    bail!("protected Secrets files currently require a Unix filesystem");
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).context("open protected file")?;
    let metadata = file.metadata().context("inspect protected file")?;
    ensure!(metadata.is_file(), "protected file must be regular");
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        ensure!(
            metadata.permissions().mode() & 0o077 == 0 && metadata.nlink() == 1,
            "protected file must be private (mode 0600) with one link"
        );
    }
    Ok(file)
}

fn read_private(path: &Path, limit: u64) -> Result<Zeroizing<Vec<u8>>> {
    let file = private_file(path)?;
    let mut encoded = Zeroizing::new(Vec::new());
    file.take(limit + 1)
        .read_to_end(&mut encoded)
        .context("read protected file")?;
    ensure!(
        encoded.len() as u64 <= limit,
        "protected file exceeds size limit"
    );
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use p256::ecdsa::SigningKey;
    use std::sync::{Arc, Mutex};
    use turnkey_api_key_stamper::TurnkeyP256ApiKey;
    use turnkey_enclave_encrypt::{P256Public, server::EnclaveEncryptServer};
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{method, path},
    };
    const ORG: &str = "00000000-0000-4000-8000-000000000001";
    const SECRET: &str = "00000000-0000-4000-8000-000000000002";
    const ACTIVITY: &str = "00000000-0000-4000-8000-000000000003";
    fn quorum() -> (SigningKey, QuorumPublicKey) {
        let key = SigningKey::random(&mut OsRng);
        let public = key.verifying_key().to_encoded_point(false);
        let bytes = [public.as_bytes(), public.as_bytes()].concat();
        (key, QuorumPublicKey::from_bytes(bytes).unwrap())
    }
    fn activity(body: &str, status: &str, result: Value) -> Value {
        let parsed: Value = serde_json::from_str(body).unwrap();
        json!({"activity":{"id":ACTIVITY,"organizationId":parsed["organizationId"],"type":parsed["type"],"fingerprint":fingerprint(body),"status":status,"intent":{"exportSecretsIntent":parsed["parameters"]},"result":result}})
    }
    #[derive(Parser)]
    struct ParserCli {
        #[command(subcommand)]
        command: SecretCommand,
    }
    #[test]
    fn parser_rejects_secret_arguments_and_unsafe_shapes() {
        for args in [
            vec!["tk", "export", SECRET],
            vec![
                "tk",
                "export",
                "bad",
                "--output",
                "out",
                "--state-file",
                "state",
            ],
            vec!["tk", "import", "--name", "x", "--input-json", "secret"],
            vec!["tk", "list", "--limit", "0"],
            vec!["tk", "resume", "--state-file", "state", "--timeout", "0"],
        ] {
            assert!(ParserCli::try_parse_from(args).is_err());
        }
    }
    #[test]
    fn paths_and_properties_fail_before_authentication() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        fs::write(&out, b"original").unwrap();
        assert!(
            SecretCommand::Export {
                id: Uuid::parse_str(SECRET).unwrap(),
                output: out.clone(),
                state_file: dir.path().join("state"),
                timeout: 1
            }
            .prepare()
            .is_err()
        );
        assert_eq!(fs::read(&out).unwrap(), b"original");
        let properties = dir.path().join("properties");
        fs::write(&properties, br#"{"sensitive":42}"#).unwrap();
        assert!(
            SecretCommand::Import {
                name: "x".into(),
                input_file: out,
                static_properties_file: Some(properties),
                init_activity_id: None
            }
            .prepare()
            .is_err()
        );
    }
    #[tokio::test]
    async fn metadata_list_paginates_and_strips_unknown_payload_fields() {
        let server = MockServer::start().await;
        Mock::given(path("/public/v1/query/list_secrets")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"secrets":[{"secretId":SECRET,"name":"token","secretPayload":"must-not-output"}]}))).expect(1).mount(&server).await;
        let (_, q) = quorum();
        let out = PreparedSecret::List {
            limit: 1,
            cursor: Some(Uuid::parse_str(ACTIVITY).unwrap()),
        }
        .run_with_key(ORG, &server.uri(), &TurnkeyP256ApiKey::generate(), &q)
        .await
        .unwrap();
        assert_eq!(
            out.data().unwrap(),
            &json!({"items":[{"secretId":SECRET,"name":"token","staticProperties":[],"createdAtUnixMs":"0"}],"nextCursor":SECRET})
        );
        let body: Value =
            serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
        assert_eq!(
            body["paginationOptions"],
            json!({"limit":"1","after":ACTIVITY})
        );
    }
    #[tokio::test]
    async fn import_encrypts_bytes_and_submits_only_ciphertext() {
        let server = MockServer::start().await;
        let (signing, q) = quorum();
        let enclave =
            EnclaveEncryptServer::from_enclave_auth_key(signing, ORG.into(), Some("user".into()));
        let bundle = serde_json::to_string(&enclave.publish_target().unwrap()).unwrap();
        let mut receiver = enclave.into_recv();
        Mock::given(path("/public/v1/submit/init_import_secrets"))
            .respond_with(move |r: &Request| {
                ResponseTemplate::new(200).set_body_json(activity(
                    std::str::from_utf8(&r.body).unwrap(),
                    "ACTIVITY_STATUS_COMPLETED",
                    json!({"initImportSecretsResult":{"enclaveTargetMessages":[bundle]}}),
                ))
            })
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/public/v1/submit/import_secrets"))
            .respond_with(|r: &Request| {
                ResponseTemplate::new(200).set_body_json(activity(
                    std::str::from_utf8(&r.body).unwrap(),
                    "ACTIVITY_STATUS_COMPLETED",
                    json!({"importSecretsResult":{"secretIds":[SECRET]}}),
                ))
            })
            .expect(1)
            .mount(&server)
            .await;
        let plaintext = b"synthetic-token-\x00\xff";
        let out = PreparedSecret::Import {
            name: "demo".into(),
            plaintext: Zeroizing::new(plaintext.to_vec()),
            properties: std::collections::BTreeMap::from([("purpose".into(), "demo".into())]),
            init_activity_id: None,
        }
        .run_with_key(ORG, &server.uri(), &TurnkeyP256ApiKey::generate(), &q)
        .await
        .unwrap();
        assert_eq!(out.data().unwrap()["secretIds"], json!([SECRET]));
        let requests = server.received_requests().await.unwrap();
        let body: Value = serde_json::from_slice(&requests[1].body).unwrap();
        let payload = serde_json::from_str(
            body["parameters"]["secrets"][0]["secretPayload"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(receiver.decrypt(&payload).unwrap(), plaintext);
        assert_eq!(
            body["parameters"]["secrets"][0]["staticProperties"],
            json!([{"key":"purpose","value":"demo"}])
        );
        assert!(!String::from_utf8_lossy(&requests[1].body).contains("synthetic-token"));
        assert!(
            !serde_json::to_string(&out)
                .unwrap()
                .contains("synthetic-token")
        );
    }
    #[tokio::test]
    async fn export_persists_exact_proposal_before_submit_and_scrubs_key_after_durable_output() {
        let server = MockServer::start().await;
        let (signing, q) = quorum();
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("secret");
        let state_path = dir.path().join("state");
        let check_path = state_path.clone();
        let response = Arc::new(Mutex::new(Value::Null));
        let stored = response.clone();
        Mock::given(method("POST"))
            .and(path("/public/v1/submit/export_secrets"))
            .respond_with(move |r: &Request| {
                let body = std::str::from_utf8(&r.body).unwrap();
                let state = ExportState::read(&check_path).unwrap();
                assert_eq!(state.proposal, body);
                let parsed: Value = serde_json::from_str(body).unwrap();
                let target: P256Public = hex::decode(
                    parsed["parameters"]["secrets"][0]["targetPublicKey"]
                        .as_str()
                        .unwrap(),
                )
                .unwrap()
                .try_into()
                .unwrap();
                let enclave =
                    EnclaveEncryptServer::from_enclave_auth_key(signing.clone(), ORG.into(), None);
                let bundle = serde_json::to_string(
                    &enclave
                        .encrypt(&target, b"synthetic secret\x00\xff")
                        .unwrap(),
                )
                .unwrap();
                let value = activity(
                    body,
                    "ACTIVITY_STATUS_COMPLETED",
                    json!({"exportSecretsResult":{"secretPayloads":[bundle]}}),
                );
                *stored.lock().unwrap() = value.clone();
                ResponseTemplate::new(200).set_body_json(value)
            })
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(move |_: &Request| {
                ResponseTemplate::new(200).set_body_json(response.lock().unwrap().clone())
            })
            .mount(&server)
            .await;
        let key = TurnkeyP256ApiKey::generate();
        let out = PreparedSecret::Export {
            id: Uuid::parse_str(SECRET).unwrap(),
            output: output.clone(),
            state_file: state_path.clone(),
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(!out.failed());
        assert_eq!(fs::read(&output).unwrap(), b"synthetic secret\x00\xff");
        let state = ExportState::read(&state_path).unwrap();
        assert!(state.completed);
        assert!(state.key_material.is_none());
        let out = PreparedSecret::Resume {
            state,
            state_file: state_path,
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(!out.failed());
        assert!(
            !serde_json::to_string(&out)
                .unwrap()
                .contains("synthetic secret")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(output).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    fn recovery_state(
        endpoint: &str,
        key: &TurnkeyP256ApiKey,
        q: &QuorumPublicKey,
        output: PathBuf,
    ) -> ExportState {
        let ikm = [7u8; 32];
        let target = ExportClient::dangerous_from_bytes(ikm, q)
            .target_public_key()
            .unwrap();
        let proposal=serde_json::to_string(&envelope("EXPORT_SECRETS",ORG,"1",json!({"secrets":[{"secretId":SECRET,"targetPublicKey":target,"encryptionSuite":SUITE}]}))).unwrap();
        ExportState {
            version: 1,
            organization_id: Uuid::parse_str(ORG).unwrap(),
            api_base_url: endpoint.into(),
            identity_public_key: hex::encode(key.compressed_public_key()),
            secret_id: Uuid::parse_str(SECRET).unwrap(),
            target_public_key: target,
            fingerprint: fingerprint(&proposal),
            proposal,
            output,
            key_material: Some(hex::encode(ikm)),
            activity_id: None,
            completed: false,
        }
    }
    #[tokio::test]
    async fn unknown_submission_resume_discovers_fingerprint_and_denial_never_replays() {
        let server = MockServer::start().await;
        let (_, q) = quorum();
        let key = TurnkeyP256ApiKey::generate();
        let dir = tempfile::tempdir().unwrap();
        let path_state = dir.path().join("state");
        let state = recovery_state(&server.uri(), &key, &q, dir.path().join("out"));
        let value = activity(&state.proposal, "ACTIVITY_STATUS_REJECTED", Value::Null);
        state.save(&path_state, true).unwrap();
        Mock::given(path("/public/v1/query/list_activities"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"activities":[value["activity"]]})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let out = PreparedSecret::Resume {
            state,
            state_file: path_state.clone(),
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(out.failed());
        assert_eq!(out.data().unwrap()["activity"]["id"], ACTIVITY);
        assert!(!dir.path().join("out").exists());
        assert!(
            ExportState::read(&path_state)
                .unwrap()
                .key_material
                .is_some()
        );
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
    #[tokio::test]
    async fn pending_times_out_with_recovery_and_wrong_identity_makes_no_request() {
        let server = MockServer::start().await;
        let (_, q) = quorum();
        let key = TurnkeyP256ApiKey::generate();
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join("state");
        let mut state = recovery_state(&server.uri(), &key, &q, dir.path().join("out"));
        state.activity_id = Some(ACTIVITY.into());
        let value = activity(
            &state.proposal,
            "ACTIVITY_STATUS_CONSENSUS_NEEDED",
            Value::Null,
        );
        state.save(&state_path, true).unwrap();
        Mock::given(path("/public/v1/query/get_activity"))
            .respond_with(ResponseTemplate::new(200).set_body_json(value))
            .mount(&server)
            .await;
        let out = PreparedSecret::Resume {
            state,
            state_file: state_path.clone(),
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(out.failed());
        assert_eq!(serde_json::to_value(out).unwrap()["code"], "wait_timeout");
        server.reset().await;
        let before = server.received_requests().await.unwrap().len();
        let state = ExportState::read(&state_path).unwrap();
        assert!(
            PreparedSecret::Resume {
                state,
                state_file: state_path,
                timeout: 1
            }
            .run_with_key(ORG, &server.uri(), &TurnkeyP256ApiKey::generate(), &q)
            .await
            .is_err()
        );
        assert_eq!(server.received_requests().await.unwrap().len(), before);
    }
    #[tokio::test]
    async fn recovery_rejects_wrong_activity_or_corrupt_bundle_without_output() {
        for corrupt_bundle in [false, true] {
            let server = MockServer::start().await;
            let (_, q) = quorum();
            let key = TurnkeyP256ApiKey::generate();
            let dir = tempfile::tempdir().unwrap();
            let state_path = dir.path().join("state");
            let mut state = recovery_state(&server.uri(), &key, &q, dir.path().join("out"));
            state.activity_id = Some(ACTIVITY.into());
            let mut value = activity(
                &state.proposal,
                "ACTIVITY_STATUS_COMPLETED",
                json!({"exportSecretsResult":{"secretPayloads":["malformed"]}}),
            );
            if !corrupt_bundle {
                value["activity"]["fingerprint"] = json!("sha256:wrong");
            }
            state.save(&state_path, true).unwrap();
            Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(value))
                .mount(&server)
                .await;
            assert!(
                PreparedSecret::Resume {
                    state,
                    state_file: state_path.clone(),
                    timeout: 1
                }
                .run_with_key(ORG, &server.uri(), &key, &q)
                .await
                .is_err()
            );
            assert!(!dir.path().join("out").exists());
            assert!(
                ExportState::read(&state_path)
                    .unwrap()
                    .key_material
                    .is_some()
            );
        }
    }

    #[tokio::test]
    async fn malformed_submission_is_unknown_and_resume_does_not_submit_again() {
        let server = MockServer::start().await;
        let (_, q) = quorum();
        let key = TurnkeyP256ApiKey::generate();
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join("state");
        Mock::given(path("/public/v1/submit/export_secrets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .expect(1)
            .mount(&server)
            .await;
        let out = PreparedSecret::Export {
            id: Uuid::parse_str(SECRET).unwrap(),
            output: dir.path().join("out"),
            state_file: state_path.clone(),
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert_eq!(
            serde_json::to_value(&out).unwrap()["code"],
            "submission_unknown"
        );
        assert_eq!(
            out.data().unwrap()["stateFile"],
            state_path.to_str().unwrap()
        );
        let state = ExportState::read(&state_path).unwrap();
        Mock::given(path("/public/v1/query/list_activities"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"activities":[]})))
            .mount(&server)
            .await;
        let out = PreparedSecret::Resume {
            state,
            state_file: state_path,
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(out.failed());
        server.verify().await;
    }
    #[tokio::test]
    async fn crash_after_output_publication_reconciles_only_identical_private_bytes() {
        for identical in [true, false] {
            let server = MockServer::start().await;
            let (signing, q) = quorum();
            let key = TurnkeyP256ApiKey::generate();
            let dir = tempfile::tempdir().unwrap();
            let state_path = dir.path().join("state");
            let output = dir.path().join("out");
            let mut state = recovery_state(&server.uri(), &key, &q, output.clone());
            state.activity_id = Some(ACTIVITY.into());
            let target: P256Public = hex::decode(&state.target_public_key)
                .unwrap()
                .try_into()
                .unwrap();
            let enclave = EnclaveEncryptServer::from_enclave_auth_key(signing, ORG.into(), None);
            let bundle =
                serde_json::to_string(&enclave.encrypt(&target, b"synthetic token").unwrap())
                    .unwrap();
            let response = activity(
                &state.proposal,
                "ACTIVITY_STATUS_COMPLETED",
                json!({"exportSecretsResult":{"secretPayloads":[bundle]}}),
            );
            state.save(&state_path, true).unwrap();
            let mut file = tempfile::NamedTempFile::new_in(dir.path()).unwrap();
            file.write_all(if identical {
                b"synthetic token"
            } else {
                b"other bytes"
            })
            .unwrap();
            file.persist_noclobber(&output).unwrap();
            Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(response))
                .mount(&server)
                .await;
            let result = PreparedSecret::Resume {
                state,
                state_file: state_path.clone(),
                timeout: 1,
            }
            .run_with_key(ORG, &server.uri(), &key, &q)
            .await;
            assert_eq!(result.is_ok(), identical);
            assert_eq!(ExportState::read(&state_path).unwrap().completed, identical);
            assert_eq!(
                fs::read(&output).unwrap(),
                if identical {
                    b"synthetic token".to_vec()
                } else {
                    b"other bytes".to_vec()
                }
            );
        }
    }
    #[tokio::test]
    async fn initialization_resume_denial_or_wrong_id_never_imports() {
        for wrong_id in [true, false] {
            let server = MockServer::start().await;
            let (_, q) = quorum();
            let response = json!({"activity":{"id":if wrong_id {SECRET} else {ACTIVITY},"organizationId":ORG,"type":"ACTIVITY_TYPE_INIT_IMPORT_SECRETS","status":"ACTIVITY_STATUS_REJECTED","intent":{"initImportSecretsIntent":{"encryptionSuite":SUITE,"numSecrets":1}}}});
            Mock::given(path("/public/v1/query/get_activity"))
                .respond_with(ResponseTemplate::new(200).set_body_json(response))
                .expect(1)
                .mount(&server)
                .await;
            let result = PreparedSecret::Import {
                name: "demo".into(),
                plaintext: Zeroizing::new(b"synthetic".to_vec()),
                properties: Default::default(),
                init_activity_id: Some(Uuid::parse_str(ACTIVITY).unwrap()),
            }
            .run_with_key(ORG, &server.uri(), &TurnkeyP256ApiKey::generate(), &q)
            .await;
            if wrong_id {
                assert!(result.is_err());
            } else {
                assert!(result.unwrap().failed());
            }
            assert_eq!(server.received_requests().await.unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn unknown_recovery_walks_older_activity_pages_using_after() {
        let server = MockServer::start().await;
        let (_, q) = quorum();
        let key = TurnkeyP256ApiKey::generate();
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join("state");
        let state = recovery_state(&server.uri(), &key, &q, dir.path().join("out"));
        let matched = activity(&state.proposal, "ACTIVITY_STATUS_REJECTED", Value::Null);
        state.save(&state_path, true).unwrap();
        Mock::given(path("/public/v1/query/list_activities")).respond_with(move |r:&Request| {
            let body:Value=serde_json::from_slice(&r.body).unwrap();let cursor=body["paginationOptions"]["after"].as_str().unwrap();
            if cursor.is_empty() { ResponseTemplate::new(200).set_body_json(json!({"activities":(0..100).map(|i|json!({"id":format!("newer-{i}"),"fingerprint":"other"})).collect::<Vec<_>>()})) }
            else { assert_eq!(cursor,"newer-99"); ResponseTemplate::new(200).set_body_json(json!({"activities":[matched["activity"]]})) }
        }).expect(2).mount(&server).await;
        let out = PreparedSecret::Resume {
            state,
            state_file: state_path,
            timeout: 1,
        }
        .run_with_key(ORG, &server.uri(), &key, &q)
        .await
        .unwrap();
        assert!(out.failed());
        assert_eq!(out.data().unwrap()["activity"]["id"], ACTIVITY);
        server.verify().await;
    }
    #[test]
    fn recovery_schema_rejects_proposal_or_completion_tampering() {
        let (_, q) = quorum();
        let key = TurnkeyP256ApiKey::generate();
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join("state");
        let mut state = recovery_state("https://api.turnkey.com", &key, &q, dir.path().join("out"));
        state.fingerprint = "wrong".into();
        state.save(&state_path, true).unwrap();
        assert!(ExportState::read(&state_path).is_err());
        state.fingerprint = fingerprint(&state.proposal);
        state.completed = true;
        state.save(&state_path, false).unwrap();
        assert!(ExportState::read(&state_path).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn recovery_refuses_symlinks_hardlinks_world_readable_and_concurrent_access() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        fs::write(&target, b"{}").unwrap();
        let link = dir.path().join("link");
        symlink(&target, &link).unwrap();
        assert!(read_private(&link, 100).is_err());
        assert!(new_path(link).is_err());
        fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_private(&target, 100).is_err());
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
        fs::hard_link(&target, dir.path().join("hard")).unwrap();
        assert!(read_private(&target, 100).is_err());
        let _lock = StateLock::acquire(&target).unwrap();
        assert!(StateLock::acquire(&target).is_err());
    }
}
