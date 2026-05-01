//! Shared utils for TVC CLI.

use anyhow::Context;
use serde::de::DeserializeOwned;
use std::path::Path;

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
