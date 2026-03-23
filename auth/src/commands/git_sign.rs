use clap::Args as ClapArgs;

use anyhow::anyhow;

use crate::ssh;
use crate::turnkey::{Signer, TurnkeySigner};

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub ssh_keygen_args: Vec<String>,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let invocation = ssh::GitSignInvocation::parse(&args.ssh_keygen_args)?;
    let signer = TurnkeySigner::from_env()?;
    let payload = read_payload(&invocation.payload_path).await?;
    let public_key = read_public_key(&invocation.public_key_path).await?;
    let armored = build_armored_signature(&payload, &public_key, &signer).await?;
    let signature_path = invocation.signature_path();
    write_signature(&signature_path, &armored).await?;
    Ok(())
}

async fn read_payload(path: &std::path::Path) -> anyhow::Result<Vec<u8>> {
    Ok(tokio::fs::read(path).await?)
}

async fn read_public_key(path: &std::path::Path) -> anyhow::Result<String> {
    Ok(tokio::fs::read_to_string(path).await?)
}

async fn write_signature(path: &std::path::Path, armored: &str) -> anyhow::Result<()> {
    Ok(tokio::fs::write(path, armored).await?)
}

pub(crate) async fn build_armored_signature<S>(
    payload: &[u8],
    public_key_line: &str,
    signer: &S,
) -> anyhow::Result<String>
where
    S: Signer + ?Sized,
{
    let parsed_public_key = ssh::parse_public_key_line(public_key_line)?;
    let configured_public_key = signer.get_public_key().await?;
    if parsed_public_key.public_key != configured_public_key {
        return Err(anyhow!(
            "requested SSH public key does not match the configured Turnkey key"
        ));
    }
    let signed_data = ssh::build_signed_data("git", payload);
    let signature = signer.sign_ed25519(&signed_data).await?;

    ssh::encode_armored_signature(
        &parsed_public_key.public_key_blob,
        "git",
        ssh::DEFAULT_HASH_ALGORITHM,
        &signature,
    )
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use anyhow::Result;

    use crate::ssh::encode_public_key_line;
    use crate::turnkey::Signer;

    use super::build_armored_signature;

    struct FakeSigner {
        public_key: Vec<u8>,
        signature: Vec<u8>,
    }

    impl Signer for FakeSigner {
        fn get_public_key(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
            Box::pin(async { Ok(self.public_key.clone()) })
        }

        fn sign_ed25519<'a>(
            &'a self,
            _payload: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            Box::pin(async { Ok(self.signature.clone()) })
        }
    }

    #[tokio::test]
    async fn build_armored_signature_rejects_mismatched_public_key() {
        let signer = FakeSigner {
            public_key: vec![0x11; 32],
            signature: vec![0x22; 64],
        };
        let requested = encode_public_key_line(&vec![0x33; 32], None).expect("should encode key");

        let error = build_armored_signature(b"payload", &requested, &signer)
            .await
            .expect_err("should reject mismatched key");

        assert!(error
            .to_string()
            .contains("does not match the configured Turnkey key"));
    }
}
