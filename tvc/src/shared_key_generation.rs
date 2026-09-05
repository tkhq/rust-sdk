//! Offline credential creation with an explicit private-key destination.
use crate::{
    config::turnkey::{KeyCurve, StoredApiKey},
    shared_operations::OperationOutput,
};
use anyhow::{Context, Result};
use clap::Args;
use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use zeroize::Zeroizing;

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// New credential JSON path. Existing files are never overwritten.
    #[arg(long)]
    output: PathBuf,
}
impl GenerateArgs {
    pub fn run(self) -> Result<OperationOutput> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&self.output)
            .with_context(|| format!("create credential destination {}", self.output.display()))?;
        let result = (|| -> Result<_> {
            let key = TurnkeyP256ApiKey::generate();
            let public_key = hex::encode(key.compressed_public_key());
            let mut stored = StoredApiKey {
                public_key: public_key.clone(),
                private_key: hex::encode(key.private_key()),
                curve: KeyCurve::P256,
            };
            let encoded = serde_json::to_vec(&stored);
            use zeroize::Zeroize;
            stored.private_key.zeroize();
            let encoded = Zeroizing::new(encoded?);
            file.write_all(&encoded)
                .context("write private credential")?;
            file.sync_all().context("persist private credential")?;
            Ok(public_key)
        })();
        drop(file);
        match result {
            Ok(public_key) => Ok(OperationOutput::result(
                "api-key.generate",
                json!({"publicKey": public_key, "curve": "p256", "path": self.output}),
            )),
            Err(error) => {
                fs::remove_file(&self.output).context("remove incomplete credential file")?;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_credentials_are_valid_private_and_not_in_output() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("key.json");
        let output = GenerateArgs {
            output: path.clone(),
        }
        .run()
        .unwrap();
        let stored: StoredApiKey = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        TurnkeyP256ApiKey::from_strings(&stored.private_key, Some(&stored.public_key)).unwrap();
        let result = serde_json::to_value(output).unwrap();
        assert_eq!(result["data"]["publicKey"], stored.public_key);
        assert!(!result.to_string().contains(&stored.private_key));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }
    #[test]
    fn existing_destination_is_unchanged() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("key.json");
        fs::write(&path, b"existing").unwrap();
        assert!(
            GenerateArgs {
                output: path.clone()
            }
            .run()
            .is_err()
        );
        assert_eq!(fs::read(path).unwrap(), b"existing");
    }
    #[cfg(unix)]
    #[test]
    fn dangling_symlink_is_not_followed() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("target");
        let path = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &path).unwrap();
        assert!(GenerateArgs { output: path }.run().is_err());
        assert!(!target.exists());
    }
}
