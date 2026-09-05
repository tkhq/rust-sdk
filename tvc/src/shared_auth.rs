//! Shared identity selection independent of TVC operator configuration.

use crate::{
    client::{AuthenticatedClient, build_turnkey_client},
    config::turnkey::{Config, KeyCurve, StoredApiKey},
};
use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_client::generated::GetWhoamiRequest;
use uuid::Uuid;

const DEFAULT_URL: &str = "https://api.turnkey.com";

/// Global identity selectors. Credential secrets are never command-line arguments.
#[derive(Debug, Default, Args)]
pub struct AuthOptions {
    #[arg(long, global = true, env = "TK_CONFIG")]
    config: Option<PathBuf>,
    #[arg(long, global = true, env = "TK_PROFILE")]
    profile: Option<String>,
    #[arg(long, global = true)]
    organization_id: Option<Uuid>,
    #[arg(long, global = true)]
    api_base_url: Option<String>,
}

impl AuthOptions {
    pub(crate) fn has_selection(&self) -> bool {
        self.config.is_some()
            || self.profile.is_some()
            || self.organization_id.is_some()
            || self.api_base_url.is_some()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub(crate) struct InvalidAuthInput(pub String);

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Save and select an existing API credential after verifying it remotely.
    Login(LoginArgs),
    /// Inspect local credential readiness without contacting the server.
    Status,
    /// Verify the selected identity with Turnkey.
    Whoami,
    /// Clear the saved profile selection; keep credentials and remote access intact.
    Logout,
}

#[derive(Debug, Args)]
pub struct LoginArgs {
    /// Existing P256 credential JSON file (public_key, private_key, curve).
    #[arg(long)]
    api_key_file: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum ProfileCommand {
    List,
    Show { name: String },
    Use { name: String },
    Delete { name: String },
    Import(ImportArgs),
}

#[derive(Debug, Clone, ValueEnum)]
enum ImportSource {
    Tvc,
    ExperimentalTk,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(long, value_enum)]
    from: ImportSource,
    #[arg(long)]
    name: String,
    #[arg(long)]
    source: Option<PathBuf>,
    #[arg(long)]
    source_profile: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    version: u32,
    active_profile: Option<String>,
    #[serde(default)]
    profiles: BTreeMap<String, Profile>,
}
impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            active_profile: None,
            profiles: BTreeMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Profile {
    organization_id: Uuid,
    api_base_url: String,
    api_key_file: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh_signing_key_id: Option<String>,
}

/// Resolved identity; intentionally does not implement Debug or Serialize.
pub struct ResolvedAuth {
    pub org_id: String,
    pub api_base_url: String,
    pub stamper: TurnkeyP256ApiKey,
    source: &'static str,
    profile: Option<String>,
}
impl ResolvedAuth {
    pub fn into_client(self) -> Result<AuthenticatedClient> {
        Ok(AuthenticatedClient {
            client: build_turnkey_client(self.stamper, &self.api_base_url)?,
            org_id: self.org_id,
            api_base_url: self.api_base_url,
        })
    }
}

/// A shared command result, emitted by the common presentation boundary.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOutput {
    schema_version: u32,
    reason: &'static str,
    command: &'static str,
    status: &'static str,
    data: Value,
}
impl Display for AuthOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}
fn result(command: &'static str, data: Value) -> AuthOutput {
    AuthOutput {
        schema_version: 1,
        reason: "command_result",
        command,
        status: "completed",
        data,
    }
}

