use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

const DEFAULT_API_BASE_URL: &str = "https://api.turnkey.com";
const CONFIG_PATH_ENV: &str = "TURNKEY_AUTH_CONFIG_PATH";

const ORGANIZATION_ID_ENV: &str = "TURNKEY_ORGANIZATION_ID";
const API_PUBLIC_KEY_ENV: &str = "TURNKEY_API_PUBLIC_KEY";
const API_PRIVATE_KEY_ENV: &str = "TURNKEY_API_PRIVATE_KEY";
const PRIVATE_KEY_ID_ENV: &str = "TURNKEY_PRIVATE_KEY_ID";
const API_BASE_URL_ENV: &str = "TURNKEY_API_BASE_URL";
const REDACTED_VALUE: &str = "<redacted>";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Fully resolved Turnkey auth configuration.
pub struct Config {
    /// Turnkey organization identifier.
    pub organization_id: String,
    /// Turnkey API public key used for request stamping.
    pub api_public_key: String,
    /// Turnkey API private key used for request stamping.
    pub api_private_key: String,
    /// Turnkey Ed25519 private key identifier.
    pub private_key_id: String,
    /// Base URL for the Turnkey API.
    pub api_base_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Partially resolved auth configuration with missing required fields preserved as `None`.
pub struct ResolvedConfig {
    /// Resolved Turnkey organization identifier, if present.
    pub organization_id: Option<String>,
    /// Resolved Turnkey API public key, if present.
    pub api_public_key: Option<String>,
    /// Resolved Turnkey API private key, if present.
    pub api_private_key: Option<String>,
    /// Resolved Turnkey private key identifier, if present.
    pub private_key_id: Option<String>,
    /// Resolved API base URL, including defaulting.
    pub api_base_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Supported config keys exposed through the CLI.
pub enum ConfigKey {
    /// `turnkey.organizationId`
    OrganizationId,
    /// `turnkey.apiPublicKey`
    ApiPublicKey,
    /// `turnkey.apiPrivateKey`
    ApiPrivateKey,
    /// `turnkey.privateKeyId`
    PrivateKeyId,
    /// `turnkey.apiBaseUrl`
    ApiBaseUrl,
}

impl Config {
    /// Resolves the complete auth configuration from the current process environment and config file.
    pub async fn resolve() -> Result<Self> {
        ResolvedConfig::resolve()
            .await
            .and_then(ResolvedConfig::into_complete)
    }

    /// Resolves the complete auth configuration from an explicit config path and environment map.
    pub fn resolve_from_map(path: &Path, env: &BTreeMap<String, String>) -> Result<Self> {
        ResolvedConfig::resolve_from_map(path, env).and_then(ResolvedConfig::into_complete)
    }
}

impl ResolvedConfig {
    /// Resolves the effective auth configuration, preserving unset required values as `None`.
    pub async fn resolve() -> Result<Self> {
        let path = global_config_path()?;
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        Self::resolve_from_map_async(&path, &env).await
    }

    /// Resolves the effective auth configuration from an explicit config path and environment map.
    pub fn resolve_from_map(path: &Path, env: &BTreeMap<String, String>) -> Result<Self> {
        let persisted = load_persisted_config_sync(path)?;
        Ok(Self {
            organization_id: resolve_value(
                env,
                ORGANIZATION_ID_ENV,
                persisted.turnkey.organization_id.as_deref(),
            ),
            api_public_key: resolve_value(
                env,
                API_PUBLIC_KEY_ENV,
                persisted.turnkey.api_public_key.as_deref(),
            ),
            api_private_key: resolve_value(
                env,
                API_PRIVATE_KEY_ENV,
                persisted.turnkey.api_private_key.as_deref(),
            ),
            private_key_id: resolve_value(
                env,
                PRIVATE_KEY_ID_ENV,
                persisted.turnkey.private_key_id.as_deref(),
            ),
            api_base_url: resolve_value(
                env,
                API_BASE_URL_ENV,
                persisted.turnkey.api_base_url.as_deref(),
            )
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string()),
        })
    }

    /// Resolves the effective auth configuration from an explicit config path and environment map.
    pub async fn resolve_from_map_async(
        path: &Path,
        env: &BTreeMap<String, String>,
    ) -> Result<Self> {
        let persisted = load_persisted_config(path).await?;
        Ok(Self {
            organization_id: resolve_value(
                env,
                ORGANIZATION_ID_ENV,
                persisted.turnkey.organization_id.as_deref(),
            ),
            api_public_key: resolve_value(
                env,
                API_PUBLIC_KEY_ENV,
                persisted.turnkey.api_public_key.as_deref(),
            ),
            api_private_key: resolve_value(
                env,
                API_PRIVATE_KEY_ENV,
                persisted.turnkey.api_private_key.as_deref(),
            ),
            private_key_id: resolve_value(
                env,
                PRIVATE_KEY_ID_ENV,
                persisted.turnkey.private_key_id.as_deref(),
            ),
            api_base_url: resolve_value(
                env,
                API_BASE_URL_ENV,
                persisted.turnkey.api_base_url.as_deref(),
            )
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string()),
        })
    }

    /// Returns the effective value for a specific config key.
    pub fn get(&self, key: ConfigKey) -> Option<&str> {
        match key {
            ConfigKey::OrganizationId => self.organization_id.as_deref(),
            ConfigKey::ApiPublicKey => self.api_public_key.as_deref(),
            ConfigKey::ApiPrivateKey => self.api_private_key.as_deref(),
            ConfigKey::PrivateKeyId => self.private_key_id.as_deref(),
            ConfigKey::ApiBaseUrl => Some(&self.api_base_url),
        }
    }

