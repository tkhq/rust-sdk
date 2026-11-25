//! Contains the definition and data for Turnkey Quorum key public keys.
use std::str::FromStr;

use p256::ecdsa::VerifyingKey;

use crate::errors::EnclaveEncryptError;

/// Turnkey's production signer quorum public key
/// Key export/import bundles are signed by this key.
pub const TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY: &str = "04ca7c0d624c75de6f34af342e87a21e0d8c83efd1bd5b5da0c0177c147f744fba6f01f9f37356f9c617659aafa55f6e0af8d169a8f054d153ab3201901fb63ecb04cf288fe433cc4e1aa0ce1632feac4ea26bf2f5a09dcfe5a42c398e06898710330f0572882f4dbdf0f5304b8fc8703acd69adca9a4bbf7f5d00d20a5e364b2569";

/// Turnkey's preprod signer quorum public key
/// We check it in here for convenience, for internal Turnkey employees. Goes without saying: do not use in production code!
pub const TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY: &str = "048e92f6cdcc0b375505980a298d9b79201db1f08b1f135360d2864af1a67186ec0dbeb570d396a456226b0844be93dbc0180abbf7e2e4c9cfde8d5da4e3f8a49004f3422b8afbe425d6ece77b8d2469954715a2ff273ab7ac89f1ed70e0a9325eaa1698b4351fd1b23734e65c0b6a86b62dd49d70b37c94606aac402cbd84353212";

/// Turnkey's development Signer quorum public key. Do not use in production.
pub const TURNKEY_DEV_SIGNER_QUORUM_PUBLIC_KEY: &str = "046101205064b86e23b9619dad4a887a3cb31ac4cb2cb8256556fdf315c7d42d9cece7bbc7b5ebc70a34b2faa4d1f0dffad0c2706b08f0065e0cb0534140b66564048cf9ed5f579298cc1571823a3222b82d80c529c551f6070fbe712ae1a9e8d1a23b7006e306d27190358dfcd9c44624918a00f23c920a33cb14f5b026eafc865d";

/// Expected Quorum public key length: twice the length of a SEC1 uncompressed public key (65 * 2)
pub const QUORUM_PUBLIC_KEY_BYTE_LENGTH: usize = 130;
const UNCOMPRESSED_POINT_BYTE_LENGTH: usize = QUORUM_PUBLIC_KEY_BYTE_LENGTH / 2;

/// A quorum public key with uncompressed P-256 encryption and signing points, in that order.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QuorumPublicKey {
    bytes: Vec<u8>,
}

impl QuorumPublicKey {
    /// Parses a quorum public key from bytes and validates both points.
    #[deprecated(note = "use QuorumPublicKey::try_from with a byte slice")]
    pub fn from_bytes<B: AsRef<[u8]>>(b: B) -> Result<Self, EnclaveEncryptError> {
        Self::try_from(b.as_ref())
    }

    /// Parses a hex-encoded quorum public key and validates both points.
    #[deprecated(note = "use str::parse::<QuorumPublicKey>")]
    pub fn from_string<S: AsRef<str>>(s: S) -> Result<Self, EnclaveEncryptError> {
        s.as_ref().parse()
    }

    /// Quorum Public Key for Turnkey's production signer.
    ///
    /// # Panics
    /// Not expected given the public keys are static and valid
    #[must_use]
    pub fn production_signer() -> Self {
        TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY
            .parse()
            .expect("static public key should be valid")
    }

    /// Quorum Public Key for Turnkey's preprod signer. Do not use in production code. See `production_signer`
    ///
    /// # Panics
    /// Not expected given the public keys are static and valid
    #[must_use]
    pub fn preprod_signer() -> Self {
        TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY
            .parse()
            .expect("static public key should be valid")
    }

    /// Quorum public key for Turnkey's development Signer. Do not use in production.
    ///
    /// # Panics
    /// Not expected given the public key is static and valid.
    #[must_use]
    pub fn dev_signer() -> Self {
        TURNKEY_DEV_SIGNER_QUORUM_PUBLIC_KEY
            .parse()
            .expect("static public key should be valid")
    }

    /// Returns a `VerifyingKey` for this `QuorumPublicKey`
    pub fn verifying_key(&self) -> Result<VerifyingKey, EnclaveEncryptError> {
        VerifyingKey::from_sec1_bytes(&self.bytes[UNCOMPRESSED_POINT_BYTE_LENGTH..])
            .map_err(|_| EnclaveEncryptError::InvalidVerifyingKeyBytes)
    }