fn env(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}
fn home() -> Result<PathBuf> {
    env("HOME")
        .map(PathBuf::from)
        .context("HOME is required when no explicit configuration path is supplied")
}
fn registry_path(options: &AuthOptions) -> Result<PathBuf> {
    options
        .config
        .clone()
        .map(Ok)
        .unwrap_or_else(|| Ok(home()?.join(".config/turnkey/tk.config.toml")))
}
fn load(path: &Path) -> Result<Registry> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Registry::default()),
        Err(e) => return Err(e).with_context(|| format!("read registry {}", path.display())),
    };
    // Avoid including source lines, which could contain accidentally pasted secrets.
    let registry: Registry = toml::from_str(&text).map_err(|_| {
        anyhow::Error::from(InvalidAuthInput(format!(
            "invalid identity registry {}",
            path.display()
        )))
    })?;
    if registry.version != 1 {
        bail!(
            "unsupported registry version {} in {}",
            registry.version,
            path.display()
        );
    }
    Ok(registry)
}
// Serialize registry mutations so concurrent agents cannot silently lose profiles.
struct RegistryLock(PathBuf);
impl RegistryLock {
    fn acquire(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lock = path.with_extension("lock");
        secure_create(&lock, std::process::id().to_string().as_bytes())
            .context("identity registry is locked; retry after the other writer completes")?;
        Ok(Self(lock))
    }
}
impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn save(path: &Path, registry: &Registry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let content = toml::to_string_pretty(registry)?;
    secure_create(&temporary, content.as_bytes())?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(error).context("replace identity registry");
    }
    Ok(())
}
fn secure_create(path: &Path, contents: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).with_context(|| {
        format!(
            "create {} without overwriting existing files",
            path.display()
        )
    })?;
    if let Err(error) = file.write_all(contents).and_then(|_| file.sync_all()) {
        let _ = fs::remove_file(path);
        return Err(error).context("write protected file");
    }
    Ok(())
}
fn parse_key(private: &str, public: &str) -> Result<TurnkeyP256ApiKey> {
    let bytes = hex::decode(private).context("private credential must be hexadecimal")?;
    if bytes.len() != 32 {
        bail!("P256 private credentials must contain exactly 32 bytes");
    }
    TurnkeyP256ApiKey::from_strings(private, Some(public)).context("invalid P256 credential pair")
}

fn read_key(path: &Path) -> Result<TurnkeyP256ApiKey> {
    let text =
        fs::read_to_string(path).with_context(|| format!("read credential {}", path.display()))?;
    let key: StoredApiKey = serde_json::from_str(&text)
        .map_err(|_| anyhow::anyhow!("invalid credential JSON in {}", path.display()))?;
    if !matches!(key.curve, KeyCurve::P256) {
        bail!("shared authentication supports only P256 credentials");
    }
    parse_key(&key.private_key, &key.public_key)
}
fn endpoint(options: &AuthOptions, fallback: String) -> Result<String> {
    let endpoint = options
        .api_base_url
        .clone()
        .or_else(|| env("TURNKEY_API_BASE_URL"))
        .unwrap_or(fallback);
    parse_endpoint(endpoint)
}
fn parse_endpoint(endpoint: String) -> Result<String> {
    let url = reqwest::Url::parse(&endpoint).context("invalid API base URL")?;
    if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        bail!("API base URL must be an HTTP(S) URL without credentials, query or fragment");
    }
    Ok(endpoint)
}

/// Resolve a complete identity without mixing credentials across sources.
pub async fn resolve(options: &AuthOptions) -> Result<ResolvedAuth> {
    if options.profile.is_none() {
        let canonical = [
            "TURNKEY_ORGANIZATION_ID",
            "TURNKEY_API_PUBLIC_KEY",
            "TURNKEY_API_PRIVATE_KEY",
        ]
        .map(std::env::var_os);
        let legacy =
            ["TVC_ORG_ID", "TVC_API_KEY_PUBLIC", "TVC_API_KEY_PRIVATE"].map(std::env::var_os);
        let has_canonical = canonical.iter().any(Option::is_some);
        let has_legacy = legacy.iter().any(Option::is_some);
        if has_canonical && has_legacy {
            bail!(
                "conflicting TURNKEY and TVC credential bundles; select --profile or supply only one bundle"
            );
        }
        if has_canonical || has_legacy {
            let [org, public, private] = if has_canonical { canonical } else { legacy };
            let (Some(org), Some(public), Some(private)) = (org, public, private) else {
                bail!(
                    "partial credential environment: organization ID, public key, and private key are all required"
                );
            };
            let org = org.into_string().map_err(|_| {
                InvalidAuthInput("organization environment value is not valid Unicode".into())
            })?;
            let public = public.into_string().map_err(|_| {
                InvalidAuthInput("public key environment value is not valid Unicode".into())
            })?;
            let private = private.into_string().map_err(|_| {
                InvalidAuthInput("private key environment value is not valid Unicode".into())
            })?;
            if org.is_empty() || public.is_empty() || private.is_empty() {
                return Err(InvalidAuthInput(
                    "credential environment fields must not be empty".into(),
                )
                .into());
            }
            let org = match options.organization_id {
                Some(org) => org,
                None => Uuid::parse_str(&org).context("invalid environment organization ID")?,
            };
            return Ok(ResolvedAuth {
                org_id: org.to_string(),
                api_base_url: endpoint(
                    options,
                    if has_legacy {
                        env("TVC_API_BASE_URL").unwrap_or_else(|| DEFAULT_URL.into())
                    } else {
                        DEFAULT_URL.into()
                    },
                )?,
                stamper: parse_key(&private, &public)?,
                source: if has_canonical {
                    "environment"
                } else {
                    "legacy_environment"
                },
                profile: None,
            });
        }
    }
    let path = registry_path(options)?;
    let registry = load(&path)?;
    let name = options
        .profile
        .as_ref()
        .or(registry.active_profile.as_ref())
        .context("no selected identity; use --profile or tk login")?;
    let profile = registry
        .profiles
        .get(name)
        .with_context(|| format!("profile {name} does not exist in {}", path.display()))?;
    Ok(ResolvedAuth {
        org_id: options
            .organization_id
            .unwrap_or(profile.organization_id)
            .to_string(),
        api_base_url: endpoint(options, profile.api_base_url.clone())?,
        stamper: read_key(&profile.api_key_file)?,
        source: "profile",
        profile: Some(name.clone()),
    })
}

