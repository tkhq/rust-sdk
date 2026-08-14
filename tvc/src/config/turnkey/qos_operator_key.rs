//! Stored QOS operator key for manifest signing.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{self, Debug, Display, Formatter},
    path::Path,
    str::FromStr,
};
use tracing::debug;

/// Byte length of a stored operator public key: two 65-byte uncompressed
/// SEC1 points.
const QOS_OPERATOR_PUBLIC_KEY_LEN: usize = 130;

/// A stored QOS operator public key, opaque to tvc.
///
/// The bytes are `qos_p256::P256Public::to_bytes()`'s composite encoding —
/// `encrypt_public ‖ sign_public`, two 65-byte uncompressed SEC1 points —
/// but tvc never reads the halves, so it deliberately doesn't model the
/// split. Hex is the display and serialization form.
///
/// TODO(TVC-270): the proper home for this type is qos_p256, as a bytemuck
/// repr-backed wire form of `P256Public` (which already holds the two keys
/// as separate fields, as does the Turnkey API's proto); move it upstream
/// and store that type here instead.
#[derive(Clone, Copy, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct QosOperatorPublicKey([u8; QOS_OPERATOR_PUBLIC_KEY_LEN]);

/// Error returned when parsing a [`QosOperatorPublicKey`].
#[derive(Debug, displaydoc::Display, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum QosOperatorPublicKeyParseError {
    /// must be bare hex encoded
    InvalidHex,
    /// must be 130 bytes, got {0}
    WrongLength(usize),
}

impl TryFrom<&[u8]> for QosOperatorPublicKey {
    type Error = QosOperatorPublicKeyParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        bytes
            .try_into()
            .map(Self)
            .map_err(|_| QosOperatorPublicKeyParseError::WrongLength(bytes.len()))
    }
}

impl FromStr for QosOperatorPublicKey {
    type Err = QosOperatorPublicKeyParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(value.trim()).map_err(|_| QosOperatorPublicKeyParseError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl Display for QosOperatorPublicKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

/// Implements `Debug` as `TypeName(<hex>)`. The type is passed as an
/// identifier so a rename that misses this call site fails to compile,
/// unlike a name written into a string literal.
macro_rules! impl_hex_debug {
    ($ty:ident) => {
        impl Debug for $ty {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($ty), "({})"), hex::encode(self.0))
            }
        }
    };
}

impl_hex_debug!(QosOperatorPublicKey);

/// The nil key. Exists for the payload-enumeration tests on [`crate::outcome::Outcome`],
/// which construct every outcome shape via `Default`.
impl Default for QosOperatorPublicKey {
    fn default() -> Self {
        // Not `Self(Default::default())`: std's array `Default` stops at 32
        // elements, so the repeat expression delegates per element instead.
        Self([Default::default(); QOS_OPERATOR_PUBLIC_KEY_LEN])
    }
}

/// Operator key stored in operator.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredQosOperatorKey {
    /// The composite public key (see [`QosOperatorPublicKey`]), hex-encoded
    /// on disk.
    pub public_key: QosOperatorPublicKey,
    /// Hex-encoded private key
    pub private_key: String,
}

impl StoredQosOperatorKey {
    /// Load an operator key from a registered `operator.json` path.
    pub async fn load(path: &Path) -> Result<Option<Self>> {
        debug!(operator_key_path = %path.display(), "loading stored operator key");
        if !path.exists() {
            debug!(operator_key_path = %path.display(), "stored operator key not found");
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("failed to read operator key: {}", path.display()))?;

        let key: StoredQosOperatorKey = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse operator key: {}", path.display()))?;

        debug!(operator_key_path = %path.display(), "loaded stored operator key");

        Ok(Some(key))
    }

    /// Save an operator key to a registered `operator.json` path.
    pub async fn save(&self, path: &Path) -> Result<()> {
        debug!(operator_key_path = %path.display(), "saving stored operator key");

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let content =
            serde_json::to_string_pretty(self).context("failed to serialize operator key")?;

        crate::util::write_owner_only_file(path, content)
            .await
            .with_context(|| format!("failed to write operator key: {}", path.display()))?;

        debug!(operator_key_path = %path.display(), "saved stored operator key");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_reprints_canonical_hex() {
        let hex_key = hex::encode([7u8; QOS_OPERATOR_PUBLIC_KEY_LEN]);

        let key: QosOperatorPublicKey = hex_key.parse().expect("valid length and charset");

        assert_eq!(key.to_string(), hex_key);
    }

    #[test]
    fn normalizes_case_and_whitespace() {
        let hex_key = hex::encode([0xABu8; QOS_OPERATOR_PUBLIC_KEY_LEN]);

        let key: QosOperatorPublicKey = format!(" {} ", hex_key.to_uppercase())
            .parse()
            .expect("uppercase hex with surrounding whitespace is accepted");

        assert_eq!(key.to_string(), hex_key);
    }

    #[test]
    fn rejects_wrong_length_and_bad_charset() {
        assert_eq!(
            "abcd".parse::<QosOperatorPublicKey>(),
            Err(QosOperatorPublicKeyParseError::WrongLength(2))
        );
        assert_eq!(
            "not hex".parse::<QosOperatorPublicKey>(),
            Err(QosOperatorPublicKeyParseError::InvalidHex)
        );
    }

    #[test]
    fn accepts_a_generated_composite_key() {
        let pair = qos_p256::P256Pair::generate().expect("keygen");

        let key = QosOperatorPublicKey::try_from(pair.public_key().to_bytes().as_slice())
            .expect("qos composite encoding is 130 bytes");

        assert_eq!(key.to_string(), hex::encode(pair.public_key().to_bytes()));
    }
}