    /// Returns the encryption target `PublicKey` for this `QuorumPublicKey`
    pub fn encrypt_public_key(&self) -> Result<p256::PublicKey, EnclaveEncryptError> {
        p256::PublicKey::from_sec1_bytes(&self.bytes[..UNCOMPRESSED_POINT_BYTE_LENGTH])
            .map_err(|_| EnclaveEncryptError::InvalidEncryptionKey)
    }
}

impl AsRef<[u8]> for QuorumPublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl FromStr for QuorumPublicKey {
    type Err = EnclaveEncryptError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(value).map_err(|e| EnclaveEncryptError::HexDecode(e.to_string()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for QuorumPublicKey {
    type Error = EnclaveEncryptError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != QUORUM_PUBLIC_KEY_BYTE_LENGTH {
            return Err(EnclaveEncryptError::IncorrectQuorumPublicKeyBytesLength(
                bytes.len(),
            ));
        }
        let key = Self {
            bytes: bytes.to_vec(),
        };
        key.encrypt_public_key()?;
        key.verifying_key()?;
        Ok(key)
    }
}

impl TryFrom<qos_p256::P256Public> for QuorumPublicKey {
    type Error = EnclaveEncryptError;

    fn try_from(value: qos_p256::P256Public) -> Result<Self, Self::Error> {
        Self::try_from(value.to_bytes().as_slice())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn assert_signer_key(hex_key: &str, signer: QuorumPublicKey) {
        let bytes = hex::decode(hex_key).unwrap();
        assert_eq!(QuorumPublicKey::try_from(bytes.as_slice()).unwrap(), signer);
        assert_eq!(hex_key.parse::<QuorumPublicKey>().unwrap(), signer);
        assert_eq!(signer.as_ref(), bytes);
        assert!(signer.encrypt_public_key().is_ok());
        assert!(signer.verifying_key().is_ok());
    }

    #[test]
    fn production_signer_conversions_preserve_bytes() {
        assert_signer_key(
            TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY,
            QuorumPublicKey::production_signer(),
        );
    }

    #[test]
    fn preprod_signer_conversions_preserve_bytes() {
        assert_signer_key(
            TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY,
            QuorumPublicKey::preprod_signer(),
        );
    }

    #[test]
    fn dev_signer_conversions_preserve_bytes() {
        let key = QuorumPublicKey::dev_signer();
        assert_ne!(key, QuorumPublicKey::production_signer());
        assert_ne!(key, QuorumPublicKey::preprod_signer());
        assert_signer_key(TURNKEY_DEV_SIGNER_QUORUM_PUBLIC_KEY, key);
    }

    #[test]
    fn from_str_accepts_mixed_case() {
        let (encryption, signing) = TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY
            .split_at(2 * UNCOMPRESSED_POINT_BYTE_LENGTH);
        let mixed = format!("{}{signing}", encryption.to_ascii_uppercase());
        assert_eq!(
            mixed.parse::<QuorumPublicKey>().unwrap(),
            QuorumPublicKey::production_signer()
        );
    }

    #[test]
    fn try_from_rejects_incorrect_length() {
        assert_eq!(
            QuorumPublicKey::try_from([0; 64].as_slice()).unwrap_err(),
            EnclaveEncryptError::IncorrectQuorumPublicKeyBytesLength(64)
        );
    }

    #[test]
    fn try_from_rejects_invalid_encryption_point() {
        let mut bytes = hex::decode(TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY).unwrap();
        bytes[1..UNCOMPRESSED_POINT_BYTE_LENGTH].fill(0);
        assert_eq!(
            QuorumPublicKey::try_from(bytes.as_slice()).unwrap_err(),
            EnclaveEncryptError::InvalidEncryptionKey
        );
    }

    #[test]
    fn try_from_rejects_invalid_signing_point() {
        let mut bytes = hex::decode(TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY).unwrap();
        bytes[UNCOMPRESSED_POINT_BYTE_LENGTH + 1..].fill(0);
        assert_eq!(
            QuorumPublicKey::try_from(bytes.as_slice()).unwrap_err(),
            EnclaveEncryptError::InvalidVerifyingKeyBytes
        );
    }

    #[test]
    fn from_str_rejects_malformed_hex() {
        assert_eq!(
            "not hex!".parse::<QuorumPublicKey>().unwrap_err(),
            EnclaveEncryptError::HexDecode("Invalid character 'n' at position 0".to_string()),
        );
    }
}
