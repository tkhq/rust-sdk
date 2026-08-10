//! Turnkey CLI configuration management.
//!
//! Config files are stored at `~/.config/turnkey/`:
//! - `tvc.config.toml` - Main config: org registry keyed by organization ID,
//!   the alias map, active org, and key paths
//! - `orgs/<org-id>/api_key.json` - Default location for API keys
//! - `orgs/<org-id>/operator.json` - Default location for operator keys
//!
//! Aliases are strictly an edge concept: the serialized `[aliases]` table maps
//! human names to organization UUIDs, and nothing else in the config is keyed
//! by name. Key paths are stored in the config so users can customize storage
//! locations; directories keyed by the legacy `orgs/<alias>/` layout remain
//! readable and are migrated by interactive login.
//!
//! Older config schemas (v0, v1) are migrated eagerly at load behind a
//! `tvc.config.toml.backup` fence; see [`Config::load`].

mod api_key;
mod qos_operator_key;

pub use api_key::{KeyCurve, StoredApiKey};
pub use qos_operator_key::{QosOperatorPublicKey, StoredQosOperatorKey};

use anyhow::{Context, Result, bail};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::HashMap,
    convert::Infallible,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::debug;
use uuid::Uuid;

const CONFIG_DIR: &str = ".config/turnkey";
const CONFIG_FILE: &str = "tvc.config.toml";
const ORGS_DIR: &str = "orgs";
const API_KEY_FILE: &str = "api_key.json";
const OPERATOR_KEY_FILE: &str = "operator.json";
const CONFIG_VERSION: u16 = 2;
const DEFAULT_OPERATOR_NAME: &str = "default";

/// Current in-memory TVC configuration (the v2 disk schema).
///
/// Older disk schemas are migrated eagerly at load: the original file is
/// renamed to `tvc.config.toml.backup`, the migrated config is written, and
/// only then is the backup removed, so a crash at any point leaves either the
/// old file or the backup intact.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Config {
    /// The currently active organization.
    #[serde(default)]
    pub active_org: Option<Uuid>,
    /// Org registry keyed by organization ID, in config-file order.
    #[serde(default)]
    pub orgs: IndexMap<Uuid, OrgConfig>,
    /// Edge-side names for organizations.
    #[serde(default)]
    pub aliases: Aliases,
    /// Last created app ID per organization (for convenience).
    #[serde(default)]
    pub last_created_app_id: HashMap<Uuid, Uuid>,
    /// Last manifest set operator IDs per organization (for convenience).
    #[serde(default)]
    pub last_operator_ids: HashMap<Uuid, Vec<Uuid>>,
    /// Unrecognized top-level fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Edge-side names for UUID-identified resources.
///
/// The one serialized alias surface: a name maps to exactly one ID by map
/// construction, and every mutation goes through these methods. Nothing in
/// here is org-specific, so future resource aliases (apps, deployments) can
/// reuse the shape unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Aliases(IndexMap<String, Uuid>);

impl Aliases {
    /// Resolve a name, remembering it for output echoing.
    pub fn resolve(&self, name: &str) -> Option<Resolved<'_>> {
        self.0.get_key_value(name).map(|(name, id)| Resolved {
            name: Some(name.as_str()),
            id: *id,
        })
    }

    /// Names bound to `id`, in config-file order.
    pub fn names_of(&self, id: Uuid) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter(move |(_, bound)| **bound == id)
            .map(|(name, _)| name.as_str())
    }

    /// Bind `name` to `id`, returning the ID it previously pointed at.
    pub fn bind(&mut self, name: String, id: Uuid) -> Option<Uuid> {
        self.0.insert(name, id)
    }

    /// Drop every name bound to `id`, returning the removed names in
    /// config-file order.
    pub fn unbind_all(&mut self, id: Uuid) -> Vec<String> {
        let removed = self.names_of(id).map(str::to_string).collect();
        self.0.retain(|_, bound| *bound != id);
        removed
    }

    /// All bindings, in config-file order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, Uuid)> {
        self.0.iter().map(|(name, id)| (name.as_str(), *id))
    }
}

