//! Turnkey CLI configuration management.
//!
//! Config files are stored at `~/.config/turnkey/`:
//! - `tvc.config.toml` - Main config with org registry, active org, and key paths
//! - `orgs/<alias>/api_key.json` - Default location for API keys
//! - `orgs/<alias>/operator.json` - Default location for operator keys
//!
//! Key paths are stored in the config so users can customize storage locations.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/turnkey";
const CONFIG_FILE: &str = "tvc.config.toml";
const ORGS_DIR: &str = "orgs";
const API_KEY_FILE: &str = "api_key.json";
const OPERATOR_KEY_FILE: &str = "operator.json";

/// Main configuration stored in tvc.config.toml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// The currently active organization alias
    pub active_org: Option<String>,
    /// Map of org alias -> org config
    #[serde(default)]
    pub orgs: HashMap<String, OrgConfig>,
}

/// Configuration for a single organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgConfig {
    /// The Turnkey organization ID
    pub id: String,
    /// Path to the API key file
    pub api_key_path: PathBuf,
    /// Path to the operator key file
    pub operator_key_path: PathBuf,
}

/// API key stored in api_key.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Hex-encoded private key
    pub private_key: String,
}

/// Operator key stored in operator.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorKey {
    /// Hex-encoded compressed public key
    pub public_key: String,
    /// Hex-encoded private key
    pub private_key: String,
}

/// Returns the base config directory: `~/.config/turnkey/`
pub fn config_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    Ok(PathBuf::from(home).join(CONFIG_DIR))
}

/// Returns the path to tvc.config.toml
pub fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir()?.join(CONFIG_FILE))
}

/// Returns the default orgs directory: `~/.config/turnkey/orgs/`
pub fn orgs_dir() -> Result<PathBuf> {
    Ok(config_dir()?.join(ORGS_DIR))
}

/// Returns the default directory for a specific org: `~/.config/turnkey/orgs/<alias>/`
pub fn default_org_dir(alias: &str) -> Result<PathBuf> {
    Ok(orgs_dir()?.join(alias))
}

/// Returns the default API key path for an org
pub fn default_api_key_path(alias: &str) -> Result<PathBuf> {
    Ok(default_org_dir(alias)?.join(API_KEY_FILE))
}

/// Returns the default operator key path for an org
pub fn default_operator_key_path(alias: &str) -> Result<PathBuf> {
    Ok(default_org_dir(alias)?.join(OPERATOR_KEY_FILE))
}

impl Config {
    /// Load config from disk, or return default if it doesn't exist
    pub async fn load() -> Result<Self> {
        let path = config_file_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }

        let content = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;

        Ok(config)
    }

    /// Save config to disk
    pub async fn save(&self) -> Result<()> {
        let path = config_file_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("failed to serialize config")?;

        tokio::fs::write(&path, content)
            .await
            .with_context(|| format!("failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get the active organization config, if any
    pub fn active_org_config(&self) -> Option<(&String, &OrgConfig)> {
        let alias = self.active_org.as_ref()?;
        self.orgs.get(alias).map(|config| (alias, config))
    }

    /// Add or update an organization with default key paths
    pub fn add_org(&mut self, alias: &str, org_id: String) -> Result<()> {
        let org_config = OrgConfig {
            id: org_id,
            api_key_path: default_api_key_path(alias)?,
            operator_key_path: default_operator_key_path(alias)?,
        };
        self.orgs.insert(alias.to_string(), org_config);
        Ok(())
    }

    /// Set the active organization
    pub fn set_active_org(&mut self, alias: &str) -> Result<()> {
        if !self.orgs.contains_key(alias) {
            bail!("organization '{}' not found in config", alias);
        }
        self.active_org = Some(alias.to_string());
        Ok(())
    }

    /// Get list of configured org aliases
    pub fn org_aliases(&self) -> Vec<&String> {
        self.orgs.keys().collect()
    }
}

impl ApiKey {
    /// Load API key from the path specified in org config
    pub async fn load(org_config: &OrgConfig) -> Result<Option<Self>> {
        let path = &org_config.api_key_path;
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read API key: {}", path.display()))?;

        let key: ApiKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse API key: {}", path.display()))?;

        Ok(Some(key))
    }

    /// Save API key to the path specified in org config
    pub async fn save(&self, org_config: &OrgConfig) -> Result<()> {
        let path = &org_config.api_key_path;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(self).context("failed to serialize API key")?;

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write API key: {}", path.display()))?;

        Ok(())
    }
}

impl OperatorKey {
    /// Load operator key from the path specified in org config
    pub async fn load(org_config: &OrgConfig) -> Result<Option<Self>> {
        let path = &org_config.operator_key_path;
        if !path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read operator key: {}", path.display()))?;

        let key: OperatorKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse operator key: {}", path.display()))?;

        Ok(Some(key))
    }

    /// Save operator key to the path specified in org config
    pub async fn save(&self, org_config: &OrgConfig) -> Result<()> {
        let path = &org_config.operator_key_path;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(self).context("failed to serialize operator key")?;

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("failed to write operator key: {}", path.display()))?;

        Ok(())
    }
}
