use std::collections::BTreeMap;
use std::fs;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub organization_id: String,
    pub api_public_key: String,
    pub api_private_key: String,
    pub private_key_id: String,
    pub api_base_url: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub organization_id: Option<String>,
    pub api_public_key: Option<String>,
    pub api_private_key: Option<String>,
    pub private_key_id: Option<String>,
    pub api_base_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigKey {
    OrganizationId,
    ApiPublicKey,
    ApiPrivateKey,
    PrivateKeyId,
    ApiBaseUrl,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Self::resolve()
    }

    pub fn resolve() -> Result<Self> {
        ResolvedConfig::resolve().and_then(ResolvedConfig::into_complete)
    }

    pub fn resolve_from_map(path: &Path, env: &BTreeMap<String, String>) -> Result<Self> {
        ResolvedConfig::resolve_from_map(path, env).and_then(ResolvedConfig::into_complete)
    }
}

impl ResolvedConfig {
    pub fn resolve() -> Result<Self> {
        let path = global_config_path()?;
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        Self::resolve_from_map(&path, &env)
    }

    pub fn resolve_from_map(path: &Path, env: &BTreeMap<String, String>) -> Result<Self> {
        let persisted = load_persisted_config(path)?;
        Ok(Self {
            organization_id: read_value(env, ORGANIZATION_ID_ENV)
                .or_else(|| persisted.turnkey.organization_id.clone()),
            api_public_key: read_value(env, API_PUBLIC_KEY_ENV)
                .or_else(|| persisted.turnkey.api_public_key.clone()),
            api_private_key: read_value(env, API_PRIVATE_KEY_ENV)
                .or_else(|| persisted.turnkey.api_private_key.clone()),
            private_key_id: read_value(env, PRIVATE_KEY_ID_ENV)
                .or_else(|| persisted.turnkey.private_key_id.clone()),
            api_base_url: read_value(env, API_BASE_URL_ENV)
                .or_else(|| persisted.turnkey.api_base_url.clone())
                .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string()),
        })
    }

    pub fn get(&self, key: ConfigKey) -> Option<&str> {
        match key {
            ConfigKey::OrganizationId => self.organization_id.as_deref(),
            ConfigKey::ApiPublicKey => self.api_public_key.as_deref(),
            ConfigKey::ApiPrivateKey => self.api_private_key.as_deref(),
            ConfigKey::PrivateKeyId => self.private_key_id.as_deref(),
            ConfigKey::ApiBaseUrl => Some(&self.api_base_url),
        }
    }

    pub fn render_toml(&self) -> Result<String> {
        toml::to_string_pretty(&DisplayConfigFile {
            turnkey: DisplayTurnkeyConfig {
                organization_id: self.organization_id.clone().unwrap_or_default(),
                api_public_key: self.api_public_key.clone().unwrap_or_default(),
                api_private_key: self.api_private_key.clone().unwrap_or_default(),
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

pub fn load_resolved_config() -> Result<ResolvedConfig> {
    ResolvedConfig::resolve()
}

pub fn get_config_value(key: ConfigKey) -> Result<String> {
    let config = load_resolved_config()?;
    config
        .get(key)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("config value is not set"))
}

pub fn render_resolved_config() -> Result<String> {
    load_resolved_config()?.render_toml()
}

pub fn set_persisted_config_value(key: ConfigKey, value: &str) -> Result<()> {
    let path = global_config_path()?;
    let mut persisted = load_persisted_config(&path)?;
    persisted.turnkey.set(key, value.to_string());
    save_persisted_config(&path, &persisted)
}

fn load_persisted_config(path: &Path) -> Result<PersistedConfigFile> {
    match fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).context("failed to parse config file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PersistedConfigFile::default()),
        Err(error) => Err(error.into()),
    }
}

fn save_persisted_config(path: &Path, config: &PersistedConfigFile) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let serialized = toml::to_string_pretty(config).context("failed to serialize config file")?;
    fs::write(path, serialized)?;
    Ok(())
}

fn read_value(env: &BTreeMap<String, String>, key: &str) -> Option<String> {
    env.get(key).and_then(|value| normalize_value(value))
}

fn read_value_from_process_env(key: &str) -> Option<String> {
    std::env::var(key).ok().and_then(|value| normalize_value(&value))
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