/// A resolved org reference that remembers how the user named it, so output
/// echoes the input: alias in -> alias out, UUID in -> UUID out.
///
/// Derefs to the [`Uuid`], so it drops into every ID-keyed lookup unchanged.
#[derive(Debug, Clone, Copy)]
pub struct Resolved<'a> {
    name: Option<&'a str>,
    id: Uuid,
}

impl Resolved<'_> {
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// The name the user typed, when they typed one.
    pub fn name(&self) -> Option<&str> {
        self.name
    }
}

impl From<Uuid> for Resolved<'static> {
    fn from(id: Uuid) -> Self {
        Self { name: None, id }
    }
}

impl std::ops::Deref for Resolved<'_> {
    type Target = Uuid;

    fn deref(&self) -> &Uuid {
        &self.id
    }
}

impl Borrow<Uuid> for Resolved<'_> {
    fn borrow(&self) -> &Uuid {
        &self.id
    }
}

impl Display for Resolved<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.name {
            Some(name) => name.fmt(f),
            None => self.id.fmt(f),
        }
    }
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

/// Versioned on-disk schemas and migrations into the current runtime model.
mod disk {
    use super::{Aliases, CONFIG_VERSION, Config, OperatorKind, OperatorRecord, OrgConfig};
    use anyhow::{Context, Result, bail};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use uuid::Uuid;

    /// A parsed config plus how it got here, so the load path knows whether
    /// an eager write-back (with its backup fence) is required.
    pub(super) enum Loaded {
        Current(Config),
        Migrated {
            config: Config,
            /// Human-facing notes about lossy migration decisions
            /// (e.g. merged duplicate profiles).
            notes: Vec<String>,
        },
    }

    /// Every supported shape of `tvc.config.toml`.
    enum DiskConfig {
        /// The legacy, unversioned config schema.
        V0(v0::Config),
        /// The alias-keyed schema, identified by `version = 1` on disk.
        V1(v1::Config),
        /// The current schema, identified by `version = 2` on disk.
        V2(Config),
    }

    impl DiskConfig {
        /// Parse the version header once, then deserialize the remaining table
        /// into the matching supported schema.
        fn from_toml(content: &str) -> Result<Self> {
            let mut table: toml::Table =
                toml::from_str(content).context("failed to parse config TOML")?;

            // The version marker is removed before deserializing a schema
            // payload so it cannot be captured as an unknown field in
            // `Config::extra`.
            let version = table
                .remove("version")
                .map(|value| {
                    u16::deserialize(value)
                        .context("config version must be an unsigned 16-bit integer")
                })
                .transpose()?;

            let config = toml::Value::Table(table);

            match version {
                None => config
                    .try_into()
                    .map(Self::V0)
                    .context("failed to parse v0 config"),
                Some(1) => config
                    .try_into()
                    .map(Self::V1)
                    .context("failed to parse v1 config"),
                Some(CONFIG_VERSION) => {
                    let config = config.try_into().context("failed to parse v2 config")?;
                    Ok(Self::V2(config))
                }
                Some(version) if version > CONFIG_VERSION => bail!(
                    "config written by a newer tvc (version {version}); this tvc supports through version {CONFIG_VERSION}"
                ),
                Some(version) => bail!("unsupported tvc config version {version}"),
            }
        }
    }

    pub(super) fn from_toml(content: &str) -> Result<Loaded> {
        match DiskConfig::from_toml(content)? {
            DiskConfig::V0(config) => Ok(migrate_v1(migrate_v0(config)?)),
            DiskConfig::V1(config) => Ok(migrate_v1(config)),
            DiskConfig::V2(config) => Ok(Loaded::Current(config)),
        }
    }