    /// Serializes the effective config as JSON with sensitive values redacted.
    pub fn render_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&DisplayConfigFile {
            turnkey: DisplayTurnkeyConfig {
                organization_id: self.organization_id.clone().unwrap_or_default(),
                api_public_key: self.api_public_key.clone().unwrap_or_default(),
                api_private_key: redact_if_present(self.api_private_key.as_deref()),
                private_key_id: self.private_key_id.clone().unwrap_or_default(),
                api_base_url: self.api_base_url.clone(),
            },
        })
        .context("failed to render resolved config")
    }

    fn into_complete(self) -> Result<Config> {
        Ok(Config {
            organization_id: required_value("turnkey.organizationId", self.organization_id)?,
            api_public_key: required_value("turnkey.apiPublicKey", self.api_public_key)?,
            api_private_key: required_value("turnkey.apiPrivateKey", self.api_private_key)?,
            private_key_id: required_value("turnkey.privateKeyId", self.private_key_id)?,
            api_base_url: self.api_base_url,
        })
    }
}

impl ConfigKey {
    /// Parses a dotted config key name accepted by the CLI.
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "turnkey.organizationId" => Ok(Self::OrganizationId),
            "turnkey.apiPublicKey" => Ok(Self::ApiPublicKey),
            "turnkey.apiPrivateKey" => Ok(Self::ApiPrivateKey),
            "turnkey.privateKeyId" => Ok(Self::PrivateKeyId),
            "turnkey.apiBaseUrl" => Ok(Self::ApiBaseUrl),
            _ => Err(anyhow!("unsupported config key: {value}")),
        }
    }
}

/// Returns the global auth config path, honoring `TURNKEY_AUTH_CONFIG_PATH` when set.
pub fn global_config_path() -> Result<PathBuf> {
    if let Some(path) = read_value_from_process_env(CONFIG_PATH_ENV) {
        return Ok(PathBuf::from(path));
    }

    let home = read_value_from_process_env("HOME")
        .ok_or_else(|| anyhow!("missing HOME; set {CONFIG_PATH_ENV} to choose a config path"))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("turnkey")
        .join("auth.toml"))
}

/// Returns one resolved config value, redacting the private key when requested.
pub async fn get_resolved_config_value(key: ConfigKey) -> Result<String> {
    let config = ResolvedConfig::resolve().await?;
    let value = config
        .get(key)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("config value is not set"))?;

    Ok(match key {
        ConfigKey::ApiPrivateKey => REDACTED_VALUE.to_string(),
        _ => value.to_owned(),
    })
}

/// Renders the effective config as redacted JSON.
pub async fn render_config() -> Result<String> {
    ResolvedConfig::resolve().await?.render_json()
}

/// Persists one config value to the global config file.
pub async fn set_config_value(key: ConfigKey, value: &str) -> Result<()> {
    let path = global_config_path()?;
    let mut persisted = load_persisted_config(&path).await?;
    persisted.turnkey.set(key, value.to_string());
    save_persisted_config(&path, &persisted).await
}

async fn load_persisted_config(path: &Path) -> Result<PersistedConfigFile> {
    match tokio::fs::read_to_string(path).await {
        Ok(contents) => toml::from_str(&contents).context("failed to parse config file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(PersistedConfigFile::default())
        }
        Err(error) => Err(error.into()),
    }
}

fn load_persisted_config_sync(path: &Path) -> Result<PersistedConfigFile> {
    match std::fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).context("failed to parse config file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(PersistedConfigFile::default())
        }
        Err(error) => Err(error.into()),
    }
}

async fn save_persisted_config(path: &Path, config: &PersistedConfigFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let serialized = toml::to_string_pretty(config).context("failed to serialize config file")?;
    tokio::fs::write(path, serialized).await?;
    Ok(())
}

fn read_value(env: &BTreeMap<String, String>, key: &str) -> Option<String> {
    env.get(key).and_then(|value| normalize_value(value))
}

fn resolve_value(
    env: &BTreeMap<String, String>,
    env_key: &str,
    persisted: Option<&str>,
) -> Option<String> {
    read_value(env, env_key).or_else(|| persisted.and_then(normalize_value))
}

fn read_value_from_process_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .and_then(|value| normalize_value(&value))
}

fn normalize_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn required_value(name: &str, value: Option<String>) -> Result<String> {
    value.ok_or_else(|| anyhow!("missing required config value: {name}"))
}

fn redact_if_present(value: Option<&str>) -> String {
    match value {
        Some(value) if !value.is_empty() => REDACTED_VALUE.to_string(),
        _ => String::new(),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedConfigFile {
    #[serde(default)]
    turnkey: PersistedTurnkeyConfig,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedTurnkeyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    organization_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    api_public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    api_private_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    private_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    api_base_url: Option<String>,
}

impl PersistedTurnkeyConfig {
    fn set(&mut self, key: ConfigKey, value: String) {
        match key {
            ConfigKey::OrganizationId => self.organization_id = Some(value),
            ConfigKey::ApiPublicKey => self.api_public_key = Some(value),
            ConfigKey::ApiPrivateKey => self.api_private_key = Some(value),
            ConfigKey::PrivateKeyId => self.private_key_id = Some(value),
            ConfigKey::ApiBaseUrl => self.api_base_url = Some(value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DisplayConfigFile {
    turnkey: DisplayTurnkeyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DisplayTurnkeyConfig {
    organization_id: String,
    api_public_key: String,
    api_private_key: String,
    private_key_id: String,
    api_base_url: String,
}
