//! Contains the definition and data for Turnkey Quorum key public keys.
use p256::ecdsa::VerifyingKey;

use crate::errors::EnclaveEncryptError;

/// Turnkey's production signer quorum public key
/// Key export/import bundles are signed by this key.
pub const TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY: &str = "04ca7c0d624c75de6f34af342e87a21e0d8c83efd1bd5b5da0c0177c147f744fba6f01f9f37356f9c617659aafa55f6e0af8d169a8f054d153ab3201901fb63ecb04cf288fe433cc4e1aa0ce1632feac4ea26bf2f5a09dcfe5a42c398e06898710330f0572882f4dbdf0f5304b8fc8703acd69adca9a4bbf7f5d00d20a5e364b2569";

/// Turnkey's preprod signer quorum public key
/// We check it in here for convenience, for internal Turnkey employees. Goes without saying: do not use in production code!
pub const TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY: &str = "048e92f6cdcc0b375505980a298d9b79201db1f08b1f135360d2864af1a67186ec0dbeb570d396a456226b0844be93dbc0180abbf7e2e4c9cfde8d5da4e3f8a49004f3422b8afbe425d6ece77b8d2469954715a2ff273ab7ac89f1ed70e0a9325eaa1698b4351fd1b23734e65c0b6a86b62dd49d70b37c94606aac402cbd84353212";

/// Expected Quorum public key length: twice the length of a SEC1 uncompressed public key (65 * 2)
pub const QUORUM_PUBLIC_KEY_BYTE_LENGTH: usize = 130;

/// Represents a Quorum key public key component.
#[derive(Debug, PartialEq, Eq)]
pub struct QuorumPublicKey {
    bytes: Vec<u8>,
}

impl QuorumPublicKey {
    /// Create a new `QuorumPublicKey` from an array of bytes
    pub fn from_bytes<B: AsRef<[u8]>>(b: B) -> Result<Self, EnclaveEncryptError> {
        if b.as_ref().len() != QUORUM_PUBLIC_KEY_BYTE_LENGTH {
            return Err(EnclaveEncryptError::IncorrectQuorumPublicKeyBytesLength(
                b.as_ref().len(),
            ));
        }
        Ok(Self {
            bytes: b.as_ref().to_vec(),
        })
    }

    /// Create a new `QuorumPublicKey` from a str
    pub fn from_string<S: AsRef<str>>(s: S) -> Result<Self, EnclaveEncryptError> {
        let b =
            hex::decode(s.as_ref()).map_err(|e| EnclaveEncryptError::HexDecode(e.to_string()))?;
        Self::from_bytes(b)
    }

    /// Quorum Public Key for Turnkey's production signer.
    ///
    /// # Panics
    /// Not expected given the public keys are static and valid
    #[must_use]
    pub fn production_signer() -> Self {
        Self::from_string(TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY)
            .expect("static public key should be valid")
    }

    /// Quorum Public Key for Turnkey's preprod signer. Do not use in production code. See `production_signer`
    ///
    /// # Panics
    /// Not expected given the public keys are static and valid
    #[must_use]
    pub fn preprod_signer() -> Self {
        Self::from_string(TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY)
            .expect("static public key should be valid")
    }

    /// Returns a `VerifyingKey` for this `QuorumPublicKey`
    pub fn verifying_key(&self) -> Result<VerifyingKey, EnclaveEncryptError> {
        // Quorum public keys are actually 2 keys concatenated together
        // The first key is the encryption key, used to encrypt data to the enclave.
        // The second public key is the signing key, used to sign data coming out of the enclave, for authenticity.
        // The verifying key thus only uses the second part of the public key,
        VerifyingKey::from_sec1_bytes(&self.bytes[65..])
            .map_err(|_| EnclaveEncryptError::InvalidVerifyingKeyBytes)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn from_bytes_and_from_str_valid() {
        let prod_pub1 = QuorumPublicKey::from_bytes(
            hex::decode(TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY).unwrap(),
        )
        .unwrap();
        let prod_pub2 =
            QuorumPublicKey::from_string(TURNKEY_PRODUCTION_SIGNER_QUORUM_PUBLIC_KEY).unwrap();
        let prod_pub3 = QuorumPublicKey::production_signer();
        assert!(prod_pub1.verifying_key().is_ok());
        assert_eq!(prod_pub1, prod_pub2);
        assert_eq!(prod_pub2, prod_pub3);

        let preprod_pub1 = QuorumPublicKey::from_bytes(
            hex::decode(TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY).unwrap(),
        )
        .unwrap();
        let preprod_pub2 =
            QuorumPublicKey::from_string(TURNKEY_PREPROD_SIGNER_QUORUM_PUBLIC_KEY).unwrap();
        let preprod_pub3 = QuorumPublicKey::preprod_signer();
        assert!(preprod_pub1.verifying_key().is_ok());
        assert_eq!(preprod_pub1, preprod_pub2);
        assert_eq!(preprod_pub2, preprod_pub3);
    }

    #[test]
    fn from_bytes_invalid_length() {
        let bytes = vec![0u8; 64]; // too short
        assert_eq!(
            QuorumPublicKey::from_bytes(&bytes).unwrap_err(),
            EnclaveEncryptError::IncorrectQuorumPublicKeyBytesLength(64)
        );
    }

    #[test]
    fn from_str_valid() {
        const VALID_HEX_KEY: &str = concat!(
            "04",                                                               // uncompressed prefix
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899", // x
            "99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa", // y
            // second key
            "04", // uncompressed prefix
            "11223344556677889900aabbccddeeff11223344556677889900aabbccddeeff", // x
            "ffeeddccbbaa00998877665544332211ffeeddccbbaa00998877665544332211"  // y
        );
        let quorum_pub = QuorumPublicKey::from_string(VALID_HEX_KEY).unwrap();
        assert_eq!(
            quorum_pub.verifying_key().unwrap_err(),
            EnclaveEncryptError::InvalidVerifyingKeyBytes
        );
    }

    #[test]
    fn from_str_invalid_hex() {
        assert_eq!(
            QuorumPublicKey::from_string("not hex!").unwrap_err(),
            EnclaveEncryptError::HexDecode("Invalid character 'n' at position 0".to_string()),
        );
    }
}
