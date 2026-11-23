#![doc = include_str!("../README.md")]
use std::fs;
use std::path::Path;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use k256::ecdsa::{
    Signature as K256Signature, SigningKey as K256SigningKey, VerifyingKey as K256VerifyingKey,
};
use p256::{
    SecretKey,
    ecdsa::{
        Signature as P256Signature, SigningKey as P256SigningKey, VerifyingKey as P256VerifyingKey,
        signature::RandomizedSigner as _,
    },
};
use rand_core::OsRng;
use serde::Serialize;
use thiserror::Error;

pub const SIGNATURE_SCHEME_P256: &str = "SIGNATURE_SCHEME_TK_API_P256";
pub const SIGNATURE_SCHEME_SECP256K1: &str = "SIGNATURE_SCHEME_TK_API_SECP256K1";
pub const API_KEY_STAMP_HEADER_NAME: &str = "X-Stamp";
pub const SECP256K1_PRIVATE_KEY_SIZE: usize = 32;

#[derive(Error, Debug, PartialEq)]
pub enum StamperError {
    #[error("cannot load private key bytes: {0}")]
    InvalidPrivateKeyBytes(String),

    #[error("cannot load public key bytes: {0}")]
    InvalidPublicKeyBytes(String),

    #[error("public key mismatch. Expected {0}, got {1}")]
    PublicKeyMismatch(String, String),

    #[error("cannot open file at {0}: {1}")]
    Io(String, String),

    #[error("cannot decode hex: {0}")]
    HexDecode(String),

    #[error("randomized signing failed")]
    RandomizedSigningFailed,
}

/// A stamp header to attach to an HTTP request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StampHeader {
    pub name: String,
    pub value: String,
}

/// Generic stamper that operates over raw bytes and returns an HTTP header.
pub trait Stamp {
    /// Sign the supplied request body and return the HTTP header that authenticates it.
    fn stamp(&self, body: &[u8]) -> Result<StampHeader, StamperError>;
}

/// Marks stampers that produce a fresh signature for each signing attempt.
///
/// Each successful call to [`Stamp::stamp`] must produce a fresh signature,
/// even when the request bytes and key are identical. The stamp must use the
/// Turnkey `X-Stamp` format with a P-256 or secp256k1 API key.
///
/// This gives each attempt a different fingerprint so you can check that
/// the proof in the response matches the specific request you sent.
pub trait UniqueStamp: Stamp {}

/// Represents a Turnkey API key using the P-256 curve.
#[derive(Debug, PartialEq)]
pub struct TurnkeyP256ApiKey {
    signing_key: P256SigningKey,
}

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TurnkeyApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
}

impl TurnkeyP256ApiKey {
    /// Generates a new random API key.
    pub fn generate() -> Self {
        let signing_key = P256SigningKey::random(&mut OsRng);
        Self { signing_key }
    }

    /// Creates a new API key from private key bytes.
    /// Optionally takes in public key bytes to check against, after derivation.
    /// If the derived public key doesn't match the passed in value, `PublicKeyMismatch` is returned.
    pub fn from_bytes<B: AsRef<[u8]>>(
        private_key_bytes: B,
        public_key_bytes: Option<B>,
    ) -> Result<Self, StamperError> {
        let secret_key = SecretKey::from_bytes(private_key_bytes.as_ref().into())
            .map_err(|e| StamperError::InvalidPrivateKeyBytes(e.to_string()))?;
        let signing_key = P256SigningKey::from(secret_key);

        if let Some(pub_bytes) = public_key_bytes {
            let expected = P256VerifyingKey::from_sec1_bytes(pub_bytes.as_ref())
                .map_err(|e| StamperError::InvalidPublicKeyBytes(e.to_string()))?;
            let actual = signing_key.verifying_key();
            if expected != *actual {
                return Err(StamperError::PublicKeyMismatch(
                    hex::encode(expected.to_encoded_point(true).as_bytes()),
                    hex::encode(actual.to_encoded_point(true).as_bytes()),
                ));
            }
        }

        Ok(Self { signing_key })
    }

    /// Load an API key from hex-encoded strings
    /// Optionally takes in public a public key to check against, after derivation.
    /// If the derived public key doesn't match the passed in value, `PublicKeyMismatch` is returned.
    pub fn from_strings<S: AsRef<str>>(
        private_key: S,
        public_key: Option<S>,
    ) -> Result<Self, StamperError> {
        let private_key_bytes = hex::decode(private_key.as_ref())
            .map_err(|e| StamperError::HexDecode(e.to_string()))?;

        let public_key_bytes = if let Some(public_key) = public_key {
            Some(
                hex::decode(public_key.as_ref())
                    .map_err(|e| StamperError::HexDecode(e.to_string()))?,
            )
        } else {
            None
        };

        Self::from_bytes(private_key_bytes, public_key_bytes)
    }

