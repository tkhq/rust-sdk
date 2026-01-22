use crate::util::read_file_to_string;
use anyhow::anyhow;
use qos_p256::P256Pair;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

/// Something that can do key pair operations with the QOS p256 scheme.
pub trait Pair: Send + Sync {
    /// Sign the given message.
    fn sign(
        &self,
        message: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + Send + '_>>;

    /// The public key for this pair.
    fn public_key(&self) -> Vec<u8>;
}

/// A local QOS P256 key pair.
#[derive(Clone)]
pub struct LocalPair {
    pair: Arc<P256Pair>,
}

impl LocalPair {
    /// Create a local signer from a file containing a 32 byte, hex encoded seed.
    pub async fn from_master_seed(path: &Path) -> anyhow::Result<Self> {
        let hex_seed = read_file_to_string(path).await?;
        Self::from_hex_seed(&hex_seed)
    }

    /// Create a local signer from a hex-encoded 32 byte seed string.
    pub fn from_hex_seed(hex_seed: &str) -> anyhow::Result<Self> {
        let bytes_seed: [u8; 32] = hex::decode(hex_seed.trim())?
            .try_into()
            .map_err(|v: Vec<u8>| anyhow!("seed must be exactly 32 bytes, got {}", v.len()))?;

        let pair = P256Pair::from_master_seed(&bytes_seed)
            .map_err(|_| anyhow!("could not create key from seed"))?;

        Ok(Self {
            pair: Arc::new(pair),
        })
    }
}

impl Pair for LocalPair {
    fn sign(
        &self,
        message: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<u8>>> + Send + '_>> {
        let pair2 = Arc::clone(&self.pair);

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                pair2
                    .sign(&message)
                    .map_err(|_| anyhow!("failed to sign with local signer"))
            })
            .await?
        })
    }

    fn public_key(&self) -> Vec<u8> {
        self.pair.public_key().to_bytes()
    }
}
