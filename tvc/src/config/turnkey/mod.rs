//! Turnkey CLI configuration management.
//!
//! Config files are stored at `~/.config/turnkey/`:
//! - `tvc.config.toml` - Main config with org registry, active org, and key paths
//! - `orgs/<org-id>/api_key.json` - Default location for API keys
//! - `orgs/<org-id>/operator.json` - Default location for operator keys
//!
//! Key paths are stored in the config so users can customize storage locations.
//! Directories keyed by the legacy `orgs/<alias>/` layout remain readable —
//! paths are data, not schema — and interactive login migrates them to the
//! id-keyed layout.

mod api_key;
mod qos_operator_key;

pub use api_key::{KeyCurve, StoredApiKey};
pub use qos_operator_key::StoredQosOperatorKey;

use anyhow::{Context, Result, bail};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::debug;
use uuid::Uuid;

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
    /// Map of org alias -> org config, in config-file order. The order is
    /// preserved so saves don't churn the file and so "first profile in the
    /// config" is well-defined for default-alias repair.
    #[serde(default)]
    pub orgs: IndexMap<String, OrgConfig>,
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
    use indexmap::IndexMap;
    use serde::Serialize;
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
        let mut config = DiskConfig::from_toml(content)?.into_current()?;
        // Like the v0 migration, repairs are lazy: the next save persists them.
        config.normalize_default_aliases();
        Ok(config)
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
            .collect::<Result<IndexMap<_, _>>>()?;

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
    /// Human-readable name within the organization.
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
    pub operator_id: Uuid,
    pub wallet_id: Uuid,
    pub path: String,
    pub encrypt_public_key: String,
    pub sign_public_key: String,
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Configuration for a single organization.
///
/// A profile's alias is the key it is registered under in [`Config::orgs`];
/// it is deliberately not repeated here so the name cannot diverge from the
/// registry. Lookups that need both return `(alias, config)` pairs — see
/// [`Config::matching_profiles`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OrgConfig {
    /// The Turnkey organization ID
    pub id: Uuid,
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
    /// Marks the profile that org-ID resolution follows while several profiles
    /// share this organization ID ("default_alias", not "default": profiles are
    /// commonly literally aliased `default`). Only meaningful — and only
    /// serialized — while such duplicates exist.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default_alias: bool,
    /// Unrecognized organization fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

impl OrgConfig {
    /// Return the sole active local operator registry entry.
    pub(crate) fn select_local_operator(&self, org_alias: &str) -> Result<&OperatorRecord> {
        if self.default_operator_kind != OperatorKind::Local {
            bail!(
                "the active operator kind for org '{org_alias}' is {}",
                self.default_operator_kind
            )
        }

        let candidates: Vec<_> = self
            .operators
            .iter()
            .filter(|operator| matches!(operator.kind, OperatorRecordKind::Local(_)))
            .collect();

        // TODO: Decouple this function from its org_alias callsite so it is more
        // flexible to be used anywhere else.
        match candidates.as_slice() {
            [] => bail!("No local operator configured for org '{org_alias}'"),
            [operator] => Ok(*operator),
            _ => bail!("Multiple local operators are configured for org '{org_alias}'"),
        }
    }

    /// Return the kind-specific record for the sole active local operator.
    pub fn select_local_record(&self, org_alias: &str) -> Result<&LocalOperatorRecord> {
        let operator = self.select_local_operator(org_alias)?;
        let OperatorRecordKind::Local(local) = &operator.kind else {
            bail!("selected operator is not local");
        };
        Ok(local)
    }

    /// Whether this profile's key files sit exactly in `dir` under the
    /// default file names — i.e. `dir` is a default-layout directory the
    /// profile owns. A profile with no local operators only needs its API key
    /// there.
    pub(crate) fn has_default_layout_at(&self, dir: &Path) -> bool {
        let operator_key = dir.join(OPERATOR_KEY_FILE);

        self.api_key_path == dir.join(API_KEY_FILE)
            && self
                .operators
                .iter()
                .filter_map(|operator| match &operator.kind {
                    OperatorRecordKind::Local(local) => Some(&local.key_path),
                    _ => None,
                })
                .all(|key_path| *key_path == operator_key)
    }
}