    /// Helper to create an API from pre-existing files. This is useful if you used
    /// the Turnkey CLI (<https://github.com/tkhq/tkcli>) to generate keys.
    pub fn from_files<P: AsRef<Path>, Q: AsRef<Path>>(
        private_key_path: P,
        public_key_path: Q,
    ) -> Result<Self, StamperError> {
        let private_key = fs::read_to_string(private_key_path.as_ref()).map_err(|e| {
            StamperError::Io(
                private_key_path.as_ref().display().to_string(),
                e.to_string(),
            )
        })?;
        let public_key = fs::read_to_string(public_key_path.as_ref()).map_err(|e| {
            StamperError::Io(
                public_key_path.as_ref().display().to_string(),
                e.to_string(),
            )
        })?;

        Self::from_strings(private_key, Some(public_key))
    }

    /// Returns the compressed public key bytes
    pub fn compressed_public_key(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }

    /// Returns the private key bytes
    pub fn private_key(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }
}

impl Stamp for TurnkeyP256ApiKey {
    /// Signs the request body with fresh randomness so each submission attempt
    /// gets its own fingerprint, even when the request bytes are identical.
    ///
    /// # Panics
    ///
    /// Panics if the operating system cannot provide randomness.
    fn stamp(&self, body: &[u8]) -> Result<StampHeader, StamperError> {
        let sig: P256Signature = self
            .signing_key
            .try_sign_with_rng(&mut OsRng, body)
            .map_err(|_| StamperError::RandomizedSigningFailed)?;
        encode_stamp_header(
            &self.compressed_public_key(),
            sig.to_der().as_bytes(),
            SIGNATURE_SCHEME_P256,
        )
    }
}

impl UniqueStamp for TurnkeyP256ApiKey {}

/// Formats a request signature as a Turnkey `X-Stamp` header.
/// The value is base64url-encoded JSON containing the hex-encoded public key,
/// hex-encoded DER signature, and signature scheme.
fn encode_stamp_header(
    public_key: &[u8],
    signature: &[u8],
    scheme: &str,
) -> Result<StampHeader, StamperError> {
    let stamp = TurnkeyApiStamp {
        public_key: hex::encode(public_key),
        signature: hex::encode(signature),
        scheme: scheme.to_string(),
    };
    let json = serde_json::to_vec(&stamp).map_err(|_| StamperError::RandomizedSigningFailed)?;
    Ok(StampHeader {
        name: API_KEY_STAMP_HEADER_NAME.to_string(),
        value: BASE64_URL_SAFE_NO_PAD.encode(json),
    })
}

/// Represents a Turnkey API key using the **secp256k1** curve.
#[derive(Debug, PartialEq)]
pub struct TurnkeySecp256k1ApiKey {
    signing_key: K256SigningKey,
}

impl TurnkeySecp256k1ApiKey {
    /// Generates a new Secp256k1 API key.
    pub fn generate() -> Self {
        let signing_key = K256SigningKey::random(&mut OsRng);
        Self { signing_key }
    }

    /// Creates a new API key from private key bytes.
    pub fn from_bytes<B: AsRef<[u8]>>(
        private_key_bytes: B,
        public_key_bytes: Option<B>,
    ) -> Result<Self, StamperError> {
        let priv_bytes = private_key_bytes.as_ref();

        // Ensure the private key is exactly 32 bytes, as required by K256SigningKey::from_bytes().
        if priv_bytes.len() != SECP256K1_PRIVATE_KEY_SIZE {
            return Err(StamperError::InvalidPrivateKeyBytes(format!(
                "expected {} bytes, got {}",
                SECP256K1_PRIVATE_KEY_SIZE,
                priv_bytes.len(),
            )));
        }

        let signing_key = K256SigningKey::from_bytes(priv_bytes.into())
            .map_err(|e| StamperError::InvalidPrivateKeyBytes(e.to_string()))?;

        if let Some(pub_bytes) = public_key_bytes {
            let expected = K256VerifyingKey::from_sec1_bytes(pub_bytes.as_ref())
                .map_err(|e| StamperError::InvalidPublicKeyBytes(e.to_string()))?;
            let actual = signing_key.verifying_key();
            if expected != *actual {
                return Err(StamperError::PublicKeyMismatch(
                    hex::encode(expected.to_encoded_point(true).as_bytes()),
                    hex::encode(actual.to_encoded_point(true).as_bytes()),
                ));
            }
        }

        Ok(Self { signing_key })
    }

