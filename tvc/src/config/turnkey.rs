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
mod yubikey;

pub use api_key::{KeyCurve, StoredApiKey};
pub use qos_operator_key::{
    QosOperatorPublicKey, QosOperatorPublicKeyParseError, StoredQosOperatorKey,
};
pub use yubikey::{
    Registration, YubiKeyOperatorRecord, YubiKeyRegistry, YubiKeyRegistryEntry, YubiKeySerial,
    YubiKeySerialParseError,
};

use anyhow::{Context, Result, bail};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::convert::Infallible;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

pub const CONFIG_DIR: &str = ".config/turnkey";
pub const CONFIG_FILE: &str = "tvc.config.toml";
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
    /// config" is well-defined.
    #[serde(default)]
    pub orgs: IndexMap<String, OrgConfig>,
    /// Registered YubiKey devices, shared across organizations. Absent from
    /// disk entirely while empty, so configs predating the registry rewrite
    /// byte-identically.
    #[serde(default, skip_serializing_if = "YubiKeyRegistry::is_empty")]
    pub yubikeys: YubiKeyRegistry,
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
    use super::{
        CONFIG_VERSION, Config, OperatorKind, OperatorRecord, OperatorRecordKind, OrgConfig,
        YubiKeyRegistry,
    };
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
            let config = match self {
                Self::V0(config) => migrate_v0(config)?,
                Self::V1(config) => config,
            };

            // An organization operator may only reference a registered
            // device; rejecting a dangling serial here keeps everything
            // downstream of the parse free of that state.
            let dangling = config.orgs.iter().find_map(|(alias, org)| {
                org.operators
                    .iter()
                    .find_map(|operator| match &operator.kind {
                        OperatorRecordKind::Yubikey(record)
                            if !config.yubikeys.contains(record.serial) =>
                        {
                            Some((alias, &operator.name, record.serial))
                        }
                        _ => None,
                    })
            });

            if let Some((alias, name, serial)) = dangling {
                bail!(
                    "organization '{alias}' operator '{name}' references YubiKey {serial}, \
                     which is not in the yubikeys registry; edit tvc.config.toml to add the \
                     matching [[yubikeys]] entry or remove this operator record"
                );
            }

            Ok(config)
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
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(Config {
            active_org: config.active_org,
            orgs,
            yubikeys: YubiKeyRegistry::default(),
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
    Yubikey,
}

impl Display for OperatorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => "local".fmt(f),
            Self::Hosted => "hosted".fmt(f),
            Self::Yubikey => "yubikey".fmt(f),
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

    /// A serial-only YubiKey operator record, named `yubikey-{serial}` so
    /// several can coexist unambiguously within one organization.
    pub fn yubikey(serial: YubiKeySerial) -> Self {
        Self {
            name: format!("yubikey-{serial}"),
            kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                serial,
                extra: toml::Table::new(),
            }),
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
    Yubikey(YubiKeyOperatorRecord),
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
    /// Unrecognized organization fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Failure modes of selecting the sole local operator of an organization.
/// Which organization it was is the caller's context to add.
#[derive(Debug, Error)]
pub enum SelectLocalOperatorError {
    #[error("no local operator is configured")]
    NoLocalOperator,
    #[error("multiple local operators are configured")]
    MultipleLocalOperators,
}

/// Failure modes of selecting the sole hosted operator of an organization.
/// Which organization it was is the caller's context to add.
#[derive(Debug, Error)]
pub enum SelectHostedOperatorError {
    #[error("no hosted operator is configured")]
    NoHostedOperator,
    #[error("multiple hosted operators are configured")]
    MultipleHostedOperators,
}

/// The operator backend a newly added organization starts with.
pub enum NewOrgOperator {
    /// A local key file at the alias's default path, generated on first
    /// login.
    LocalKeyFile,
    /// A registered YubiKey, referenced by serial only; the organization
    /// defaults to the yubikey backend.
    Yubikey(YubiKeySerial),
}

/// The serial is already referenced by one of the organization's operators.
/// Which organization it was is the caller's context to add.
#[derive(Debug, Error)]
#[error("YubiKey {serial} is already an operator of this organization")]
pub struct DuplicateYubiKeyOperator {
    pub serial: YubiKeySerial,
}

/// Failure modes of selecting an organization's YubiKey operator.
/// Which organization it was is the caller's context to add.
#[derive(Debug, Error)]
pub enum SelectYubiKeyOperatorError {
    #[error("no YubiKey operator is configured")]
    NoYubiKeyOperator,
    #[error(
        "multiple YubiKey operators are configured (serials {})",
        serials.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
    )]
    MultipleYubiKeyOperators { serials: Vec<YubiKeySerial> },
    #[error("no YubiKey operator has serial {serial}")]
    SerialNotAnOperator { serial: YubiKeySerial },
}

