//! Shared utils for TVC CLI.

use anyhow::Context;
use rand_core::{OsRng, RngCore};
use serde::de::DeserializeOwned;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::Path,
};

/// Read a file to string with contextual error information.
///
/// Trims leading and trailing whitespace from the content.
pub async fn read_file_to_string(path: &Path) -> anyhow::Result<String> {
    let content = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read file: {}", path.display()))?;
    Ok(content.trim().to_string())
}

/// Read and parse a JSON file with contextual error information.
pub(crate) async fn read_json_file<T>(path: &Path, label: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let contents = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read {label}: {}", path.display()))?;

    serde_json::from_slice(&contents)
        .with_context(|| format!("failed to parse {label}: {}", path.display()))
}

/// Write contents to a file with contextual error information.
pub async fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> anyhow::Result<()> {
    tokio::fs::write(path, contents)
        .await
        .with_context(|| format!("failed to write file: {}", path.display()))
}

/// Write contents to a file that is readable and writable only by its owner.
pub(crate) async fn write_owner_only_file(
    path: &Path,
    contents: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt as _;
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .await?;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .await?;
    file.write_all(contents.as_ref()).await?;
    file.flush().await
}

pub(crate) fn owner_only_dir(path: &Path) -> io::Result<()> {
    let created = match fs::DirBuilder::new().mode(0o700).create(path) {
        Ok(()) => true,
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or(error)?;
            owner_only_dir(parent)?;
            match fs::DirBuilder::new().mode(0o700).create(path) {
                Ok(()) => true,
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => false,
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    };
    if !fs::metadata(path)?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            "private directory path is not a directory",
        ));
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    if created && let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}

/// Hold this stable sidecar open to serialize updates across atomic replacements.
pub(crate) fn lock_owner_only_file(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.try_lock().map_err(io::Error::from)?;
    Ok(file)
}

/// Replace a private record durably without a cancellation point during the write.
pub(crate) fn atomic_write_owner_only(path: &Path, contents: &[u8]) -> io::Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.permissions().readonly() => {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "destination is read-only",
            ));
        }
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => (),
    }
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("file has no parent directory"))?;
    let mut random = [0; 16];
    OsRng
        .try_fill_bytes(&mut random)
        .map_err(io::Error::other)?;
    let temporary = parent.join(format!(".{}.tmp", hex::encode(random)));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)?;
        file.write_all(contents)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_private_directory_ancestors_and_atomic_records_have_owner_only_permissions() {
        let directory = tempfile::tempdir().unwrap();
        let parent = directory.path().join("config/turnkey/pending-creations");
        owner_only_dir(&parent).unwrap();
        for path in [
            directory.path().join("config"),
            directory.path().join("config/turnkey"),
            parent.clone(),
        ] {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        let path = parent.join("record.json");
        atomic_write_owner_only(&path, b"original").unwrap();
        atomic_write_owner_only(&path, b"replacement").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(fs::read_dir(parent).unwrap().count(), 1);
    }
}