    /// Serialization-only envelope. Its private construction guarantees that
    /// every saved config is labeled with the current version.
    #[derive(Serialize)]
    struct V2Envelope<'a> {
        version: u16,
        #[serde(flatten)]
        config: &'a Config,
    }

    pub(super) fn to_toml(config: &Config) -> Result<String> {
        toml::to_string_pretty(&V2Envelope {
            version: CONFIG_VERSION,
            config,
        })
        .context("failed to serialize config")
    }

    /// Rekey a v1 config: aliases move to the alias map, orgs and the side
    /// maps key by organization ID. A v1 organization registered under
    /// several aliases collapses to one entry — the `default_alias`-marked
    /// profile's settings win, else the first in file order — and every alias
    /// survives as a name for it; dropped profiles are reported in the notes
    /// (their key files are left on disk untouched).
    fn migrate_v1(config: v1::Config) -> Loaded {
        let entries: Vec<(String, v1::OrgConfig)> = config.orgs.into_iter().collect();

        // Choose one surviving profile per organization: the first
        // `default_alias`-marked one in file order, else the first outright.
        let mut winner_of: IndexMap<Uuid, usize> = IndexMap::new();
        let mut aliases = Aliases::default();

        for (index, (alias, org)) in entries.iter().enumerate() {
            aliases.bind(alias.clone(), org.id);

            match winner_of.entry(org.id) {
                indexmap::map::Entry::Vacant(entry) => {
                    entry.insert(index);
                }
                indexmap::map::Entry::Occupied(mut entry) => {
                    if org.default_alias && !entries[*entry.get()].1.default_alias {
                        entry.insert(index);
                    }
                }
            }
        }

        let mut orgs: IndexMap<Uuid, OrgConfig> = IndexMap::new();
        let mut notes = Vec::new();

        for (index, (alias, org)) in entries.into_iter().enumerate() {
            let v1::OrgConfig {
                id,
                api_key_path,
                api_base_url,
                default_operator_kind,
                operators,
                default_alias: _,
                extra,
            } = org;

            if winner_of.get(&id) == Some(&index) {
                orgs.insert(
                    id,
                    OrgConfig {
                        api_key_path,
                        api_base_url,
                        default_operator_kind,
                        operators,
                        extra,
                    },
                );
            } else {
                // The alias keeps working (it now names the kept profile);
                // only this profile's settings and key-file registration are
                // superseded, and its files stay on disk.
                notes.push(format!(
                    "merged duplicate profile '{alias}' for organization {id}: the alias now \
                     names the kept profile; key files at {} were left on disk",
                    api_key_path.display()
                ));
            }
        }

        let active_org = config
            .active_org
            .and_then(|alias| aliases.resolve(&alias).map(|resolved| resolved.id()));

        fn rekey<V>(
            map: std::collections::HashMap<String, V>,
            aliases: &Aliases,
        ) -> std::collections::HashMap<Uuid, V> {
            map.into_iter()
                .filter_map(|(alias, value)| {
                    aliases
                        .resolve(&alias)
                        .map(|resolved| (resolved.id(), value))
                })
                .collect()
        }

        Loaded::Migrated {
            config: Config {
                active_org,
                orgs,
                last_created_app_id: rekey(config.last_created_app_id, &aliases),
                last_operator_ids: rekey(config.last_operator_ids, &aliases),
                aliases,
                extra: config.extra,
            },
            notes,
        }
    }

    fn migrate_v0(config: v0::Config) -> Result<v1::Config> {
        let orgs = config
            .orgs
            .into_iter()
            .map(|(alias, org)| migrate_v0_org(&alias, org).map(|org| (alias, org)))
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(v1::Config {
            active_org: config.active_org,
            orgs,
            last_created_app_id: config.last_created_app_id,
            last_operator_ids: config.last_operator_ids,
            extra: config.extra,
        })
    }

    fn migrate_v0_org(alias: &str, mut table: toml::Table) -> Result<v1::OrgConfig> {
        let operator_key_path = table.remove("operator_key_path").with_context(|| {
            format!("v0 config for organization '{alias}' is missing operator_key_path")
        })?;
        let operator_key_path: PathBuf = operator_key_path.try_into().with_context(|| {
            format!("invalid operator_key_path in v0 config for organization '{alias}'")
        })?;
        let mut org: v1::OrgConfig = toml::Value::Table(table)
            .try_into()
            .with_context(|| format!("failed to migrate v0 config for organization '{alias}'"))?;

        org.default_operator_kind = OperatorKind::Local;
        org.operators = vec![OperatorRecord::local(operator_key_path)];
        Ok(org)
    }

    /// The alias-keyed v1 schema (read-only: parsed for migration).
    mod v1 {
        use super::super::{OperatorKind, OperatorRecord};
        use indexmap::IndexMap;
        use serde::Deserialize;
        use std::collections::HashMap;
        use std::path::PathBuf;
        use uuid::Uuid;

        #[derive(Deserialize)]
        pub(super) struct Config {
            #[serde(default)]
            pub(super) active_org: Option<String>,
            #[serde(default)]
            pub(super) orgs: IndexMap<String, OrgConfig>,
            #[serde(default)]
            pub(super) last_created_app_id: HashMap<String, Uuid>,
            #[serde(default)]
            pub(super) last_operator_ids: HashMap<String, Vec<Uuid>>,
            #[serde(default, flatten)]
            pub(super) extra: toml::Table,
        }

        #[derive(Deserialize)]
        pub(super) struct OrgConfig {
            pub(super) id: Uuid,
            pub(super) api_key_path: PathBuf,
            #[serde(default = "super::super::default_api_base_url")]
            pub(super) api_base_url: String,
            #[serde(default)]
            pub(super) default_operator_kind: OperatorKind,
            #[serde(default)]
            pub(super) operators: Vec<OperatorRecord>,
            /// v1's duplicate-profile marker; consumed by the merge and never
            /// carried into v2.
            #[serde(default)]
            pub(super) default_alias: bool,
            #[serde(default, flatten)]
            pub(super) extra: toml::Table,
        }
    }

    pub(super) mod v0 {
        use serde::Deserialize;
        use std::collections::HashMap;
        use uuid::Uuid;

        /// Legacy top-level schema. Organization tables remain untyped until
        /// migration extracts `operator_key_path` and parses the v1 shape.
        #[derive(Deserialize)]
        pub(super) struct Config {
            #[serde(default)]
            pub(super) active_org: Option<String>,
            #[serde(default)]
            pub(super) orgs: HashMap<String, toml::Table>,
            #[serde(default)]
            pub(super) last_created_app_id: HashMap<String, Uuid>,
            #[serde(default)]
            pub(super) last_operator_ids: HashMap<String, Vec<Uuid>>,
            /// Unknown root values are carried forward so a migration does not
            /// discard data owned by another writer.
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

/// Configuration for a single organization, keyed by its organization ID in
/// [`Config::orgs`]; the ID is deliberately not repeated here so identity has
/// exactly one home. Human names live in [`Config::aliases`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OrgConfig {
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
    /// Load config from disk, or return default if it doesn't exist.
    ///
    /// Older schemas are migrated eagerly behind a backup fence: the original
    /// file is renamed to `<name>.backup`, the migrated config is written,
    /// and only after a successful write is the backup removed. An existing
    /// backup file therefore means a previous migration crashed, and loading
    /// refuses to proceed until it is resolved manually.
    pub async fn load() -> Result<Self> {
        Self::load_from_path(&config_file_path()?).await
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
        let backup_path = backup_file_path(path);

        if backup_path.exists() {
            bail!(
                r#"found {backup} from an interrupted config migration.

Compare it with {config} (the backup is the pre-migration version; the config
file may be missing or already migrated), keep the right contents at
{config}, delete {backup}, and re-run."#,
                backup = backup_path.display(),
                config = path.display(),
            );
        }

        debug!(config_path = %path.display(), "loading tvc config");

        if !path.exists() {
            debug!(config_path = %path.display(), "tvc config not found; using defaults");
            return Ok(Config::default());
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let loaded = disk::from_toml(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;

        let config = match loaded {
            disk::Loaded::Current(config) => config,
            disk::Loaded::Migrated { config, notes } => {
                config
                    .persist_migration(path, &backup_path)
                    .await
                    .with_context(|| {
                        format!("failed to migrate config file: {}", path.display())
                    })?;

                // One-time migration event with user-relevant, lossy
                // decisions (merged duplicates, orphaned key files): loading
                // has no shell handle, tracing defaults to off, and the
                // stdout contract stays untouched — so stderr directly is
                // the only channel that reliably reaches the user.
                #[allow(clippy::print_stderr)]
                for note in &notes {
                    eprintln!("config migration: {note}");
                }

                config
            }
        };

        debug!(
            config_path = %path.display(),
            active_org = ?config.active_org,
            org_count = config.orgs.len(),
            "loaded tvc config"
        );

        Ok(config)
    }

    /// The eager-migration backup fence: park the original, write the
    /// migrated config, and only then remove the original. A crash between
    /// any two steps leaves the pre-migration contents recoverable at one of
    /// the two paths, and the backup's presence blocks the next load.
    async fn persist_migration(&self, path: &Path, backup_path: &Path) -> Result<()> {
        tokio::fs::rename(path, backup_path)
            .await
            .with_context(|| format!("failed to back up config to {}", backup_path.display()))?;

        self.save_to_path(path).await?;

        tokio::fs::remove_file(backup_path).await.with_context(|| {
            format!(
                "failed to remove migration backup {}",
                backup_path.display()
            )
        })?;

        debug!(config_path = %path.display(), "eagerly migrated config schema");

        Ok(())
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

    /// Resolve an org query to a configured organization, remembering the
    /// typed name for output echoing. `None` covers an unknown alias, an
    /// unknown ID, and an alias whose organization table is missing.
    pub fn resolve(&self, query: &OrgQuery) -> Option<(Resolved<'_>, &OrgConfig)> {
        let resolved = match query {
            OrgQuery::Alias(name) => self.aliases.resolve(name)?,
            OrgQuery::Id(id) => Resolved::from(*id),
        };

        self.orgs.get(&resolved.id()).map(|org| (resolved, org))
    }

    /// The active organization as a [`Resolved`] reference. The user typed
    /// nothing to echo, so it is named by its first alias when one exists —
    /// the human name — and by its ID otherwise.
    pub fn resolve_active(&self) -> Option<(Resolved<'_>, &OrgConfig)> {
        let (id, org) = self.active_org_config()?;
        let name = self.aliases.names_of(id).next();
        Some((Resolved { name, id }, org))
    }

    /// The name to show for an organization the user did not name themselves:
    /// its first alias, else the bare ID.
    pub fn display_name(&self, id: Uuid) -> String {
        self.aliases
            .names_of(id)
            .next()
            .map(str::to_string)
            .unwrap_or_else(|| id.to_string())
    }

    /// Get the active organization config, if any
    pub fn active_org_config(&self) -> Option<(Uuid, &OrgConfig)> {
        let id = self.active_org?;
        self.orgs.get(&id).map(|config| (id, config))
    }

    /// Register an organization with default key paths. An existing entry for
    /// the same organization is replaced wholesale; names are bound separately
    /// through [`Config::aliases`].
    pub fn add_org(&mut self, org_id: Uuid, api_base_url: String) -> Result<()> {
        debug!(%org_id, %api_base_url, "adding organization config");
        let org_config = OrgConfig {
            api_key_path: default_api_key_path(org_id)?,
            api_base_url,
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(default_operator_key_path(org_id)?)],
            extra: toml::Table::new(),
        };
        self.orgs.insert(org_id, org_config);
        Ok(())
    }

    /// Remove an organization: its registry entry, every alias bound to it,
    /// and the convenience state tracked for it. If it was the active org,
    /// the active org is cleared.
    ///
    /// Returns the removed [`OrgConfig`] and the unbound aliases, or `None`
    /// if the organization is not configured. This only touches the config;
    /// deleting key files on disk is the caller's responsibility.
    pub fn remove_org(&mut self, org_id: Uuid) -> Option<(OrgConfig, Vec<String>)> {
        let removed = self.orgs.shift_remove(&org_id)?;
        debug!(%org_id, "removing organization config");
        let unbound = self.aliases.unbind_all(org_id);
        self.last_created_app_id.remove(&org_id);
        self.last_operator_ids.remove(&org_id);

        if self.active_org == Some(org_id) {
            self.active_org = None;
        }

        Some((removed, unbound))
    }

    /// Set the active organization
    pub fn set_active_org(&mut self, org_id: Uuid) -> Result<()> {
        debug!(%org_id, "setting active organization");
        if !self.orgs.contains_key(&org_id) {
            bail!("organization '{org_id}' not found in config");
        }
        self.active_org = Some(org_id);
        Ok(())
    }

    /// Organizations whose key files sit exactly in the legacy alias-keyed
    /// default layout, as `(alias, org_id)` pairs in config order — the
    /// candidates interactive login migrates to the id-keyed layout. A name
    /// that spells its own organization ID (the two layouts coincide) is not
    /// a candidate.
    pub(crate) fn legacy_layout_profiles(&self) -> Result<Vec<(String, Uuid)>> {
        self.aliases
            .iter()
            .map(|(name, id)| {
                let legacy = legacy_org_dir(name)?;
                let migrates = legacy != default_org_dir(id)?
                    && self
                        .orgs
                        .get(&id)
                        .is_some_and(|org| org.has_default_layout_at(&legacy));
                Ok(migrates.then(|| (name.to_string(), id)))
            })
            .filter_map(Result::transpose)
            .collect()
    }

    /// Store the last created app ID for the active org
    pub fn set_last_app_id(&mut self, app_id: Uuid) -> Result<()> {
        let org_id = self.active_org.context("no active organization set")?;
        self.last_created_app_id.insert(org_id, app_id);
        Ok(())
    }

    /// Get the last created app ID for the active org, if any
    pub fn get_last_app_id(&self) -> Option<Uuid> {
        self.last_created_app_id.get(&self.active_org?).cloned()
    }

    /// Store the last manifest set operator IDs for the active org
    pub fn set_last_operator_ids(&mut self, operator_ids: Vec<Uuid>) -> Result<()> {
        let org_id = self.active_org.context("no active organization set")?;
        self.last_operator_ids.insert(org_id, operator_ids);
        Ok(())
    }

    /// Get the last manifest set operator IDs for the active org
    pub fn get_last_operator_ids(&self) -> Option<&[Uuid]> {
        self.last_operator_ids
            .get(&self.active_org?)
            .map(Vec::as_slice)
    }
}

/// The eager-migration backup path for a config file:
/// `tvc.config.toml.backup` next to `tvc.config.toml`.
fn backup_file_path(config_path: &Path) -> PathBuf {
    let mut backup = config_path.as_os_str().to_owned();
    backup.push(".backup");
    PathBuf::from(backup)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    const ORG_1: Uuid = Uuid::from_u128(1);
    const ORG_2: Uuid = Uuid::from_u128(2);
    const OPERATOR_ID: Uuid = Uuid::from_u128(123);
    const APP_ID: Uuid = Uuid::from_u128(321);

    fn test_org(api_base_url: &str, dir: &Path) -> OrgConfig {
        OrgConfig {
            api_key_path: dir.join("api_key.json"),
            api_base_url: api_base_url.to_string(),
            default_operator_kind: OperatorKind::Local,
            operators: vec![OperatorRecord::local(dir.join("operator.json"))],
            extra: toml::Table::new(),
        }
    }

    #[test]
    fn aliases_resolve_remembers_the_typed_name() {
        let mut aliases = Aliases::default();
        aliases.bind("prod".to_string(), ORG_1);

        let resolved = aliases.resolve("prod").expect("bound name resolves");

        assert_eq!(resolved.id(), ORG_1);
        assert_eq!(resolved.to_string(), "prod");
        assert!(aliases.resolve("staging").is_none());
    }

    #[test]
    fn resolved_from_an_id_displays_the_id() {
        let resolved = Resolved::from(ORG_1);

        assert_eq!(resolved.name(), None);
        assert_eq!(resolved.to_string(), ORG_1.to_string());
    }

    #[test]
    fn names_of_lists_synonyms_in_config_order() {
        let mut aliases = Aliases::default();
        aliases.bind("prod".to_string(), ORG_1);
        aliases.bind("staging".to_string(), ORG_2);
        aliases.bind("production".to_string(), ORG_1);

        let names: Vec<&str> = aliases.names_of(ORG_1).collect();

        assert_eq!(names, ["prod", "production"]);
    }

    #[test]
    fn bind_returns_the_previous_target_and_unbind_all_drops_names() {
        let mut aliases = Aliases::default();
        assert_eq!(aliases.bind("prod".to_string(), ORG_1), None);
        assert_eq!(aliases.bind("prod".to_string(), ORG_2), Some(ORG_1));

        aliases.bind("backup".to_string(), ORG_2);

        assert_eq!(aliases.unbind_all(ORG_2), ["prod", "backup"]);
        assert_eq!(aliases.names_of(ORG_2).count(), 0);
    }

    #[test]
    fn config_resolve_rejects_a_dangling_alias() {
        let mut config = Config::default();
        config.aliases.bind("prod".to_string(), ORG_1);

        // The alias points at an organization with no registry entry.
        let Ok(query) = OrgQuery::from_str("prod");
        assert!(config.resolve(&query).is_none());
    }

    fn v0_config() -> String {
        format!(
            r#"
active_org = "default"
future_root = "keep-root"

[orgs.default]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/api.json"
operator_key_path = "/keys/operator.json"
future_org = 42

[last_created_app_id]
default = "{}"

[last_operator_ids]
default = ["{}"]
"#,
            APP_ID, OPERATOR_ID
        )
    }

    #[tokio::test]
    async fn migrates_v0_to_v2_eagerly_with_backup_lifecycle() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(&path, v0_config()).await.unwrap();

        let config = Config::load_from_path(&path).await.unwrap();

        assert_eq!(config.aliases.resolve("default").unwrap().id(), ORG_1);
        assert_eq!(config.active_org, Some(ORG_1));
        assert_eq!(
            config.orgs[&ORG_1].api_key_path,
            PathBuf::from("/keys/api.json")
        );
        assert_eq!(
            config.orgs[&ORG_1].extra["future_org"],
            toml::Value::Integer(42)
        );
        assert_eq!(
            config.extra["future_root"],
            toml::Value::String("keep-root".into())
        );
        assert_eq!(config.last_created_app_id[&ORG_1], APP_ID);
        assert_eq!(config.last_operator_ids[&ORG_1], vec![OPERATOR_ID]);

        // Eagerly rewritten: the file on disk is v2 and the backup is gone.
        let saved = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(saved.contains("version = 2"), "{saved}");
        assert!(!backup_file_path(&path).exists());

        // A second load takes the already-current path and changes nothing.
        let reloaded = Config::load_from_path(&path).await.unwrap();
        assert_eq!(reloaded, config);
    }

    const V1_DUPLICATES: &str = r#"
version = 1
active_org = "alias-a"

[orgs.alias-a]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/a/api.json"
api_base_url = "https://a.example"

[orgs.alias-b]
id = "00000000-0000-0000-0000-000000000001"
api_key_path = "/keys/b/api.json"
api_base_url = "https://b.example"
default_alias = true
"#;

    #[tokio::test]
    async fn v1_duplicates_merge_to_the_marked_profile_keeping_every_alias() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(&path, V1_DUPLICATES).await.unwrap();

        let config = Config::load_from_path(&path).await.unwrap();

        // The marked profile's settings win; both names survive; the marker
        // itself does not exist in v2.
        assert_eq!(config.orgs.len(), 1);
        assert_eq!(config.orgs[&ORG_1].api_base_url, "https://b.example");
        assert_eq!(config.aliases.resolve("alias-a").unwrap().id(), ORG_1);
        assert_eq!(config.aliases.resolve("alias-b").unwrap().id(), ORG_1);
        assert_eq!(config.active_org, Some(ORG_1));
        assert!(
            !tokio::fs::read_to_string(&path)
                .await
                .unwrap()
                .contains("default_alias")
        );
    }

    #[tokio::test]
    async fn v1_unmarked_duplicates_keep_the_first_profile_in_file_order() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(&path, V1_DUPLICATES.replace("default_alias = true\n", ""))
            .await
            .unwrap();

        let config = Config::load_from_path(&path).await.unwrap();

        assert_eq!(config.orgs[&ORG_1].api_base_url, "https://a.example");
    }

    #[tokio::test]
    async fn an_existing_backup_blocks_loading() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(&path, v0_config()).await.unwrap();
        tokio::fs::write(backup_file_path(&path), "old contents")
            .await
            .unwrap();

        let error = Config::load_from_path(&path)
            .await
            .expect_err("backup file must block the load");

        assert!(
            error.to_string().contains("interrupted config migration"),
            "{error}"
        );
    }

    #[tokio::test]
    async fn a_backup_without_a_config_still_blocks_instead_of_defaulting() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");
        tokio::fs::write(backup_file_path(&path), "old contents")
            .await
            .unwrap();

        let error = Config::load_from_path(&path)
            .await
            .expect_err("the user's data is in the backup; a fresh default would hide it");

        assert!(
            error.to_string().contains("interrupted config migration"),
            "{error}"
        );
    }

    #[tokio::test]
    async fn v2_round_trips_without_a_migration() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("tvc.config.toml");

        let mut config = Config::default();
        config
            .add_org(ORG_1, API_BASE_URL_PROD.to_string())
            .unwrap();
        config.aliases.bind("prod".to_string(), ORG_1);
        config.set_active_org(ORG_1).unwrap();

        config.save_to_path(&path).await.unwrap();
        let loaded = Config::load_from_path(&path).await.unwrap();

        assert_eq!(loaded, config);
        assert!(!backup_file_path(&path).exists());
    }

    #[test]
    fn rejects_malformed_and_newer_versions() {
        let malformed = disk::from_toml("version = \"one\"")
            .map(|_| ())
            .expect_err("malformed must fail");
        assert!(
            malformed
                .to_string()
                .contains("config version must be an unsigned 16-bit integer")
        );

        let newer = disk::from_toml("version = 3")
            .map(|_| ())
            .expect_err("newer must fail");
        assert!(
            newer.to_string().contains("written by a newer tvc"),
            "{newer}"
        );
    }

    #[test]
    fn remove_org_unbinds_every_alias_and_clears_active() {
        let mut config = Config::default();
        config
            .add_org(ORG_1, API_BASE_URL_PROD.to_string())
            .unwrap();
        config.aliases.bind("prod".to_string(), ORG_1);
        config.aliases.bind("production".to_string(), ORG_1);
        config.set_active_org(ORG_1).unwrap();

        let (_, unbound) = config.remove_org(ORG_1).expect("org is configured");

        assert_eq!(unbound, ["prod", "production"]);
        assert_eq!(config.active_org, None);
        assert!(config.aliases.resolve("prod").is_none());
    }

    #[test]
    fn has_default_layout_at_requires_both_default_file_names() {
        let dir = Path::new("/keys/org");
        let org = test_org(API_BASE_URL_PROD, dir);

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
}
