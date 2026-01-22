//! Shared utils for TVC CLI.

use anyhow::Context;
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

/// Write contents to a file with contextual error information.
pub async fn write_file(path: &Path, contents: impl AsRef<[u8]>) -> anyhow::Result<()> {
    tokio::fs::write(path, contents)
        .await
        .with_context(|| format!("failed to write file: {}", path.display()))
}
