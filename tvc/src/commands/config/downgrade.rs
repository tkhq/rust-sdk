//! Rewrite `tvc.config.toml` from schema version 2 back to version 1, so a
//! tvc release that predates version 2 can read it again after a rollback.
//!
//! This module deliberately knows nothing about the runtime config model. It
//! works on the raw TOML document and names only the keys whose shape differs
//! between the two schemas; everything else — the YubiKey registry, operator
//! records, fields written by other tvc versions — passes through untouched.
//! That keeps the command buildable on either side of the version-2
//! migration, which is the point: the release a user rolls back *to* must
//! already carry it.
//!
//! Version 2 keys organizations by ID and holds their human names in an
//! `[aliases]` table; version 1 keys organizations by name and repeats the ID
//! inside each entry. An organization with several names therefore comes out
//! as several version-1 profiles (the exact inverse of the upgrade's merge),
//! and one with no name is keyed by its ID.

use crate::{config::turnkey::config_file_path, outcome::Outcome};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

/// The schema this command reads.
const SOURCE_VERSION: u16 = 2;
/// The schema this command writes.
const TARGET_VERSION: u16 = 1;

/// Downgrade the config file at its default location in place, keeping the
/// version-2 original beside it as `tvc.config.toml.v2`.
///
/// Refuses while `tvc.config.toml.backup` exists — the fence a version-2 tvc
/// leaves behind when its own migration was interrupted, meaning the config
/// file's contents cannot be trusted — and while a `.v2` copy already exists,
/// so a previous copy is never overwritten. The original is copied before
/// anything is written, and the rewrite lands through a rename, so every
/// crash window leaves either the untouched original or its copy.
pub async fn run() -> Result<ConfigDowngraded> {
    let config_path = config_file_path()?;

    let sibling = |suffix: &str| {
        let mut name = config_path.as_os_str().to_owned();
        name.push(suffix);
        PathBuf::from(name)
    };

    let fence_path = sibling(".backup");
    let backup_path = sibling(".v2");
    let staging_path = sibling(".tmp");

    if fence_path.exists() {
        bail!(
            r#"found {fence} from an interrupted config migration; the config file cannot be downgraded until it is resolved.

Compare it with {config} (the backup is the pre-migration version; the config
file may be missing or already migrated), keep the right contents at
{config}, delete {fence}, and re-run."#,
            fence = fence_path.display(),
            config = config_path.display(),
        );
    }

    if backup_path.exists() {
        bail!(
            "{} already exists; move it aside or delete it before downgrading again, so the copy of the current file cannot overwrite it",
            backup_path.display()
        );
    }

    let content = tokio::fs::read_to_string(&config_path)
        .await
        .with_context(|| format!("failed to read config file: {}", config_path.display()))?;
    let downgraded = Downgraded::from_toml(&content)
        .with_context(|| format!("failed to downgrade config file: {}", config_path.display()))?;
    let rewritten = toml::to_string_pretty(&downgraded.document)
        .context("failed to serialize the downgraded config")?;

    tokio::fs::copy(&config_path, &backup_path)
        .await
        .with_context(|| format!("failed to copy the config to {}", backup_path.display()))?;
    tokio::fs::write(&staging_path, rewritten)
        .await
        .with_context(|| format!("failed to write {}", staging_path.display()))?;
    tokio::fs::rename(&staging_path, &config_path)
        .await
        .with_context(|| {
            format!(
                "failed to replace {} with the downgraded config",
                config_path.display()
            )
        })?;

    Ok(ConfigDowngraded {
        config_path,
        backup_path,
        notes: downgraded.notes,
    })
}

/// A version-1 document produced from a version-2 one, plus the human-facing
/// notes about every decision that was not a pure re-keying.
struct Downgraded {
    document: V1Document,
    notes: Vec<String>,
}

impl Downgraded {
    /// Parse a version-2 `tvc.config.toml` and convert it. Other versions are
    /// refused: 0 and 1 need no downgrade, anything newer is unknown here.
    fn from_toml(content: &str) -> Result<Self> {
        let mut table: toml::Table =
            toml::from_str(content).context("failed to parse config TOML")?;

        // The version marker comes off the table so it cannot be captured as
        // an unknown field and re-emitted next to the new one.
        let version = table
            .remove("version")
            .map(|value| {
                u16::deserialize(value).context("config version must be an unsigned 16-bit integer")
            })
            .transpose()?;

        match version {
            Some(SOURCE_VERSION) => {}
            None => bail!(
                "config is unversioned (schema version 0), which this tvc reads as is; nothing to downgrade"
            ),
            Some(version) if version < SOURCE_VERSION => bail!(
                "config is already at version {version}, which this tvc reads as is; nothing to downgrade"
            ),
            Some(version) => bail!(
                "config version {version} is not one this tvc can downgrade; only version {SOURCE_VERSION} is"
            ),
        }

        let document = V2Document::deserialize(toml::Value::Table(table))
            .context("failed to parse v2 config")?;
        Self::try_from(document)
    }
}

