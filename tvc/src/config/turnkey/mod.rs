//! Turnkey CLI configuration management.
//!
//! Config files are stored at `~/.config/turnkey/`:
//! - `tvc.config.toml` - Main config with org registry, active org, and key paths
//! - `orgs/<alias>/api_key.json` - Default location for API keys
//! - `orgs/<alias>/operator.json` - Default location for operator keys
//!
//! Key paths are stored in the config so users can customize storage locations.

mod api_key;
mod qos_operator_key;

pub use api_key::{KeyCurve, StoredApiKey};
pub use qos_operator_key::StoredQosOperatorKey;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::path::PathBuf;
use tracing::debug;

const CONFIG_DIR: &str = ".config/turnkey";
const CONFIG_FILE: &str = "tvc.config.toml";
const ORGS_DIR: &str = "orgs";
const API_KEY_FILE: &str = "api_key.json";
const OPERATOR_KEY_FILE: &str = "operator.json";
const CONFIG_VERSION: u16 = 1;
const DEFAULT_OPERATOR_NAME: &str = "default";

/// Current in-memory TVC configuration.
///
/// Disk schemas are versioned separately below. Loading a legacy v0 config
/// converts it to this model without writing it back; the next existing save
/// point persists it as v1.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Config {
    /// The currently active organization alias
    #[serde(default)]
    pub active_org: Option<String>,
    /// Map of org alias -> org config
    #[serde(default)]
    pub orgs: HashMap<String, OrgConfig>,
    /// Map of org alias -> last created app ID (for convenience)
    #[serde(default)]
    pub last_created_app_id: HashMap<String, String>,
    /// Map of org alias -> last manifest set operator IDs (for convenience)
    #[serde(default)]
    pub last_operator_ids: HashMap<String, Vec<String>>,
    /// Unrecognized top-level fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Versioned on-disk schemas and migrations into the current runtime model.
mod disk {
    use super::{CONFIG_VERSION, Config, OperatorKind, OperatorRecord, OrgConfig};
    use anyhow::{Context, Result, bail};
    use serde::Serialize;
    use std::collections::HashMap;
    use std::path::PathBuf;

    /// Every supported shape of `tvc.config.toml`.
    enum DiskConfig {
        /// The legacy, unversioned config schema.
        V0(v0::Config),
        /// The current config schema, identified by `version = 1` on disk.
        V1(Config),
    }

    impl DiskConfig {
        /// Parse the version header once, then deserialize the remaining table
        /// into the matching supported schema.
        fn from_toml(content: &str) -> Result<Self> {
            let mut table: toml::Table =
                toml::from_str(content).context("failed to parse config TOML")?;
            let header = ConfigVersionHeader::take_from(&mut table)?;
            let config = toml::Value::Table(table);

            match header.version {
                None => config
                    .try_into()
                    .map(Self::V0)
                    .context("failed to parse v0 config"),
                Some(CONFIG_VERSION) => {
                    let config = config.try_into().context("failed to parse v1 config")?;
                    Ok(Self::V1(config))
                }
                Some(version) if version > CONFIG_VERSION => bail!(
                    "config written by a newer tvc (version {version}); this tvc supports through version {CONFIG_VERSION}"
                ),
                Some(version) => bail!("unsupported tvc config version {version}"),
            }
        }

        /// Convert any supported disk schema into the current runtime model.
        fn into_current(self) -> Result<Config> {
            match self {
                Self::V0(config) => migrate_v0(config),
                Self::V1(config) => Ok(config),
            }
        }
    }

    pub(super) fn from_toml(content: &str) -> Result<Config> {
        DiskConfig::from_toml(content)?.into_current()
    }

    /// The version marker is removed before deserializing a schema payload so
    /// it cannot be captured as an unknown field in `Config::extra`.
    struct ConfigVersionHeader {
        version: Option<u16>,
    }

    impl ConfigVersionHeader {
        fn take_from(table: &mut toml::Table) -> Result<Self> {
            let Some(value) = table.remove("version") else {
                return Ok(Self { version: None });
            };
            let version = value
                .try_into()
                .context("config version must be an unsigned 16-bit integer")?;
            Ok(Self {
                version: Some(version),
            })
        }
    }