fn default_api_base_url() -> String {
    API_BASE_URL_PROD.to_string()
}

/// A user-supplied organization reference, as taken by `--org` flags.
///
/// Organization IDs are UUIDs, so anything that parses as one is an ID and
/// everything else is a profile alias; parsing never fails.
#[derive(Debug, Clone)]
pub enum OrgQuery {
    Id(Uuid),
    Alias(String),
}

impl FromStr for OrgQuery {
    type Err = Infallible;

    fn from_str(query: &str) -> Result<Self, Infallible> {
        Ok(Uuid::parse_str(query)
            .map(Self::Id)
            .unwrap_or_else(|_| Self::Alias(query.to_string())))
    }
}

impl Display for OrgQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(id) => id.fmt(f),
            Self::Alias(alias) => alias.fmt(f),
        }
    }
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

/// Returns the default directory for a specific org: `~/.config/turnkey/orgs/<org-id>/`
pub fn default_org_dir(org_id: Uuid) -> Result<PathBuf> {
    Ok(orgs_dir()?.join(org_id.to_string()))
}

/// Returns the default API key path for an org
pub fn default_api_key_path(org_id: Uuid) -> Result<PathBuf> {
    Ok(default_org_dir(org_id)?.join(API_KEY_FILE))
}

/// Returns the default operator key path for an org
pub fn default_operator_key_path(org_id: Uuid) -> Result<PathBuf> {
    Ok(default_org_dir(org_id)?.join(OPERATOR_KEY_FILE))
}