pub async fn run_auth(command: AuthCommand, options: &AuthOptions) -> Result<AuthOutput> {
    match command {
        AuthCommand::Status => {
            let auth = resolve(options).await?;
            Ok(result(
                "auth.status",
                json!({"ready": true, "profile": auth.profile, "organizationId": auth.org_id, "apiBaseUrl": auth.api_base_url, "publicKey": hex::encode(auth.stamper.compressed_public_key()), "credentialSource": auth.source}),
            ))
        }
        AuthCommand::Whoami => {
            let auth = resolve(options).await?.into_client()?;
            let identity = auth
                .client
                .get_whoami(GetWhoamiRequest {
                    organization_id: auth.org_id,
                })
                .await?;
            Ok(result("auth.whoami", serde_json::to_value(identity)?))
        }
        AuthCommand::Logout => {
            let path = registry_path(options)?;
            let _lock = RegistryLock::acquire(&path)?;
            let mut registry = load(&path)?;
            registry.active_profile = None;
            save(&path, &registry)?;
            Ok(result(
                "auth.logout",
                json!({"activeProfile": null, "environmentCredentialsPresent": (["TURNKEY_ORGANIZATION_ID", "TURNKEY_API_PUBLIC_KEY", "TURNKEY_API_PRIVATE_KEY", "TVC_ORG_ID", "TVC_API_KEY_PUBLIC", "TVC_API_KEY_PRIVATE"].iter().any(|name| env(name).is_some()))}),
            ))
        }
        AuthCommand::Login(args) => {
            let name = options
                .profile
                .as_ref()
                .context("login requires --profile NAME")?;
            let org = options
                .organization_id
                .context("login requires --organization-id")?;
            let path = registry_path(options)?;
            let _lock = RegistryLock::acquire(&path)?;
            let mut registry = load(&path)?;
            if registry.profiles.contains_key(name) {
                bail!("profile {name} already exists; use profile use to select it");
            }
            let key_path =
                fs::canonicalize(args.api_key_file).context("resolve credential path")?;
            let base_url = endpoint(options, DEFAULT_URL.into())?;
            let client = build_turnkey_client(read_key(&key_path)?, &base_url)?;
            let identity = client
                .get_whoami(GetWhoamiRequest {
                    organization_id: org.to_string(),
                })
                .await?;
            registry.profiles.insert(
                name.clone(),
                Profile {
                    organization_id: org,
                    api_base_url: base_url,
                    api_key_file: key_path,
                    ssh_signing_key_id: None,
                },
            );
            registry.active_profile = Some(name.clone());
            save(&path, &registry)?;
            Ok(result(
                "auth.login",
                json!({"profile": name, "identity": identity}),
            ))
        }
    }
}