    /// Load an API key from hex-encoded strings (with optional public key check).
    pub fn from_strings<S: AsRef<str>>(
        private_key: S,
        public_key: Option<S>,
    ) -> Result<Self, StamperError> {
        let private_key_bytes = hex::decode(private_key.as_ref())
            .map_err(|e| StamperError::HexDecode(e.to_string()))?;

        let public_key_bytes = if let Some(pk) = public_key {
            Some(hex::decode(pk.as_ref()).map_err(|e| StamperError::HexDecode(e.to_string()))?)
        } else {
            None
        };

        Self::from_bytes(private_key_bytes, public_key_bytes)
    }

    /// Returns compressed SEC1 public key bytes
    pub fn compressed_public_key(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }
}

impl Stamp for TurnkeySecp256k1ApiKey {
    /// Signs the request body with fresh randomness so each submission attempt
    /// gets its own fingerprint, even when the request bytes are identical.
    ///
    /// # Panics
    ///
    /// Panics if the operating system cannot provide randomness.
    fn stamp(&self, body: &[u8]) -> Result<StampHeader, StamperError> {
        let sig: K256Signature = self
            .signing_key
            .try_sign_with_rng(&mut OsRng, body)
            .map_err(|_| StamperError::RandomizedSigningFailed)?;
        encode_stamp_header(
            &self.compressed_public_key(),
            sig.to_der().as_bytes(),
            SIGNATURE_SCHEME_SECP256K1,
        )
    }
}

impl UniqueStamp for TurnkeySecp256k1ApiKey {}

#[cfg(test)]
mod tests {
    use p256::ecdsa::signature::Verifier as _;
    use std::io::Write;

    use super::*;
    use k256::ecdsa::Signature as K256Signature;
    use serde_json::Value;
    use tempfile::NamedTempFile;

    #[test]
    fn encode_stamp_header_preserves_wire_fields() {
        let header = encode_stamp_header(
            &[0x00, 0xab, 0xcd, 0xef],
            &[0x30, 0x06, 0x02, 0x01, 0x0a, 0x02, 0x01, 0x0f],
            SIGNATURE_SCHEME_P256,
        )
        .unwrap();
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(header.value).unwrap();
        let json: Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(header.name, "X-Stamp");
        assert_eq!(
            json,
            serde_json::json!({
                "publicKey": "00abcdef",
                "signature": "300602010a02010f",
                "scheme": "SIGNATURE_SCHEME_TK_API_P256",
            })
        );
    }

