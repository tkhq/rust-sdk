use crate::util::read_file_to_string;
use anyhow::anyhow;
use qos_p256::P256Pair;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use zeroize::Zeroizing;

/// Boxed future returned by key pair operations.
pub type PairFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Something that can do key pair operations with the QOS p256 scheme.
pub trait Pair: Send + Sync {
    /// Sign the given message.
    fn sign(&self, message: Vec<u8>) -> PairFuture<'_, anyhow::Result<Vec<u8>>>;

    /// Decrypt the given ciphertext.
    fn decrypt(&self, ciphertext: Vec<u8>) -> PairFuture<'_, anyhow::Result<Zeroizing<Vec<u8>>>>;

    /// The public key for this pair.
    fn public_key(&self) -> Vec<u8>;
}

/// A 32-byte master seed parsed from hex. Accepts surrounding whitespace and
/// an optional `0x` prefix. Zeroized on drop.
#[derive(Clone)]
pub struct HexSeed(Zeroizing<[u8; 32]>);

impl FromStr for HexSeed {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex_seed = s.trim();
        let hex_seed = hex_seed.strip_prefix("0x").unwrap_or(hex_seed);
        let bytes_seed: [u8; 32] = hex::decode(hex_seed)?
            .try_into()
            .map_err(|v: Vec<u8>| anyhow!("seed must be exactly 32 bytes, got {}", v.len()))?;

        Ok(Self(Zeroizing::new(bytes_seed)))
    }
}

// The seed must never reach debug output, so the derive is off the table.
impl std::fmt::Debug for HexSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HexSeed(..)")
    }
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
        Self::from_seed(&hex_seed.parse()?)
    }

    /// Create a local signer from a parsed master seed.
    pub fn from_seed(seed: &HexSeed) -> anyhow::Result<Self> {
        let pair = P256Pair::from_master_seed(&seed.0)
            .map_err(|_| anyhow!("could not create key from seed"))?;

        Ok(Self {
            pair: Arc::new(pair),
        })
    }
}

impl Pair for LocalPair {
    fn sign(&self, message: Vec<u8>) -> PairFuture<'_, anyhow::Result<Vec<u8>>> {
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

    fn decrypt(&self, ciphertext: Vec<u8>) -> PairFuture<'_, anyhow::Result<Zeroizing<Vec<u8>>>> {
        let pair2 = Arc::clone(&self.pair);

        Box::pin(async move {
            tokio::task::spawn_blocking(move || {
                pair2
                    .decrypt(&ciphertext)
                    .map_err(|_| anyhow!("failed to decrypt with local signer"))
            })
            .await?
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_hex() -> String {
        "ab".repeat(32)
    }

    #[test]
    fn hex_seed_parses_bare_hex() {
        let seed: HexSeed = valid_hex().parse().unwrap();
        assert_eq!(*seed.0, [0xab; 32]);
    }

    #[test]
    fn hex_seed_ignores_0x_prefix_and_whitespace() {
        let seed: HexSeed = format!(" 0x{}\n", valid_hex()).parse().unwrap();
        assert_eq!(*seed.0, [0xab; 32]);
    }

    #[test]
    fn hex_seed_rejects_non_hex() {
        assert!("zz".repeat(32).parse::<HexSeed>().is_err());
    }

    #[test]
    fn hex_seed_rejects_wrong_length() {
        let err = "ab".repeat(31).parse::<HexSeed>().unwrap_err();
        assert!(err.to_string().contains("exactly 32 bytes"));
    }

    #[test]
    fn hex_seed_debug_redacts_the_seed() {
        let seed: HexSeed = valid_hex().parse().unwrap();
        assert_eq!(format!("{seed:?}"), "HexSeed(..)");
    }
}