pub async fn run_profile(command: ProfileCommand, options: &AuthOptions) -> Result<AuthOutput> {
    let path = registry_path(options)?;
    let _lock = if matches!(&command, ProfileCommand::List | ProfileCommand::Show { .. }) {
        None
    } else {
        Some(RegistryLock::acquire(&path)?)
    };
    let mut registry = load(&path)?;
    match command {
        ProfileCommand::List => Ok(result(
            "profile.list",
            json!({"activeProfile": registry.active_profile, "profiles": registry.profiles}),
        )),
        ProfileCommand::Show { name } => Ok(result(
            "profile.show",
            json!({"name": name, "profile": registry.profiles.get(&name).context("profile does not exist")?}),
        )),
        ProfileCommand::Use { name } => {
            let profile = registry
                .profiles
                .get(&name)
                .context("profile does not exist")?;
            read_key(&profile.api_key_file)?;
            registry.active_profile = Some(name.clone());
            save(&path, &registry)?;
            Ok(result("profile.use", json!({"activeProfile": name})))
        }
        ProfileCommand::Delete { name } => {
            registry
                .profiles
                .remove(&name)
                .context("profile does not exist")?;
            if registry.active_profile.as_ref() == Some(&name) {
                registry.active_profile = None;
            }
            save(&path, &registry)?;
            Ok(result(
                "profile.delete",
                json!({"name": name, "credentialFilesDeleted": false}),
            ))
        }
        ProfileCommand::Import(args) => {
            if registry.profiles.contains_key(&args.name) {
                bail!("profile {} already exists", args.name);
            }
            let source = match args.source {
                Some(path) => path,
                None => home()?.join(match args.from {
                    ImportSource::Tvc => ".config/turnkey/tvc.config.toml",
                    ImportSource::ExperimentalTk => ".config/turnkey/tk/tk.toml",
                }),
            };
            let text = fs::read_to_string(&source)
                .with_context(|| format!("read import source {}", source.display()))?;
            let profile = match args.from {
                ImportSource::Tvc => {
                    let config = Config::from_toml(&text).map_err(|_| {
                        InvalidAuthInput(format!(
                            "invalid TVC import configuration {}",
                            source.display()
                        ))
                    })?;
                    let org = match args.source_profile {
                        Some(name) => config
                            .orgs
                            .get(&name)
                            .context("source profile does not exist")?,
                        None if config.orgs.len() == 1 => {
                            config.orgs.values().next().context("no source profile")?
                        }
                        None => bail!(
                            "select --source-profile when importing a registry with multiple organizations"
                        ),
                    };
                    read_key(&org.api_key_path)?;
                    Profile {
                        organization_id: Uuid::parse_str(&org.id)?,
                        api_base_url: org.api_base_url.clone(),
                        api_key_file: fs::canonicalize(&org.api_key_path)?,
                        ssh_signing_key_id: None,
                    }
                }
                ImportSource::ExperimentalTk => {
                    if args.source_profile.is_some() {
                        bail!("--source-profile is only supported for TVC imports");
                    }
                    #[derive(Deserialize)]
                    struct Legacy {
                        turnkey: LegacyIdentity,
                    }
                    #[derive(Deserialize)]
                    #[serde(rename_all = "camelCase")]
                    struct LegacyIdentity {
                        organization_id: Uuid,
                        api_public_key: String,
                        api_private_key: String,
                        #[serde(default)]
                        api_base_url: Option<String>,
                        private_key_id: Option<String>,
                    }
                    let legacy: Legacy = toml::from_str(&text)
                        .map_err(|_| anyhow::anyhow!("invalid experimental tk configuration"))?;
                    let key = legacy.turnkey;
                    parse_key(&key.api_private_key, &key.api_public_key)?;
                    let api_base_url =
                        parse_endpoint(key.api_base_url.unwrap_or_else(|| DEFAULT_URL.into()))?;
                    let directory = path
                        .parent()
                        .context("registry needs a parent directory")?
                        .join("credentials");
                    fs::create_dir_all(&directory)?;
                    let key_path = directory.join(format!("{}.json", Uuid::new_v4()));
                    let stored = StoredApiKey {
                        public_key: key.api_public_key,
                        private_key: key.api_private_key,
                        curve: KeyCurve::P256,
                    };
                    secure_create(&key_path, &serde_json::to_vec(&stored)?)?;
                    Profile {
                        organization_id: key.organization_id,
                        api_base_url,
                        api_key_file: fs::canonicalize(key_path)?,
                        ssh_signing_key_id: key.private_key_id,
                    }
                }
            };
            registry.profiles.insert(args.name.clone(), profile);
            save(&path, &registry)?;
            Ok(result(
                "profile.import",
                json!({"name": args.name, "selected": false}),
            ))
        }
    }
}
