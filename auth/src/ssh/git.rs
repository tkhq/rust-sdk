use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Parsed `ssh-keygen -Y sign` style invocation data for Git signing.
pub struct GitSignInvocation {
    /// Requested SSH signing namespace.
    pub namespace: String,
    /// Path to the OpenSSH public key file passed by Git.
    pub public_key_path: PathBuf,
    /// Path to the payload file Git wants signed.
    pub payload_path: PathBuf,
}

impl GitSignInvocation {
    /// Parses the `ssh-keygen -Y sign` style arguments Git passes to an SSH signer.
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut namespace = None;
        let mut public_key_path = None;
        let mut payload_path = None;
        let mut iter = args.iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-Y" => {
                    let value = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing value after -Y"))?;
                    if value != "sign" {
                        return Err(anyhow!("unsupported ssh signer operation: {value}"));
                    }
                }
                "-n" => {
                    namespace = Some(
                        iter.next()
                            .ok_or_else(|| anyhow!("missing value after -n"))?
                            .to_string(),
                    );
                }
                "-f" => {
                    public_key_path = Some(PathBuf::from(
                        iter.next()
                            .ok_or_else(|| anyhow!("missing value after -f"))?,
                    ));
                }
                value if value.starts_with('-') => {
                    return Err(anyhow!("unsupported ssh signer argument: {value}"));
                }
                value => {
                    payload_path = Some(PathBuf::from(value));
                }
            }
        }

        let namespace = namespace.ok_or_else(|| anyhow!("missing required -n <namespace>"))?;
        if namespace != "git" {
            return Err(anyhow!("unsupported ssh signing namespace: {namespace}"));
        }

        Ok(Self {
            namespace,
            public_key_path: public_key_path
                .ok_or_else(|| anyhow!("missing required -f <public-key-file>"))?,
            payload_path: payload_path.ok_or_else(|| anyhow!("missing payload file path"))?,
        })
    }

    /// Returns the output path where the detached signature should be written.
    pub fn signature_path(&self) -> PathBuf {
        PathBuf::from(format!("{}.sig", self.payload_path.display()))
    }
}