    /// Serialization-only v1 envelope. Its private construction guarantees
    /// that every saved current config is labeled with the current version.
    #[derive(Serialize)]
    struct V1Envelope<'a> {
        version: u16,
        #[serde(flatten)]
        config: &'a Config,
    }

    pub(super) fn to_toml(config: &Config) -> Result<String> {
        toml::to_string_pretty(&V1Envelope {
            version: CONFIG_VERSION,
            config,
        })
        .context("failed to serialize config")
    }

    fn migrate_v0(config: v0::Config) -> Result<Config> {
        let orgs = config
            .orgs
            .into_iter()
            .map(|(alias, org)| migrate_v0_org(&alias, org).map(|org| (alias, org)))
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Config {
            active_org: config.active_org,
            orgs,
            last_created_app_id: config.last_created_app_id,
            last_operator_ids: config.last_operator_ids,
            extra: config.extra,
        })
    }

    fn migrate_v0_org(alias: &str, mut table: toml::Table) -> Result<OrgConfig> {
        let operator_key_path = table.remove("operator_key_path").with_context(|| {
            format!("v0 config for organization '{alias}' is missing operator_key_path")
        })?;
        let operator_key_path: PathBuf = operator_key_path.try_into().with_context(|| {
            format!("invalid operator_key_path in v0 config for organization '{alias}'")
        })?;
        let mut org: OrgConfig = toml::Value::Table(table)
            .try_into()
            .with_context(|| format!("failed to migrate v0 config for organization '{alias}'"))?;

        org.default_operator_kind = OperatorKind::Local;
        org.operators = vec![OperatorRecord::local(operator_key_path)];
        Ok(org)
    }

    pub(super) mod v0 {
        use serde::Deserialize;
        use std::collections::HashMap;

        /// Legacy top-level schema. Organization tables remain untyped until
        /// migration extracts `operator_key_path` and parses the current shape.
        #[derive(Deserialize)]
        pub(super) struct Config {
            #[serde(default)]
            pub(super) active_org: Option<String>,
            #[serde(default)]
            pub(super) orgs: HashMap<String, toml::Table>,
            #[serde(default)]
            pub(super) last_created_app_id: HashMap<String, String>,
            #[serde(default)]
            pub(super) last_operator_ids: HashMap<String, Vec<String>>,
            /// Unknown root values are carried into the current config so a
            /// migration does not discard data owned by another writer.
            #[serde(default, flatten)]
            pub(super) extra: toml::Table,
        }
    }
}

/// Known API base URLs for different environments.
pub const API_BASE_URL_PROD: &str = "https://api.turnkey.com";
pub const API_BASE_URL_PREPROD: &str = "https://api.preprod.turnkey.engineering";
pub const API_BASE_URL_DEV: &str = "https://api.dev.turnkey.engineering";
pub const API_BASE_URL_LOCAL: &str = "http://localhost:8081";

/// Dashboard base URLs corresponding to each environment.
pub const DASHBOARD_URL_PROD: &str = "https://app.turnkey.com";
pub const DASHBOARD_URL_PREPROD: &str = "https://app.preprod.turnkey.engineering";
pub const DASHBOARD_URL_DEV: &str = "https://app.dev.turnkey.engineering";

/// Maps an API base URL to the dashboard base URL for the same environment.
///
/// Any URL that isn't a known Turnkey environment (e.g. a local API) falls back
/// to the production dashboard.
pub fn dashboard_base_url(api_base_url: &str) -> &'static str {
    match api_base_url {
        API_BASE_URL_PROD => DASHBOARD_URL_PROD,
        API_BASE_URL_PREPROD => DASHBOARD_URL_PREPROD,
        API_BASE_URL_DEV => DASHBOARD_URL_DEV,
        _ => DASHBOARD_URL_PROD,
    }
}

/// The active operator backend for an organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OperatorKind {
    #[default]
    Local,
    Hosted,
}

impl Display for OperatorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => "local".fmt(f),
            Self::Hosted => "hosted".fmt(f),
        }
    }
}

/// One durable operator entry in an organization's registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OperatorRecord {
    /// Unique human-readable name within the organization.
    pub name: String,
    #[serde(flatten)]
    pub kind: OperatorRecordKind,
}

