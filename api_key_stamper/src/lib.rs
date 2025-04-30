use std::fs;
use std::path::{Path, PathBuf};

use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{Signature, SigningKey, VerifyingKey};
use p256::SecretKey;
use rand_core::OsRng;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum StamperError {
    #[error("cannot load private key bytes: {0}")]
    InvalidPrivateKeyBytes(String),

    #[error("cannot load public key bytes: {0}")]
    InvalidPublicKeyBytes(String),

    #[error("public key mismatch. Expected {0}, got {1}")]
    PublicKeyMismatch(String, String),

    #[error("cannot open file at {0}: {1}")]
    Io(PathBuf, String),

    #[error("cannot decode hex: {0}")]
    HexDecode(String),
}

/// Represents a Turnkey API key using the P-256 curve.
#[derive(Debug, PartialEq)]
pub struct TurnkeyP256ApiKey {
    signing_key: SigningKey,
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
        let secret_key = SecretKey::random(&mut OsRng);
        let signing_key = SigningKey::from(secret_key);
        Self { signing_key }
    }

    /// Creates a new API key from private key bytes.
    /// Optionally takes in public key bytes to check against, after derivation.
    pub fn from_bytes<B: AsRef<[u8]>>(
        private_key_bytes: B,
        public_key_bytes: Option<B>,
    ) -> Result<Self, StamperError> {
        let secret_key = SecretKey::from_bytes(private_key_bytes.as_ref().into())
            .map_err(|e| StamperError::InvalidPrivateKeyBytes(e.to_string()))?;
        let signing_key = SigningKey::from(secret_key);

        if let Some(pub_bytes) = public_key_bytes {
            let expected = VerifyingKey::from_sec1_bytes(pub_bytes.as_ref())
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

    /// Helper to create an API from pre-existing files. This is useful if you used
    /// the Turnkey CLI (<https://github.com/tkhq/tkcli>) to generate keys.
    pub fn from_files<P: AsRef<Path>, Q: AsRef<Path>>(
        private_key_path: P,
        public_key_path: Q,
    ) -> Result<Self, StamperError> {
        let private_key = fs::read_to_string(private_key_path.as_ref()).map_err(|e| {
            StamperError::Io(private_key_path.as_ref().to_path_buf(), e.to_string())
        })?;
        let public_key = fs::read_to_string(public_key_path.as_ref())
            .map_err(|e| StamperError::Io(public_key_path.as_ref().to_path_buf(), e.to_string()))?;

        let private_key_hex_bytes =
            hex::decode(private_key).map_err(|e| StamperError::HexDecode(e.to_string()))?;
        let public_key_hex_bytes =
            hex::decode(public_key).map_err(|e| StamperError::HexDecode(e.to_string()))?;

        Self::from_bytes(private_key_hex_bytes, Some(public_key_hex_bytes))
    }

    /// Function to produce a base64-encoded stamp.
    /// See <https://docs.turnkey.com/developer-reference/api-overview/stamps#api-keys> for more information.
    pub fn stamp<S: AsRef<str>>(&self, request_body: S) -> Result<String, StamperError> {
        let sig: Signature = self.signing_key.sign(request_body.as_ref().as_bytes());

        let stamp = TurnkeyApiStamp {
            public_key: hex::encode(self.compressed_public_key()),
            signature: hex::encode(sig.to_der()),
            scheme: "SIGNATURE_SCHEME_TK_API_P256".to_string(),
        };

        let json_stamp = serde_json::to_string(&stamp).unwrap();

        Ok(BASE64_URL_SAFE_NO_PAD.encode(json_stamp.as_bytes()))
    }

    /// Returns the compressed public key bytes
    pub fn compressed_public_key(&self) -> Vec<u8> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes()
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use p256::ecdsa::signature::Verifier;
    use std::io::Write;

    use super::*;
    use serde_json::Value;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_and_stamp() {
        let key = TurnkeyP256ApiKey::generate();
        let pub_key = key.compressed_public_key();
        assert_eq!(pub_key.len(), 33); // Compressed key

        // Test that we can provide a str or a String. Both should work!
        assert!(key.stamp("hello").is_ok());
        assert!(key.stamp(String::from("hello")).is_ok());

        // Test that a produced stamp is valid
        let stamp = key.stamp("Hi from TKHQ");

        let decoded_stamp_bytes = BASE64_URL_SAFE_NO_PAD
            .decode(stamp.unwrap())
            .expect("stamp should be valid base64");
        let decoded_stamp_string = String::from_utf8(decoded_stamp_bytes)
            .expect("stamp bytes should be valid UTF8 characters");

        // The resulting string should be valid JSON
        let json_stamp: Value =
            serde_json::from_str(&decoded_stamp_string).expect("stamp should be valid JSON");

        // The signature scheme and public key should be correct
        assert_eq!(json_stamp["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
        assert_eq!(
            json_stamp["publicKey"],
            hex::encode(key.compressed_public_key())
        );

        // And finally: the signature should verify!
        let verifying_key = key.signing_key.verifying_key();

        // Will be hex-encoded DER bytes ("30...")
        let sig_hex_string = json_stamp["signature"]
            .as_str()
            .expect("signature field should contain a string");

        let sig_bytes =
            hex::decode(sig_hex_string).expect("signature should contain valid hex-encoded bytes");
        let sig = Signature::from_der(&sig_bytes).expect("signature bytes should be valid DER");
        assert!(verifying_key.verify(b"Hi from TKHQ", &sig).is_ok());
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
        assert_eq!(err1.to_string(), "cannot open file at /tmp/does/not/exist/key.priv: No such file or directory (os error 2)");
        assert_eq!(
            err1,
            StamperError::Io(
                PathBuf::from("/tmp/does/not/exist/key.priv"),
                "No such file or directory (os error 2)".to_string()
            )
        );

        // Do the same with a bogus public key file
        let err2 = TurnkeyP256ApiKey::from_files(priv_file.path(), "/tmp/does/not/exist/key.pub")
            .unwrap_err();
        assert_eq!(err2.to_string(), "cannot open file at /tmp/does/not/exist/key.pub: No such file or directory (os error 2)");
        assert_eq!(
            err2,
            StamperError::Io(
                PathBuf::from("/tmp/does/not/exist/key.pub"),
                "No such file or directory (os error 2)".to_string()
            )
        );
    }
}