impl OrgConfig {
    /// Select the sole local operator registry entry, with its kind-specific
    /// record. Purely a registry query: whether local is the organization's
    /// default backend is resolution policy, decided elsewhere.
    pub(crate) fn select_local_operator(
        &self,
    ) -> Result<(&OperatorRecord, &LocalOperatorRecord), SelectLocalOperatorError> {
        let mut locals = self
            .operators
            .iter()
            .filter_map(|operator| match &operator.kind {
                OperatorRecordKind::Local(local) => Some((operator, local)),
                OperatorRecordKind::Hosted(_) | OperatorRecordKind::Yubikey(_) => None,
            });

        match (locals.next(), locals.next()) {
            (Some(sole), None) => Ok(sole),
            (None, _) => Err(SelectLocalOperatorError::NoLocalOperator),
            (Some(_), Some(_)) => Err(SelectLocalOperatorError::MultipleLocalOperators),
        }
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

    /// The hosted operator registry entries, each with its kind-specific
    /// record, in config order.
    pub(crate) fn hosted_operators(&self) -> impl Iterator<Item = (&str, &HostedOperatorRecord)> {
        self.operators
            .iter()
            .filter_map(|operator| match &operator.kind {
                OperatorRecordKind::Hosted(hosted) => Some((operator.name.as_str(), hosted)),
                OperatorRecordKind::Local(_) | OperatorRecordKind::Yubikey(_) => None,
            })
    }

    /// Select the sole hosted operator registry entry, with its kind-specific
    /// record. Purely a registry query: whether hosted is the organization's
    /// default backend is resolution policy, decided elsewhere.
    pub(crate) fn select_hosted_operator(
        &self,
    ) -> Result<(&str, &HostedOperatorRecord), SelectHostedOperatorError> {
        let mut hosted = self.hosted_operators();

        match (hosted.next(), hosted.next()) {
            (Some(sole), None) => Ok(sole),
            (None, _) => Err(SelectHostedOperatorError::NoHostedOperator),
            (Some(_), Some(_)) => Err(SelectHostedOperatorError::MultipleHostedOperators),
        }
    }

    /// The YubiKey operator registry entries, each with its kind-specific
    /// record, in config order.
    pub(crate) fn yubikey_operators(
        &self,
    ) -> impl Iterator<Item = (&OperatorRecord, &YubiKeyOperatorRecord)> {
        self.operators
            .iter()
            .filter_map(|operator| match &operator.kind {
                OperatorRecordKind::Yubikey(yubikey) => Some((operator, yubikey)),
                OperatorRecordKind::Local(_) | OperatorRecordKind::Hosted(_) => None,
            })
    }

    /// Select the organization's YubiKey operator registry entry, with its
    /// kind-specific record: the one with the given serial, or the sole one
    /// when no serial narrows it. Purely a registry query: whether YubiKey
    /// is the organization's default backend — and how an ambiguous
    /// selection is settled — is resolution policy, decided elsewhere.
    pub(crate) fn select_yubikey_operator(
        &self,
        serial: Option<YubiKeySerial>,
    ) -> Result<(&OperatorRecord, &YubiKeyOperatorRecord), SelectYubiKeyOperatorError> {
        let mut yubikeys = self.yubikey_operators();

        match serial {
            Some(serial) => yubikeys
                .find(|(_, yubikey)| yubikey.serial == serial)
                .ok_or(SelectYubiKeyOperatorError::SerialNotAnOperator { serial }),
            None => match (yubikeys.next(), yubikeys.next()) {
                (Some(sole), None) => Ok(sole),
                (None, _) => Err(SelectYubiKeyOperatorError::NoYubiKeyOperator),
                (Some(_), Some(_)) => Err(SelectYubiKeyOperatorError::MultipleYubiKeyOperators {
                    serials: self
                        .yubikey_operators()
                        .map(|(_, yubikey)| yubikey.serial)
                        .collect(),
                }),
            },
        }
    }

    /// Produce a YubiKey operator record for the serial, named
    /// `yubikey-{serial}` unless a name is given. Refuses a serial this
    /// organization already references, so callers can settle the config
    /// mutation before performing device I/O.
    pub(crate) fn new_yubikey_operator(
        &self,
        serial: YubiKeySerial,
        name: Option<String>,
    ) -> Result<OperatorRecord, DuplicateYubiKeyOperator> {
        if self
            .yubikey_operators()
            .any(|(_, yubikey)| yubikey.serial == serial)
        {
            return Err(DuplicateYubiKeyOperator { serial });
        }

        let mut record = OperatorRecord::yubikey(serial);

        if let Some(name) = name {
            record.name = name;
        }

        Ok(record)
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
    pub fn from_toml(content: &str) -> Result<Self> {
        disk::from_toml(content)
    }

    /// Save config to disk
    pub async fn save(&self) -> Result<()> {
        // Owns its whole failure surface: callers add what only they know
        // (e.g. recovery guidance), never the file identity.
        let path = config_file_path().context("failed to resolve the tvc.config.toml path")?;
        debug!(
            config_path = %path.display(),
            active_org = ?self.active_org,
            org_count = self.orgs.len(),
            "saving tvc config"
        );

        self.save_to_path(&path).await
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

    /// Add or update an organization with default key paths, starting with
    /// the given operator backend as its default.
    ///
    /// Callers are responsible for ensuring the alias and organization ID are
    /// not already registered under another profile; the login command
    /// enforces one profile per organization before constructing new-org
    /// inputs. An existing entry under the same alias is replaced wholesale.
    pub fn add_org(
        &mut self,
        alias: &str,
        org_id: Uuid,
        api_base_url: String,
        operator: NewOrgOperator,
    ) -> Result<()> {
        debug!(org_alias = alias, %api_base_url, "adding organization config");

        let (default_operator_kind, operators) = match operator {
            NewOrgOperator::LocalKeyFile => (
                OperatorKind::Local,
                vec![OperatorRecord::local(default_operator_key_path(org_id)?)],
            ),
            NewOrgOperator::Yubikey(serial) => {
                (OperatorKind::Yubikey, vec![OperatorRecord::yubikey(serial)])
            }
        };

        let org_config = OrgConfig {
            id: org_id,
            api_key_path: default_api_key_path(org_id)?,
            api_base_url,
            default_operator_kind,
            operators,
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

        let content = tokio::fs::read_to_string(&path).await.unwrap();
        let config = Config::from_toml(&content).unwrap();
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

    /// Serials are deliberately non-canonical (uppercase, unpadded) to pin
    /// the parse-then-canonicalize behavior on save.
    fn v1_yubikey_config() -> String {
        format!(
            r#"
version = 1
active_org = "default"

[[yubikeys]]
serial = "1C95C1F"
public_key = "{}"
future_entry = "keep"

[orgs.default]
id = "11111111-1111-4111-8111-111111111111"
api_key_path = "/keys/api.json"
default_operator_kind = "yubikey"

[[orgs.default.operators]]
name = "yubikey"
kind = "yubikey"
serial = "1C95C1F"
"#,
            "07".repeat(130)
        )
    }

    #[test]
    fn round_trips_yubikey_registry_and_operator_records() {
        let config = disk::from_toml(&v1_yubikey_config()).unwrap();

        let serial = YubiKeySerial::from(0x01c9_5c1f);
        let entry = config.yubikeys.get(serial).unwrap();
        assert_eq!(entry.public_key.to_string(), "07".repeat(130));

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Yubikey);
        let OperatorRecordKind::Yubikey(record) = &org.operators[0].kind else {
            panic!("operator must be a yubikey record");
        };
        assert_eq!(record.serial, serial);

        let saved = disk::to_toml(&config).unwrap();
        assert!(saved.contains("serial = \"01c95c1f\""));
        assert!(!saved.contains("1C95C1F"));
        assert!(saved.contains("kind = \"yubikey\""));
        assert!(saved.contains("default_operator_kind = \"yubikey\""));
        assert!(saved.contains("future_entry = \"keep\""));
        assert_eq!(disk::from_toml(&saved).unwrap(), config);
    }

    #[test]
    fn empty_registry_stays_off_disk() {
        let config = disk::from_toml(MIGRATED_V1_CONFIG).unwrap();

        assert!(!disk::to_toml(&config).unwrap().contains("yubikeys"));
    }

    #[test]
    fn rejects_duplicate_registry_serials() {
        let entry = format!(
            "[[yubikeys]]\nserial = \"01c95c1f\"\npublic_key = \"{}\"\n",
            "07".repeat(130)
        );

        let error = disk::from_toml(&format!("version = 1\n{entry}{entry}"))
            .expect_err("duplicate serials must fail");

        assert!(
            format!("{error:#}").contains("duplicate YubiKey registry entry for serial 01c95c1f")
        );
    }

    #[test]
    fn rejects_a_dangling_yubikey_operator_reference() {
        let error = disk::from_toml(
            r#"
version = 1

[orgs.default]
id = "11111111-1111-4111-8111-111111111111"
api_key_path = "/keys/api.json"

[[orgs.default.operators]]
name = "yubikey"
kind = "yubikey"
serial = "01c95c1f"
"#,
        )
        .expect_err("a reference without a registry entry must fail");

        assert_eq!(
            error.to_string(),
            "organization 'default' operator 'yubikey' references YubiKey 01c95c1f, \
             which is not in the yubikeys registry; edit tvc.config.toml to add the matching \
             [[yubikeys]] entry or remove this operator record"
        );
    }
    fn yubikey_org(serials: &[u32]) -> OrgConfig {
        OrgConfig {
            id: Uuid::from_u128(0x123),
            api_key_path: PathBuf::from("/keys/api.json"),
            api_base_url: default_api_base_url(),
            default_operator_kind: OperatorKind::Yubikey,
            operators: serials
                .iter()
                .map(|&serial| OperatorRecord {
                    name: "yubikey".to_string(),
                    kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                        serial: YubiKeySerial::from(serial),
                        extra: toml::Table::new(),
                    }),
                })
                .collect(),
            extra: toml::Table::new(),
        }
    }

    #[test]
    fn selects_the_sole_yubikey_operator_without_a_serial() {
        let org = yubikey_org(&[0x01c9_5c1f]);

        let (_, record) = org.select_yubikey_operator(None).unwrap();

        assert_eq!(record.serial, YubiKeySerial::from(0x01c9_5c1f));
    }

    #[test]
    fn selecting_among_multiple_yubikey_operators_lists_their_serials() {
        let org = yubikey_org(&[0x01c9_5c1f, 0xdead_beef]);

        let error = org.select_yubikey_operator(None).unwrap_err();

        assert_eq!(
            error.to_string(),
            "multiple YubiKey operators are configured (serials 01c95c1f, deadbeef)"
        );
    }

    #[test]
    fn selects_a_yubikey_operator_by_serial() {
        let org = yubikey_org(&[0x01c9_5c1f, 0xdead_beef]);

        let (_, record) = org
            .select_yubikey_operator(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap();

        assert_eq!(record.serial, YubiKeySerial::from(0xdead_beef));
    }

    #[test]
    fn selecting_an_unknown_serial_is_refused() {
        let org = yubikey_org(&[0x01c9_5c1f]);

        let error = org
            .select_yubikey_operator(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap_err();

        assert_eq!(error.to_string(), "no YubiKey operator has serial deadbeef");
    }

    #[test]
    fn yubikey_records_default_to_a_serial_derived_name() {
        let record = OperatorRecord::yubikey(YubiKeySerial::from(0x01c9_5c1f));

        assert_eq!(record.name, "yubikey-01c95c1f");
    }

    #[test]
    fn adding_a_duplicate_yubikey_serial_is_refused() {
        let org = yubikey_org(&[0x01c9_5c1f]);

        let error = org
            .new_yubikey_operator(YubiKeySerial::from(0x01c9_5c1f), None)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "YubiKey 01c95c1f is already an operator of this organization"
        );
        assert_eq!(org.operators.len(), 1);
    }

    #[test]
    fn removing_an_org_keeps_the_yubikey_registry() {
        let mut config = Config::default();
        config.yubikeys.register(
            YubiKeySerial::from(0x01c9_5c1f),
            QosOperatorPublicKey::try_from([7u8; 130].as_slice()).unwrap(),
        );
        config
            .orgs
            .insert("default".to_string(), yubikey_org(&[0x01c9_5c1f]));

        let removed = config.remove_org("default").unwrap();

        assert_eq!(removed.operators.len(), 1);
        assert!(config.orgs.is_empty());
        assert_eq!(config.yubikeys.serials().count(), 1);
    }

    #[test]
    fn distinct_yubikey_serials_coexist_in_one_org() {
        let mut org = yubikey_org(&[0x01c9_5c1f]);

        let record = org
            .new_yubikey_operator(YubiKeySerial::from(0xdead_beef), Some("backup".to_string()))
            .unwrap();
        org.operators.push(record);

        let (record, yubikey) = org
            .select_yubikey_operator(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap();
        assert_eq!(record.name, "backup");
        assert_eq!(yubikey.serial, YubiKeySerial::from(0xdead_beef));
    }

    const ORG_1: Uuid = Uuid::from_u128(1);
    const ORG_2: Uuid = Uuid::from_u128(2);
    const ORG_3: Uuid = Uuid::from_u128(3);

    fn config_with_org_entries<'a>(entries: impl IntoIterator<Item = (&'a str, Uuid)>) -> Config {
        Config {
            orgs: entries
                .into_iter()
                .map(|(alias, org_id)| {
                    (
                        alias.to_string(),
                        OrgConfig {
                            id: org_id,
                            api_key_path: PathBuf::from("api_key.json"),
                            api_base_url: API_BASE_URL_PROD.to_string(),
                            default_operator_kind: OperatorKind::Local,
                            operators: vec![OperatorRecord::local(PathBuf::from("operator.json"))],
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
        let config = config_with_org_entries([("a", ORG_1), ("b", ORG_2)]);

        assert_eq!(config.duplicated_org_ids(), Vec::new());
    }

    #[test]
    fn duplicated_org_ids_groups_in_config_order() {
        let config = config_with_org_entries([
            ("c", ORG_2),
            ("b", ORG_1),
            ("a", ORG_2),
            ("e", ORG_1),
            ("d", ORG_3),
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
    fn has_default_layout_at_requires_both_default_file_names() {
        let dir = Path::new("/keys/org");
        let org = OrgConfig {
            id: ORG_1,
            api_key_path: dir.join("api_key.json"),
            api_base_url: API_BASE_URL_PROD.to_string(),
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(dir.join("operator.json"))],
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
    fn matching_profiles_prefers_alias_over_id_lookup() {
        let config = config_with_org_entries([("a", ORG_1), ("b", ORG_1)]);

        let by_alias = config.matching_profiles(&OrgQuery::Alias("b".to_string()));
        assert_eq!(by_alias.len(), 1);
        assert_eq!(by_alias[0].0, "b");

        let by_id = config
            .matching_profiles(&OrgQuery::Id(ORG_1))
            .into_iter()
            .map(|(alias, _)| alias)
            .collect::<Vec<_>>();
        assert_eq!(by_id, ["a", "b"]);

        assert!(
            config
                .matching_profiles(&OrgQuery::Alias("missing".to_string()))
                .is_empty()
        );
    }

    #[test]
    fn add_org_with_a_yubikey_starts_on_the_yubikey_backend() {
        let mut config = Config::default();
        config
            .add_org(
                "default",
                Uuid::from_u128(0x123),
                API_BASE_URL_PROD.to_string(),
                NewOrgOperator::Yubikey(YubiKeySerial::from(0x01c9_5c1f)),
            )
            .unwrap();

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Yubikey);
        assert_eq!(
            org.operators,
            vec![OperatorRecord::yubikey(YubiKeySerial::from(0x01c9_5c1f))]
        );
    }
}
