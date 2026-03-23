use clap::Args as ClapArgs;

use crate::ssh::encode_public_key_line;
use crate::turnkey::{Signer, TurnkeySigner};

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {}

pub async fn run(_args: Args) -> anyhow::Result<()> {
    let signer = TurnkeySigner::from_env()?;
    println!("{}", render_public_key_line(&signer).await?);
    Ok(())
}

pub(crate) async fn render_public_key_line<S>(signer: &S) -> anyhow::Result<String>
where
    S: Signer + ?Sized,
{
    let public_key = signer.get_public_key().await?;
    encode_public_key_line(&public_key, None)
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;

    use anyhow::{anyhow, Result};

    use crate::turnkey::Signer;

    use super::render_public_key_line;

    struct FakeSigner {
        public_key: Vec<u8>,
    }

    impl Signer for FakeSigner {
        fn get_public_key(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
            Box::pin(async { Ok(self.public_key.clone()) })
        }

        fn sign_ed25519<'a>(
            &'a self,
            _payload: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
            Box::pin(async { Err(anyhow!("unexpected sign")) })
        }
    }

    #[tokio::test]
    async fn render_public_key_line_returns_openssh_line() {
        let signer = FakeSigner {
            public_key: vec![0x66; 32],
        };

        let rendered = render_public_key_line(&signer)
            .await
            .expect("should render");

        assert_eq!(
            rendered,
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm"
        );
    }
}