impl TryFrom<V2Document> for Downgraded {
    type Error = anyhow::Error;

    fn try_from(v2: V2Document) -> Result<Self> {
        let V2Document {
            active_org,
            orgs,
            aliases,
            last_created_app_id,
            last_operator_ids,
            extra,
        } = v2;
        let mut notes = Vec::new();

        // Names per organization, alphabetical: the version-1 file has no
        // alias table, so every name becomes a profile key of its own.
        let mut names_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

        for (name, org_id) in &aliases {
            if orgs.contains_key(org_id) {
                names_of
                    .entry(org_id.as_str())
                    .or_default()
                    .push(name.as_str());
            } else {
                notes.push(format!(
                    "alias '{name}' points at organization {org_id}, which has no config table; dropped"
                ));
            }
        }

        // Every organization's profile keys and its table: the names bound to
        // it, or its bare ID when nothing is bound.
        let profiles = orgs
            .iter()
            .map(|(org_id, table)| {
                let names = names_of
                    .remove(org_id.as_str())
                    .unwrap_or_else(|| vec![org_id.as_str()]);
                (org_id.as_str(), (names, table))
            })
            .collect::<BTreeMap<_, _>>();

        notes.extend(
            profiles
                .iter()
                .filter_map(|(org_id, (names, _))| match names.as_slice() {
                    [sole] if sole == org_id => Some(format!(
                        "organization {org_id} has no alias; it is registered as profile '{org_id}'"
                    )),
                    [_] => None,
                    many => Some(format!(
                        "organization {org_id} is registered under {} profiles ({}); they share its key files",
                        many.len(),
                        many.join(", ")
                    )),
                }),
        );

        let mut v1_orgs = BTreeMap::new();

        profiles
            .iter()
            .flat_map(|(org_id, (names, table))| {
                names.iter().map(move |name| (*name, *org_id, *table))
            })
            .try_for_each(|(name, org_id, table)| {
                let mut profile = table.clone();
                profile.insert("id".to_string(), toml::Value::String(org_id.to_string()));

                ensure!(
                    v1_orgs.insert(name.to_string(), profile).is_none(),
                    "profile name '{name}' would be claimed by two organizations; rename the alias before downgrading"
                );
                Ok(())
            })?;

        let active_org = active_org.and_then(|org_id| match profiles.get(org_id.as_str()) {
            Some((names, _)) => names.first().map(|name| name.to_string()),
            None => {
                notes.push(format!(
                    "active organization {org_id} is not configured; the active profile was cleared"
                ));
                None
            }
        });

        // Convenience state follows its organization onto every profile.
        let mut rekey = |map: BTreeMap<String, toml::Value>| {
            map.into_iter()
                .flat_map(|(org_id, value)| match profiles.get(org_id.as_str()) {
                    Some((names, _)) => names
                        .iter()
                        .map(|name| (name.to_string(), value.clone()))
                        .collect::<Vec<_>>(),
                    None => {
                        notes.push(format!(
                            "dropped convenience state for organization {org_id}, which is not configured"
                        ));
                        Vec::new()
                    }
                })
                .collect::<BTreeMap<_, _>>()
        };

        let last_created_app_id = rekey(last_created_app_id);
        let last_operator_ids = rekey(last_operator_ids);

        Ok(Self {
            document: V1Document {
                version: TARGET_VERSION,
                active_org,
                orgs: v1_orgs,
                last_created_app_id,
                last_operator_ids,
                extra,
            },
            notes,
        })
    }
}

/// The version-2 document, naming only the keys whose shape differs from
/// version 1. Organization tables stay opaque: nothing inside them changes.
#[derive(Deserialize)]
struct V2Document {
    #[serde(default)]
    active_org: Option<String>,
    #[serde(default)]
    orgs: BTreeMap<String, toml::Table>,
    #[serde(default)]
    aliases: BTreeMap<String, String>,
    #[serde(default)]
    last_created_app_id: BTreeMap<String, toml::Value>,
    #[serde(default)]
    last_operator_ids: BTreeMap<String, toml::Value>,
    /// Everything else — the YubiKey registry and fields owned by other
    /// writers — is identical in both schemas and passes through.
    #[serde(flatten)]
    extra: toml::Table,
}

