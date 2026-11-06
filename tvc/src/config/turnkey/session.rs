use super::{Config, OperatorRecordKind, StoredQosOperatorKey, disk};
use anyhow::{Context, Result};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};
use turnkey_proofs::tvc::RolePublicKey;

/// A configuration transaction's recovery-relevant outcome.
#[derive(Debug, thiserror::Error)]
pub enum ConfigSaveError {
    /// Another writer changed the exact loaded snapshot.
    #[error("config changed since it was loaded at {path}; reload before retrying the local save")]
    StaleConflict { path: PathBuf },
    /// Replacement did not happen. The previous file remains intact.
    #[error("config was not replaced at {path}")]
    BeforeReplacement {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },
    /// Replacement happened, but its durability could not be confirmed.
    #[error(
        "config was replaced at {path}, but durability is uncertain; inspect it before recovery or cleanup"
    )]
    DurabilityUncertain {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },
}

/// A missing or malformed local role-key inventory cannot authorize insertion.
#[derive(Debug, thiserror::Error)]
#[error("cannot obtain complete stored operator-role inventory: {0}")]
pub struct KeyInventoryError(#[source] anyhow::Error);

#[derive(Clone, PartialEq, Eq)]
enum Baseline {
    Missing,
    Bytes(Vec<u8>),
}

/// Exclusive ownership of a loaded path, exact snapshot, and mutable config.
///
/// The capability is deliberately not cloneable. Manual editors, older CLI
/// binaries and independently changed key files do not obey this lock.
pub struct ConfigSession {
    path: PathBuf,
    baseline: Baseline,
    config: Config,
}

impl Deref for ConfigSession {
    type Target = Config;
    fn deref(&self) -> &Config {
        &self.config
    }
}
impl DerefMut for ConfigSession {
    fn deref_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

impl ConfigSession {
    /// Load without writing, preserving absence separately from empty bytes.
    pub async fn load(path: PathBuf) -> Result<Self> {
        let baseline = read_baseline(&path)?;
        let config = match &baseline {
            Baseline::Missing => Config::default(),
            Baseline::Bytes(bytes) => {
                Config::from_toml(std::str::from_utf8(bytes).context("config is not UTF-8")?)
                    .with_context(|| format!("failed to parse config file: {}", path.display()))?
            }
        };
        Ok(Self {
            path,
            baseline,
            config,
        })
    }

    /// Give read-only handlers their data without a persistence capability.
    pub fn into_config(self) -> Config {
        self.config
    }

    /// Commit under a stable interprocess lock, rejecting stale snapshots.
    pub async fn save(&mut self) -> Result<(), ConfigSaveError> {
        self.save_checked(None).await
    }

    /// Save accepted insertion only after rechecking both roles against the
    /// current disk inventory while the configuration lock is held.
    pub(crate) async fn save_verified_roles(
        &mut self,
        roles: [RolePublicKey; 2],
    ) -> Result<(), ConfigSaveError> {
        self.save_checked(Some(roles)).await
    }

    async fn save_checked(
        &mut self,
        roles: Option<[RolePublicKey; 2]>,
    ) -> Result<(), ConfigSaveError> {
        let path = self.path.clone();
        let baseline = self.baseline.clone();
        let content = disk::to_toml(&self.config)
            .map_err(|source| ConfigSaveError::BeforeReplacement {
                path: path.clone(),
                source,
            })?
            .into_bytes();
        let (result, next) = tokio::task::spawn_blocking(move || {
            let result = commit(&path, baseline, &content, roles, FailurePoint::None);
            (result, content)
        })
        .await
        .map_err(|source| ConfigSaveError::BeforeReplacement {
            path: self.path.clone(),
            source: source.into(),
        })?;
        if result.is_ok() || matches!(result, Err(ConfigSaveError::DurabilityUncertain { .. })) {
            self.baseline = Baseline::Bytes(next);
        }
        result
    }
}

fn read_baseline(path: &Path) -> Result<Baseline> {
    match fs::read(path) {
        Ok(bytes) => Ok(Baseline::Bytes(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Baseline::Missing),
        Err(error) => {
            Err(error).with_context(|| format!("failed to read config file: {}", path.display()))
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FailurePoint {
    None,
    #[cfg(test)]
    BeforeWrite,
    #[cfg(test)]
    BeforeReplace,
    #[cfg(test)]
    AfterReplace,
}

fn commit(
    path: &Path,
    baseline: Baseline,
    content: &[u8],
    roles: Option<[RolePublicKey; 2]>,
    failure: FailurePoint,
) -> Result<(), ConfigSaveError> {
    let before = |source| ConfigSaveError::BeforeReplacement {
        path: path.to_owned(),
        source,
    };
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| before(error.into()))?;
    let mut lock_name = path.as_os_str().to_owned();
    lock_name.push(".lock");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let lock = options
        .open(PathBuf::from(lock_name))
        .map_err(|error| before(error.into()))?;
    tracing::debug!(config_path = %path.display(), "waiting for config commit lock");
    lock.lock().map_err(|error| before(error.into()))?;
    // Never remove the stable lock path. Dropping the file releases its lock.
    let current = read_baseline(path).map_err(before)?;
    if current != baseline {
        return Err(ConfigSaveError::StaleConflict {
            path: path.to_owned(),
        });
    }
    if matches!(fs::metadata(path), Ok(metadata) if metadata.permissions().readonly()) {
        return Err(before(
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "config is read-only").into(),
        ));
    }
    if let Some(roles) = roles {
        let config = match current {
            Baseline::Missing => Config::default(),
            Baseline::Bytes(bytes) => {
                Config::from_toml(std::str::from_utf8(&bytes).map_err(|e| before(e.into()))?)
                    .map_err(before)?
            }
        };
        let inventory = config.operator_role_keys().map_err(|e| before(e.into()))?;
        if roles.iter().any(|role| inventory.contains(role)) {
            return Err(before(
                turnkey_proofs::tvc::TvcProvenanceError::KeyReuse.into(),
            ));
        }
    }
    let mut temporary =
        tempfile::NamedTempFile::new_in(parent).map_err(|error| before(error.into()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| before(error.into()))?;
    }
    #[cfg(test)]
    if failure == FailurePoint::BeforeWrite {
        return Err(before(
            std::io::Error::other("injected temporary write failure").into(),
        ));
    }
    temporary
        .write_all(content)
        .map_err(|error| before(error.into()))?;
    temporary.flush().map_err(|error| before(error.into()))?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| before(error.into()))?;
    #[cfg(test)]
    if failure == FailurePoint::BeforeReplace {
        return Err(before(
            std::io::Error::other("injected before replacement").into(),
        ));
    }
    let _ = failure;
    temporary
        .persist(path)
        .map_err(|error| before(error.error.into()))?;
    let after = |source| ConfigSaveError::DurabilityUncertain {
        path: path.to_owned(),
        source,
    };
    #[cfg(test)]
    if failure == FailurePoint::AfterReplace {
        return Err(after(
            std::io::Error::other("injected after replacement").into(),
        ));
    }
    #[cfg(unix)]
    File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| after(error.into()))?;
    Ok(())
}

impl Config {
    /// Complete inventory of both roles, including unmarked legacy records.
    /// Missing or malformed key files and hardware cache entries fail closed.
    pub(crate) fn operator_role_keys(&self) -> Result<Vec<RolePublicKey>, KeyInventoryError> {
        (|| {
            let mut keys = Vec::new();
            for org in self.orgs.values() {
                for operator in &org.operators {
                    match &operator.kind {
                        OperatorRecordKind::Hosted(hosted) => {
                            keys.push(RolePublicKey::parse(&hosted.encrypt_public_key)?);
                            keys.push(RolePublicKey::parse(&hosted.sign_public_key)?);
                        }
                        OperatorRecordKind::Local(local) => {
                            let bytes = fs::read(&local.key_path).with_context(|| {
                                format!("read local operator key {}", local.key_path.display())
                            })?;
                            let stored: StoredQosOperatorKey = serde_json::from_slice(&bytes)
                                .with_context(|| {
                                    format!("parse local operator key {}", local.key_path.display())
                                })?;
                            let composite = stored.public_key.to_string();
                            keys.push(RolePublicKey::parse(&composite[..130])?);
                            keys.push(RolePublicKey::parse(&composite[130..])?);
                        }
                        OperatorRecordKind::Yubikey(device) => {
                            let entry = self
                                .yubikeys
                                .get(device.serial)
                                .context("missing YubiKey registry entry")?;
                            let composite = entry.public_key.to_string();
                            keys.push(RolePublicKey::parse(&composite[..130])?);
                            keys.push(RolePublicKey::parse(&composite[130..])?);
                        }
                    }
                }
            }
            for serial in self.yubikeys.serials() {
                let entry = self
                    .yubikeys
                    .get(serial)
                    .context("missing YubiKey registry entry")?;
                let composite = entry.public_key.to_string();
                keys.push(RolePublicKey::parse(&composite[..130])?);
                keys.push(RolePublicKey::parse(&composite[130..])?);
            }
            Ok(keys)
        })()
        .map_err(KeyInventoryError)
    }
}

#[cfg(test)]
mod tests;