    #[test]
    fn encode_stamp_header_is_unpadded_base64url() {
        let header = encode_stamp_header(&[0x00, 0xab], &[0x30], SIGNATURE_SCHEME_P256).unwrap();
        assert!(
            header
                .value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        );
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(header.value).unwrap();
        assert_ne!(decoded.len() % 3, 0);
        let json: Value = serde_json::from_slice(&decoded).unwrap();
        assert_eq!(json["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
    }

    #[test]
    fn p256_stamps_verify_and_use_distinct_signatures_for_the_same_body() {
        let key = TurnkeyP256ApiKey::generate();
        let public_key = key.compressed_public_key();
        assert_eq!(public_key.len(), 33);
        let body = b"hello";
        let signatures = [key.stamp(body).unwrap(), key.stamp(body).unwrap()].map(|header| {
            assert_eq!(header.name, "X-Stamp");
            let decoded = BASE64_URL_SAFE_NO_PAD.decode(header.value).unwrap();
            let stamp: Value = serde_json::from_slice(&decoded).unwrap();
            assert_eq!(stamp["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
            assert_eq!(stamp["publicKey"], hex::encode(&public_key));
            let bytes = hex::decode(stamp["signature"].as_str().unwrap()).unwrap();
            let signature = P256Signature::from_der(&bytes).unwrap();
            key.signing_key
                .verifying_key()
                .verify(body, &signature)
                .unwrap();
            signature
        });
        assert_ne!(signatures[0], signatures[1]);
    }

    #[test]
    fn test_secp256k1_from_bytes_with_incorrect_public_key() {
        let k1 = TurnkeySecp256k1ApiKey::generate();
        let k2 = TurnkeySecp256k1ApiKey::generate();

        let res = TurnkeySecp256k1ApiKey::from_bytes(
            k1.signing_key.to_bytes().to_vec(),
            Some(k2.compressed_public_key()),
        );

        assert!(matches!(res, Err(StamperError::PublicKeyMismatch(_, _))));
    }

    #[test]
    fn test_secp256k1_from_bytes_with_correct_public_key() {
        let k = TurnkeySecp256k1ApiKey::generate();

        let rebuilt = TurnkeySecp256k1ApiKey::from_bytes(
            k.signing_key.to_bytes().to_vec(),
            Some(k.compressed_public_key()),
        )
        .expect("from_bytes should succeed with matching pubkey");

        assert_eq!(rebuilt.compressed_public_key(), k.compressed_public_key());
    }

    #[test]
    fn test_secp256k1_from_bytes_with_bad_private_key_bytes() {
        // Wrong length (31 bytes) should fail key construction
        let res = TurnkeySecp256k1ApiKey::from_bytes(vec![0u8; 31], None);
        assert!(matches!(res, Err(StamperError::InvalidPrivateKeyBytes(_))));
    }

    #[test]
    fn test_secp256k1_from_bytes_with_bad_public_key_bytes() {
        let k = TurnkeySecp256k1ApiKey::generate();
        // 33 bytes, but not a valid SEC1-encoded point
        let bogus_pub = vec![0xFFu8; 33];

        let res =
            TurnkeySecp256k1ApiKey::from_bytes(k.signing_key.to_bytes().to_vec(), Some(bogus_pub));
        assert!(matches!(res, Err(StamperError::InvalidPublicKeyBytes(_))));
    }

    #[test]
    fn test_secp256k1_from_strings_with_correct_public_key() {
        let k = TurnkeySecp256k1ApiKey::generate();
        let priv_hex = hex::encode(k.signing_key.to_bytes());
        let pub_hex = hex::encode(k.compressed_public_key());

        // With explicit public key
        assert!(
            TurnkeySecp256k1ApiKey::from_strings(&priv_hex, Some(&pub_hex)).is_ok(),
            "from_strings should succeed with matching pubkey"
        );

        // Without explicit public key (no check)
        assert!(
            TurnkeySecp256k1ApiKey::from_strings(&priv_hex, None).is_ok(),
            "from_strings should also succeed without pubkey"
        );
    }

    #[test]
    fn test_secp256k1_from_strings_with_bad_hex() {
        // Bad private key hex
        assert!(matches!(
            TurnkeySecp256k1ApiKey::from_strings("zzzz", None).unwrap_err(),
            StamperError::HexDecode(_)
        ));

        // Bad public key hex
        let k = TurnkeySecp256k1ApiKey::generate();
        let priv_hex = hex::encode(k.signing_key.to_bytes());
        assert!(matches!(
            TurnkeySecp256k1ApiKey::from_strings(priv_hex, Some("public_key_string".to_string()))
                .unwrap_err(),
            StamperError::HexDecode(_)
        ));
    }

    #[test]
    fn test_secp256k1_compressed_pub_len_is_33() {
        let k = TurnkeySecp256k1ApiKey::generate();
        assert_eq!(k.compressed_public_key().len(), 33);
    }

    #[test]
    fn secp256k1_stamps_verify_and_use_distinct_signatures_for_the_same_body() {
        let key = TurnkeySecp256k1ApiKey::generate();
        let public_key = key.compressed_public_key();
        let body = b"hello-secp256k1";
        let signatures = [key.stamp(body).unwrap(), key.stamp(body).unwrap()].map(|header| {
            assert_eq!(header.name, "X-Stamp");
            let decoded = BASE64_URL_SAFE_NO_PAD.decode(header.value).unwrap();
            let stamp: Value = serde_json::from_slice(&decoded).unwrap();
            assert_eq!(stamp["scheme"], "SIGNATURE_SCHEME_TK_API_SECP256K1");
            assert_eq!(stamp["publicKey"], hex::encode(&public_key));
            let bytes = hex::decode(stamp["signature"].as_str().unwrap()).unwrap();
            let signature = K256Signature::from_der(&bytes).unwrap();
            key.signing_key
                .verifying_key()
                .verify(body, &signature)
                .unwrap();
            signature
        });
        assert_ne!(signatures[0], signatures[1]);
    }

    #[test]
    fn test_from_bytes_with_incorrect_public_key() {
        let key1 = TurnkeyP256ApiKey::generate();
        let key2 = TurnkeyP256ApiKey::generate();

        let res = TurnkeyP256ApiKey::from_bytes(
            key1.signing_key.to_bytes().to_vec(),
            Some(key2.compressed_public_key()),
        );
        assert_eq!(
            res.unwrap_err(),
            StamperError::PublicKeyMismatch(
                hex::encode(key2.compressed_public_key()), // expected
                hex::encode(key1.compressed_public_key()), // actual
            )
        );
    }

    #[test]
    fn test_from_bytes_with_correct_public_key() {
        let key = TurnkeyP256ApiKey::from_bytes(
            hex::decode("9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69")
                .unwrap(),
            Some(
                hex::decode("02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45")
                    .unwrap(),
            ),
        )
        .unwrap();
        assert_eq!(
            hex::encode(key.compressed_public_key()),
            "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45"
        );
    }

    #[test]
    fn test_from_bytes_with_bad_private_key_bytes() {
        let res = TurnkeyP256ApiKey::from_bytes(
            hex::decode("fffffffff61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c70")
                .unwrap(),
            None,
        );
        assert_eq!(
            res.unwrap_err(),
            StamperError::InvalidPrivateKeyBytes("crypto error".to_string())
        );
    }

    #[test]
    fn test_from_bytes_with_bad_public_key_bytes() {
        let res = TurnkeyP256ApiKey::from_bytes(
            hex::decode("9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69")
                .unwrap(),
            Some(
                hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                    .unwrap(),
            ),
        );
        assert_eq!(
            res.unwrap_err(),
            StamperError::InvalidPublicKeyBytes("signature error".to_string())
        );
    }

    #[test]
    fn test_from_strings_with_correct_public_key() {
        assert!(
            TurnkeyP256ApiKey::from_strings(
                "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69",
                None,
            )
            .is_ok()
        );

        assert!(
            TurnkeyP256ApiKey::from_strings(
                "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69",
                Some("02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45"),
            )
            .is_ok()
        );
    }

    #[test]
    fn test_from_strings_with_bad_hex() {
        assert_eq!(
            TurnkeyP256ApiKey::from_strings("97230", None,)
                .unwrap_err()
                .to_string(),
            "cannot decode hex: Odd number of digits".to_string()
        );

        assert_eq!(
            TurnkeyP256ApiKey::from_strings(
                "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69",
                Some("notvalid"),
            )
            .unwrap_err()
            .to_string(),
            "cannot decode hex: Invalid character 'n' at position 0".to_string()
        );
    }

    #[test]
    fn test_load_from_files() {
        let mut priv_file = NamedTempFile::new().unwrap();
        priv_file
            .write_all(
                "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69".as_bytes(),
            )
            .unwrap();

        let mut pub_file = NamedTempFile::new().unwrap();
        pub_file
            .write_all(
                "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45".as_bytes(),
            )
            .unwrap();

        let key = TurnkeyP256ApiKey::from_files(priv_file.path(), pub_file.path())
            .expect("from_files should succeed");

        assert_eq!(
            hex::encode(key.compressed_public_key()),
            "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45"
        );
    }

    #[test]
    fn test_load_from_files_with_bad_hex() {
        let mut priv_file = NamedTempFile::new().unwrap();
        priv_file.write_all("baad-private-key".as_bytes()).unwrap();

        let mut pub_file = NamedTempFile::new().unwrap();
        pub_file
            .write_all(
                "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45".as_bytes(),
            )
            .unwrap();

        let res = TurnkeyP256ApiKey::from_files(priv_file.path(), pub_file.path());
        assert_eq!(
            res.unwrap_err(),
            StamperError::HexDecode("Invalid character '-' at position 4".to_string())
        );
    }

    #[test]
    fn test_load_from_files_with_bad_file_paths() {
        let mut priv_file = NamedTempFile::new().unwrap();
        priv_file
            .write_all(
                "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69".as_bytes(),
            )
            .unwrap();

        let mut pub_file = NamedTempFile::new().unwrap();
        pub_file
            .write_all(
                "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45".as_bytes(),
            )
            .unwrap();

        // Try loading from a non-existant private key file
        let err1 = TurnkeyP256ApiKey::from_files("/tmp/does/not/exist/key.priv", pub_file.path())
            .unwrap_err();
        assert_eq!(
            err1.to_string(),
            "cannot open file at /tmp/does/not/exist/key.priv: No such file or directory (os error 2)"
        );
        assert_eq!(
            err1,
            StamperError::Io(
                "/tmp/does/not/exist/key.priv".to_string(),
                "No such file or directory (os error 2)".to_string()
            )
        );

        // Do the same with a bogus public key file
        let err2 = TurnkeyP256ApiKey::from_files(priv_file.path(), "/tmp/does/not/exist/key.pub")
            .unwrap_err();
        assert_eq!(
            err2.to_string(),
            "cannot open file at /tmp/does/not/exist/key.pub: No such file or directory (os error 2)"
        );
        assert_eq!(
            err2,
            StamperError::Io(
                "/tmp/does/not/exist/key.pub".to_string(),
                "No such file or directory (os error 2)".to_string()
            )
        );
    }
}