/// The version-1 document as written: the same opaque pass-through, plus the
/// version marker every version-1 file carries.
#[derive(Serialize)]
struct V1Document {
    version: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    active_org: Option<String>,
    orgs: BTreeMap<String, toml::Table>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    last_created_app_id: BTreeMap<String, toml::Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    last_operator_ids: BTreeMap<String, toml::Value>,
    #[serde(flatten)]
    extra: toml::Table,
}

/// Terminal outcome of `config downgrade`.
#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct ConfigDowngraded {
    config_path: PathBuf,
    backup_path: PathBuf,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

impl From<ConfigDowngraded> for Outcome {
    fn from(downgraded: ConfigDowngraded) -> Self {
        Outcome::ConfigDowngraded(downgraded)
    }
}

impl Display for ConfigDowngraded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Rewrote {} to config schema version {TARGET_VERSION}, readable by tvc releases that predate version {SOURCE_VERSION}.
The version-{SOURCE_VERSION} file was kept at {}; delete it once the older release works."#,
            self.config_path.display(),
            self.backup_path.display()
        )?;

        if !self.notes.is_empty() {
            f.write_str("\n\nNotes:")?;
        }

        self.notes
            .iter()
            .try_for_each(|note| write!(f, "\n  - {note}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORG: &str = "11111111-1111-4111-8111-111111111111";
    const NAMELESS_ORG: &str = "22222222-2222-4222-8222-222222222222";
    const GHOST_ORG: &str = "99999999-9999-4999-8999-999999999999";

    /// Two names on one organization, a nameless organization, an alias to
    /// an organization with no table, a registry entry, both convenience
    /// maps, and unknown root and organization fields.
    fn v2_config() -> String {
        format!(
            r#"
version = 2
active_org = "{ORG}"
future_root = "keep-root"

[aliases]
work = "{ORG}"
turnkey = "{ORG}"
ghost = "{GHOST_ORG}"

[[yubikeys]]
serial = "01c95c1f"
public_key = "{key}"
future_entry = "keep"

[orgs.{ORG}]
api_key_path = "/keys/work/api.json"
api_base_url = "https://api.turnkey.com"
default_operator_kind = "local"
future_org = 42

[[orgs.{ORG}.operators]]
name = "default"
kind = "local"
key_path = "/keys/work/operator.json"

[orgs.{NAMELESS_ORG}]
api_key_path = "/keys/nameless/api.json"

[last_created_app_id]
{ORG} = "33333333-3333-4333-8333-333333333333"
{GHOST_ORG} = "44444444-4444-4444-8444-444444444444"

[last_operator_ids]
{ORG} = ["55555555-5555-4555-8555-555555555555"]
"#,
            key = "07".repeat(130)
        )
    }

    fn expected_v1_config() -> String {
        format!(
            r#"
version = 1
active_org = "turnkey"
future_root = "keep-root"

[[yubikeys]]
serial = "01c95c1f"
public_key = "{key}"
future_entry = "keep"

[orgs.turnkey]
id = "{ORG}"
api_key_path = "/keys/work/api.json"
api_base_url = "https://api.turnkey.com"
default_operator_kind = "local"
future_org = 42

[[orgs.turnkey.operators]]
name = "default"
kind = "local"
key_path = "/keys/work/operator.json"

[orgs.work]
id = "{ORG}"
api_key_path = "/keys/work/api.json"
api_base_url = "https://api.turnkey.com"
default_operator_kind = "local"
future_org = 42

[[orgs.work.operators]]
name = "default"
kind = "local"
key_path = "/keys/work/operator.json"

[orgs.{NAMELESS_ORG}]
id = "{NAMELESS_ORG}"
api_key_path = "/keys/nameless/api.json"

[last_created_app_id]
turnkey = "33333333-3333-4333-8333-333333333333"
work = "33333333-3333-4333-8333-333333333333"

[last_operator_ids]
turnkey = ["55555555-5555-4555-8555-555555555555"]
work = ["55555555-5555-4555-8555-555555555555"]
"#,
            key = "07".repeat(130)
        )
    }

    fn table(content: &str) -> toml::Table {
        toml::from_str(content).unwrap()
    }

    fn rewritten(downgraded: &Downgraded) -> toml::Table {
        table(&toml::to_string_pretty(&downgraded.document).unwrap())
    }

    #[test]
    fn rewrites_a_v2_document_as_the_expected_v1_document() {
        let downgraded = Downgraded::from_toml(&v2_config()).unwrap();

        assert_eq!(rewritten(&downgraded), table(&expected_v1_config()));
        assert_eq!(
            downgraded.notes,
            vec![
                format!(
                    "alias 'ghost' points at organization {GHOST_ORG}, which has no config table; dropped"
                ),
                format!(
                    "organization {ORG} is registered under 2 profiles (turnkey, work); they share its key files"
                ),
                format!(
                    "organization {NAMELESS_ORG} has no alias; it is registered as profile '{NAMELESS_ORG}'"
                ),
                format!(
                    "dropped convenience state for organization {GHOST_ORG}, which is not configured"
                ),
            ]
        );
    }

    #[test]
    fn a_single_name_per_organization_produces_no_notes() {
        let downgraded = Downgraded::from_toml(&format!(
            r#"
version = 2
active_org = "{ORG}"

[aliases]
work = "{ORG}"

[orgs.{ORG}]
api_key_path = "/keys/work/api.json"
"#
        ))
        .unwrap();

        assert!(downgraded.notes.is_empty(), "{:?}", downgraded.notes);
        assert_eq!(
            rewritten(&downgraded),
            table(&format!(
                r#"
version = 1
active_org = "work"

[orgs.work]
id = "{ORG}"
api_key_path = "/keys/work/api.json"
"#
            ))
        );
    }

    #[test]
    fn an_active_organization_without_a_table_is_cleared_with_a_note() {
        let downgraded = Downgraded::from_toml(&format!(
            r#"
version = 2
active_org = "{GHOST_ORG}"

[orgs.{ORG}]
api_key_path = "/keys/api.json"
"#
        ))
        .unwrap();

        assert!(downgraded.document.active_org.is_none());
        assert_eq!(
            downgraded.notes,
            vec![
                format!("organization {ORG} has no alias; it is registered as profile '{ORG}'"),
                format!(
                    "active organization {GHOST_ORG} is not configured; the active profile was cleared"
                ),
            ]
        );
    }

    #[test]
    fn an_alias_spelling_a_nameless_organization_id_is_refused() {
        let error = Downgraded::from_toml(&format!(
            r#"
version = 2

[aliases]
{NAMELESS_ORG} = "{ORG}"

[orgs.{ORG}]
api_key_path = "/keys/work/api.json"

[orgs.{NAMELESS_ORG}]
api_key_path = "/keys/nameless/api.json"
"#
        ))
        .map(|_| ())
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "profile name '{NAMELESS_ORG}' would be claimed by two organizations; rename the alias before downgrading"
            )
        );
    }

    #[test]
    fn refuses_a_v1_document() {
        let error = Downgraded::from_toml("version = 1\n")
            .map(|_| ())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "config is already at version 1, which this tvc reads as is; nothing to downgrade"
        );
    }

    #[test]
    fn refuses_an_unversioned_document() {
        let error = Downgraded::from_toml("active_org = \"work\"\n")
            .map(|_| ())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "config is unversioned (schema version 0), which this tvc reads as is; nothing to downgrade"
        );
    }

    #[test]
    fn refuses_an_unknown_newer_version() {
        let error = Downgraded::from_toml("version = 3\n")
            .map(|_| ())
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "config version 3 is not one this tvc can downgrade; only version 2 is"
        );
    }

    #[test]
    fn rejects_a_malformed_version() {
        let error = Downgraded::from_toml("version = \"two\"\n")
            .map(|_| ())
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("config version must be an unsigned 16-bit integer"),
            "{error:#}"
        );
    }

    #[test]
    fn outcome_serializes_with_its_reason_and_paths() {
        let outcome = Outcome::from(ConfigDowngraded {
            config_path: PathBuf::from("/home/me/.config/turnkey/tvc.config.toml"),
            backup_path: PathBuf::from("/home/me/.config/turnkey/tvc.config.toml.v2"),
            notes: vec!["a note".to_string()],
        });

        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            serde_json::json!({
                "reason": "config_downgraded",
                "configPath": "/home/me/.config/turnkey/tvc.config.toml",
                "backupPath": "/home/me/.config/turnkey/tvc.config.toml.v2",
                "notes": ["a note"],
            })
        );
    }
}