impl OperatorRecord {
    pub fn local(key_path: PathBuf) -> Self {
        Self {
            name: DEFAULT_OPERATOR_NAME.to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path,
                operator_id: None,
                extra: toml::Table::new(),
            }),
        }
    }

    pub fn operator_kind(&self) -> OperatorKind {
        match self.kind {
            OperatorRecordKind::Local(_) => OperatorKind::Local,
            OperatorRecordKind::Hosted(_) => OperatorKind::Hosted,
        }
    }
}

/// Kind-specific durable operator metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperatorRecordKind {
    Local(LocalOperatorRecord),
    Hosted(HostedOperatorRecord),
}

/// Locator and optional Turnkey identity for a local operator key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LocalOperatorRecord {
    /// Path to a `StoredQosOperatorKey` JSON document.
    pub key_path: PathBuf,
    #[serde(default)]
    pub operator_id: Option<String>,
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Public metadata for an operator whose keys are held by Turnkey.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct HostedOperatorRecord {
    pub operator_id: String,
    pub wallet_id: String,
    pub path: String,
    pub encrypt_public_key: String,
    pub sign_public_key: String,
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Configuration for a single organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OrgConfig {
    /// The Turnkey organization ID
    pub id: String,
    /// Path to the API key file
    pub api_key_path: PathBuf,
    /// API base URL for this organization
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,
    /// Operator backend selected for default command resolution.
    #[serde(default)]
    pub default_operator_kind: OperatorKind,
    /// Durable local and hosted operator metadata.
    #[serde(default)]
    pub operators: Vec<OperatorRecord>,
    /// Unrecognized organization fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

impl OrgConfig {
    /// Return the sole active local operator record.
    pub fn select_local_record(&self, org_alias: &str) -> Result<&LocalOperatorRecord> {
        if self.default_operator_kind != OperatorKind::Local {
            bail!(
                "the active operator kind for org '{org_alias}' is {}",
                self.default_operator_kind
            )
        }

        let candidates: Vec<_> = self
            .operators
            .iter()
            .filter_map(|operator| match &operator.kind {
                OperatorRecordKind::Local(local) => Some(local),
                _ => None,
            })
            .collect();

        // TODO: Decouple this function from its org_alias callsite so it is more
        // flexible to be used anywhere else.
        match candidates.as_slice() {
            [] => bail!("No local operator configured for org '{org_alias}'"),
            [operator] => Ok(*operator),
            _ => bail!("Multiple local operators are configured for org '{org_alias}'"),
        }
    }
}

fn default_api_base_url() -> String {
    API_BASE_URL_PROD.to_string()
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
        debug!(config_path = %path.display(), "loading tvc config");
        if !path.exists() {
            debug!(config_path = %path.display(), "tvc config not found; using defaults");
            return Ok(Config::default());
        }

        let config = Self::load_from_path(&path).await?;

        debug!(
            config_path = %path.display(),
            active_org = ?config.active_org,
            org_count = config.orgs.len(),
            "loaded tvc config"
        );

        Ok(config)
    }

    /// Save config to disk
    pub async fn save(&self) -> Result<()> {
        let path = config_file_path()?;
        debug!(
            config_path = %path.display(),
            active_org = ?self.active_org,
            org_count = self.orgs.len(),
            "saving tvc config"
        );

        self.save_to_path(&path).await
    }

    async fn load_from_path(path: &Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        disk::from_toml(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))
    }

    async fn save_to_path(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!("failed to create config directory: {}", parent.display())
            })?;
        }

        let content = disk::to_toml(self)?;

        tokio::fs::write(&path, content)
            .await
            .with_context(|| format!("failed to write config file: {}", path.display()))?;

        debug!(config_path = %path.display(), "saved tvc config");

        Ok(())
    }

    /// Get the active organization config, if any
    pub fn active_org_config(&self) -> Option<(&String, &OrgConfig)> {
        let alias = self.active_org.as_ref()?;
        self.orgs.get(alias).map(|config| (alias, config))
    }

    /// Add or update an organization with default key paths
    pub fn add_org(&mut self, alias: &str, org_id: String, api_base_url: String) -> Result<()> {
        debug!(org_alias = alias, %api_base_url, "adding organization config");
        let org_config = OrgConfig {
            id: org_id,
            api_key_path: default_api_key_path(alias)?,
            api_base_url,
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(default_operator_key_path(alias)?)],
            extra: toml::Table::new(),
        };
        self.orgs.insert(alias.to_string(), org_config);
        Ok(())
    }

    /// Remove an organization from the config, along with the convenience state
    /// (last app ID, last operator IDs) tracked for it. If the removed org was
    /// the active one, the active org is cleared.
    ///
    /// Returns the removed [`OrgConfig`], or `None` if no org with that alias
    /// was configured. This only touches the config registry; deleting the
    /// org's key files on disk is the caller's responsibility.
    pub fn remove_org(&mut self, alias: &str) -> Option<OrgConfig> {
        let removed = self.orgs.remove(alias)?;
        debug!(org_alias = alias, "removing organization config");
        self.last_created_app_id.remove(alias);
        self.last_operator_ids.remove(alias);
        if self.active_org.as_deref() == Some(alias) {
            self.active_org = None;
        }
        Some(removed)
    }

    /// Set the active organization
    pub fn set_active_org(&mut self, alias: &str) -> Result<()> {
        debug!(org_alias = alias, "setting active organization");
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

    /// Store the last created app ID for the active org
    pub fn set_last_app_id(&mut self, app_id: &str) -> Result<()> {
        let alias = self
            .active_org
            .as_ref()
            .context("no active organization set")?;
        self.last_created_app_id
            .insert(alias.clone(), app_id.to_string());
        Ok(())
    }

    /// Get the last created app ID for the active org, if any
    pub fn get_last_app_id(&self) -> Option<String> {
        let alias = self.active_org.as_ref()?;
        self.last_created_app_id.get(alias).cloned()
    }

    /// Store the last manifest set operator IDs for the active org
    pub fn set_last_operator_ids(&mut self, operator_ids: &[String]) -> Result<()> {
        let alias = self
            .active_org
            .as_ref()
            .context("no active organization set")?;
        self.last_operator_ids
            .insert(alias.clone(), operator_ids.to_vec());
        Ok(())
    }

    /// Get the last manifest set operator IDs for the active org
    pub fn get_last_operator_ids(&self) -> Option<Vec<String>> {
        let alias = self.active_org.as_ref()?;
        self.last_operator_ids.get(alias).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const V0_CONFIG: &str = r#"
active_org = "default"
future_root = "keep-root"

[orgs.default]
id = "org-123"
api_key_path = "/keys/api.json"
operator_key_path = "/keys/operator.json"
future_org = 42

[last_created_app_id]
default = "app-123"

[last_operator_ids]
default = ["operator-123"]
"#;

    const MIGRATED_V1_CONFIG: &str = r#"
version = 1
active_org = "default"
future_root = "keep-root"

[orgs.default]
id = "org-123"
api_key_path = "/keys/api.json"
api_base_url = "https://api.turnkey.com"
default_operator_kind = "local"
future_org = 42

[[orgs.default.operators]]
name = "default"
kind = "local"
key_path = "/keys/operator.json"

[last_created_app_id]
default = "app-123"

[last_operator_ids]
default = ["operator-123"]
"#;

    #[tokio::test]
    async fn migrates_v0_in_memory_and_writes_v1_lazily() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(&path, V0_CONFIG).await.unwrap();

        let config = Config::load_from_path(&path).await.unwrap();
        let original = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(original, V0_CONFIG);

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Local);
        assert_eq!(org.operators.len(), 1);
        assert_eq!(org.operators[0].name, DEFAULT_OPERATOR_NAME);
        let OperatorRecordKind::Local(local) = &org.operators[0].kind else {
            panic!("migrated operator must be local");
        };
        assert_eq!(local.key_path, PathBuf::from("/keys/operator.json"));

        config.save_to_path(&path).await.unwrap();
        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        let actual = disk::from_toml(&saved).unwrap();
        let expected = disk::from_toml(MIGRATED_V1_CONFIG).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn rejects_malformed_version() {
        let error = disk::from_toml("version = \"one\"").expect_err("malformed version must fail");
        assert!(
            error
                .to_string()
                .contains("config version must be an unsigned 16-bit integer")
        );
    }
}