/// The legacy default directory for a profile: `~/.config/turnkey/orgs/<alias>/`.
/// Never used for new profiles; still recognized so deletion cleans it up and
/// interactive login can migrate it to the id-keyed layout.
pub(crate) fn legacy_org_dir(alias: &str) -> Result<PathBuf> {
    Ok(orgs_dir()?.join(alias))
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

    /// Every configured profile matching an org query, in config order.
    ///
    /// An alias query names at most one profile; an organization-ID query
    /// returns every profile registered for that organization.
    pub fn matching_profiles(&self, query: &OrgQuery) -> Vec<(&str, &OrgConfig)> {
        match query {
            OrgQuery::Alias(alias) => self
                .orgs
                .get_key_value(alias)
                .map(|(alias, org)| (alias.as_str(), org))
                .into_iter()
                .collect(),
            OrgQuery::Id(id) => self
                .orgs
                .iter()
                .filter(|(_, org)| org.id == *id)
                .map(|(alias, org)| (alias.as_str(), org))
                .collect(),
        }
    }

    /// Organization IDs registered under more than one profile, with the
    /// aliases that share them. Groups are sorted by organization ID; the
    /// aliases within a group keep their config order.
    pub fn duplicated_org_ids(&self) -> Vec<(Uuid, Vec<String>)> {
        self.orgs
            .iter()
            .fold(
                BTreeMap::<Uuid, Vec<&str>>::new(),
                |mut groups, (alias, org)| {
                    groups.entry(org.id).or_default().push(alias);
                    groups
                },
            )
            .into_iter()
            .filter(|(_, aliases)| aliases.len() > 1)
            .map(|(id, aliases)| (id, aliases.into_iter().map(String::from).collect()))
            .collect()
    }

    /// Profiles whose key files sit exactly in the legacy alias-keyed default
    /// layout, in config order — the candidates interactive login migrates to
    /// the id-keyed layout. A profile whose alias spells its own organization
    /// ID (the two layouts coincide) is not a candidate.
    pub(crate) fn legacy_layout_profiles(&self) -> Result<Vec<(String, Uuid)>> {
        self.orgs
            .iter()
            .map(|(alias, org)| {
                let legacy = legacy_org_dir(alias)?;
                let migrates =
                    legacy != default_org_dir(org.id)? && org.has_default_layout_at(&legacy);
                Ok(migrates.then(|| (alias.clone(), org.id)))
            })
            .filter_map(Result::transpose)
            .collect()
    }

    /// Make `winner` the only profile marked `default_alias` among those
    /// registered for `org_id`.
    pub(crate) fn mark_sole_default_alias(&mut self, org_id: Uuid, winner: &str) {
        self.orgs
            .iter_mut()
            .filter(|(_, org)| org.id == org_id)
            .for_each(|(alias, org)| org.default_alias = alias == winner);
    }

    /// Repair the `default_alias` markers so every duplicated organization ID
    /// has exactly one: the first marked profile in config order wins, or the
    /// first profile outright when none is marked. Runs on every load, so the
    /// rest of the CLI can rely on the marker existing (and staying stable)
    /// whenever duplicates do.
    fn normalize_default_aliases(&mut self) {
        let winners: Vec<(Uuid, String)> = self
            .duplicated_org_ids()
            .into_iter()
            .filter_map(|(org_id, aliases)| {
                let winner = aliases
                    .iter()
                    .find(|alias| self.orgs.get(*alias).is_some_and(|org| org.default_alias))
                    .or(aliases.first())?;
                Some((org_id, winner.clone()))
            })
            .collect();

        winners
            .into_iter()
            .for_each(|(org_id, winner)| self.mark_sole_default_alias(org_id, &winner));
    }

    /// Add or update an organization with default key paths.
    ///
    /// Callers are responsible for ensuring the alias and organization ID are
    /// not already registered under another profile; the login command
    /// enforces one profile per organization before constructing new-org
    /// inputs. An existing entry under the same alias is replaced wholesale.
    pub fn add_org(&mut self, alias: &str, org_id: Uuid, api_base_url: String) -> Result<()> {
        debug!(org_alias = alias, %api_base_url, "adding organization config");
        let org_config = OrgConfig {
            id: org_id,
            api_key_path: default_api_key_path(org_id)?,
            api_base_url,
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(default_operator_key_path(org_id)?)],
            default_alias: false,
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
        let removed = self.orgs.shift_remove(alias)?;
        debug!(org_alias = alias, "removing organization config");
        self.last_created_app_id.remove(alias);
        self.last_operator_ids.remove(alias);

        if self.active_org.as_deref() == Some(alias) {
            self.active_org = None;
        }

        // The default-alias marker only means something while several profiles
        // share an organization ID; clear it when the removal dissolves the
        // removed profile's group to a single survivor.
        let mut survivors: Vec<_> = self
            .orgs
            .values_mut()
            .filter(|org| org.id == removed.id)
            .collect();

        if let [survivor] = survivors.as_mut_slice() {
            survivor.default_alias = false;
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
id = "11111111-1111-4111-8111-111111111111"
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
id = "11111111-1111-4111-8111-111111111111"
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

    const ORG_1: Uuid = Uuid::from_u128(1);
    const ORG_2: Uuid = Uuid::from_u128(2);
    const ORG_3: Uuid = Uuid::from_u128(3);

    fn config_with_org_entries<'a>(
        entries: impl IntoIterator<Item = (&'a str, Uuid, bool)>,
    ) -> Config {
        Config {
            orgs: entries
                .into_iter()
                .map(|(alias, org_id, default_alias)| {
                    (
                        alias.to_string(),
                        OrgConfig {
                            id: org_id,
                            api_key_path: PathBuf::from("api_key.json"),
                            api_base_url: API_BASE_URL_PROD.to_string(),
                            default_operator_kind: OperatorKind::Local,
                            operators: Vec::new(),
                            default_alias,
                            extra: toml::Table::new(),
                        },
                    )
                })
                .collect(),
            ..Config::default()
        }
    }

    #[test]
    fn duplicated_org_ids_ignores_unique_ids() {
        let config = config_with_org_entries([("a", ORG_1, false), ("b", ORG_2, false)]);

        assert_eq!(config.duplicated_org_ids(), Vec::new());
    }

    #[test]
    fn duplicated_org_ids_groups_in_config_order() {
        let config = config_with_org_entries([
            ("c", ORG_2, false),
            ("b", ORG_1, false),
            ("a", ORG_2, false),
            ("e", ORG_1, false),
            ("d", ORG_3, false),
        ]);

        assert_eq!(
            config.duplicated_org_ids(),
            vec![
                (ORG_1, vec!["b".to_string(), "e".to_string()]),
                (ORG_2, vec!["c".to_string(), "a".to_string()]),
            ]
        );
    }

    #[test]
    fn matching_profiles_prefers_alias_over_id_lookup() {
        let config = config_with_org_entries([("a", ORG_1, false), ("b", ORG_1, false)]);

        let by_alias = config.matching_profiles(&OrgQuery::Alias("b".to_string()));
        assert_eq!(by_alias.len(), 1);
        assert_eq!(by_alias[0].0, "b");

        let by_id: Vec<&str> = config
            .matching_profiles(&OrgQuery::Id(ORG_1))
            .into_iter()
            .map(|(alias, _)| alias)
            .collect();
        assert_eq!(by_id, ["a", "b"]);

        assert!(
            config
                .matching_profiles(&OrgQuery::Alias("missing".to_string()))
                .is_empty()
        );
    }

    #[test]
    fn normalize_marks_first_profile_when_none_is_marked() {
        let config = load_normalized(
            r#"
version = 1

[orgs.beta]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"

[orgs.alpha]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"
"#,
        );

        // "beta" wins because it comes first in document order, not "alpha"
        // by name order.
        assert!(config.orgs["beta"].default_alias);
        assert!(!config.orgs["alpha"].default_alias);
    }

    #[test]
    fn normalize_keeps_first_marked_profile_and_clears_the_rest() {
        let config = load_normalized(
            r#"
version = 1

[orgs.a]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"

[orgs.b]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"
default_alias = true

[orgs.c]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"
default_alias = true
"#,
        );

        assert!(!config.orgs["a"].default_alias);
        assert!(config.orgs["b"].default_alias);
        assert!(!config.orgs["c"].default_alias);
    }

    #[test]
    fn normalize_leaves_unique_org_ids_unmarked() {
        let config = load_normalized(
            r#"
version = 1

[orgs.solo]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"
"#,
        );

        assert!(!config.orgs["solo"].default_alias);
    }

    fn load_normalized(content: &str) -> Config {
        disk::from_toml(content).expect("fixture config must parse")
    }

    #[test]
    fn has_default_layout_at_requires_both_default_file_names() {
        let dir = Path::new("/keys/org");
        let org = OrgConfig {
            id: ORG_1,
            api_key_path: dir.join("api_key.json"),
            api_base_url: API_BASE_URL_PROD.to_string(),
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(dir.join("operator.json"))],
            default_alias: false,
            extra: toml::Table::new(),
        };

        assert!(org.has_default_layout_at(dir));
        assert!(!org.has_default_layout_at(Path::new("/keys/other")));

        let custom_operator = OrgConfig {
            operators: vec![OperatorRecord::local(PathBuf::from(
                "/elsewhere/operator.json",
            ))],
            ..org.clone()
        };
        assert!(!custom_operator.has_default_layout_at(dir));

        let custom_api_key = OrgConfig {
            api_key_path: PathBuf::from("/elsewhere/api_key.json"),
            ..org
        };
        assert!(!custom_api_key.has_default_layout_at(dir));
    }

    #[test]
    fn remove_org_clears_default_alias_on_sole_survivor() {
        let mut config = config_with_org_entries([("a", ORG_1, true), ("b", ORG_1, false)]);

        config.remove_org("b").expect("b is configured");

        assert!(!config.orgs["a"].default_alias);
    }

    #[test]
    fn remove_org_keeps_default_alias_while_duplicates_remain() {
        let mut config =
            config_with_org_entries([("a", ORG_1, true), ("b", ORG_1, false), ("c", ORG_1, false)]);

        config.remove_org("c").expect("c is configured");

        assert!(config.orgs["a"].default_alias);
    }
}
