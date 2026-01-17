//! Enclave Encrypt Client
use hpke::{Deserializable, Kem as KemTrait, Serializable};
use p256::{
    ecdsa::{signature::Verifier, DerSignature, SigningKey, VerifyingKey},
    PublicKey,
};
use rand_core::OsRng;
use std::str::from_utf8;

use crate::{
    decompress_p256_public, decrypt, encrypt, errors::EnclaveEncryptError,
    quorum_public_key::QuorumPublicKey, ClientSendMsg, Kem, P256Public, ServerSendData,
    ServerSendMsg, ServerSendMsgV0, ServerSendMsgV1, ServerTargetData, ServerTargetMsg,
    ServerTargetMsgV0, ServerTargetMsgV1, DATA_VERSION, TURNKEY_HPKE_INFO,
};

/// Expected length (in bytes) for imported private keys
const EXPECTED_PRIVATE_KEY_BYTE_LENGTH: usize = 32;

/// Abstraction over `EnclaveEncryptClient` for authentication flows.
pub struct AuthenticationClient {
    encrypt_client: EnclaveEncryptClient,
}

impl Default for AuthenticationClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthenticationClient {
    /// Creates a new `AuthenticationClient` with a fresh target key pair.
    pub fn new() -> Self {
        // A VerifyingKey is required to construct `EnclaveEncryptClient`, but unused when decrypting auth bundles.
        // This is safe not to verify auth bundles authenticity, because these bundles contain credential private key material.
        // If these bundles are bogus or tampered with they would contain "bad" bytes, which implies the decrypted bytes would either not result in a valid credential, or result in a credential that is not capable to sign Turnkey activities.
        let random_key = SigningKey::random(&mut OsRng);
        let encrypt_client =
            EnclaveEncryptClient::from_enclave_auth_key(*random_key.verifying_key());
        Self { encrypt_client }
    }

    /// Creates a new `AuthenticationClient` from known private bytes.
    ///
    /// Note that these private bytes are NOT the same as the private key bytes of the target key pair.
    /// The private bytes are used as ikm (input key material) and used to derive a key pair.
    ///
    /// The size of the input key material SHOULD be >=32 bytes, so we enforce this here and return an error if the ikm is not long enough.
    ///
    /// WARNING: This interface can be misused if the same key material is provided over and over.
    /// Indeed, enclave-to-end-user secure channels assume that encryption is "one-shot". After decryption is done, the key should not be reused.
    /// You may use this interface to allow for ikm persistence and loading if it's not realistic or convenient to hold ikm in memory between initialization and decryption.
    pub fn dangerous_from_bytes<B: AsRef<[u8]>>(private: B) -> Self {
        let random_key = SigningKey::random(&mut OsRng);
        let (pair_private_key, pair_public_key) = Kem::derive_keypair(private.as_ref());
        let encrypt_client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            *random_key.verifying_key(),
            pair_public_key,
            pair_private_key,
        );
        Self { encrypt_client }
    }

    /// Returns the target public key bytes, encoded as SEC1 bytes (04 || X || Y)
    /// This value is returned as a string, ready to be inserted in auth-related init activities.
    pub fn target_public_key(&self) -> Result<String, EnclaveEncryptError> {
        Ok(hex::encode(self.encrypt_client.target_bytes()?))
    }

    /// Decrypts an authentication bundle and returns the credential bytes (raw private key bytes)
    pub fn decrypt<S: AsRef<str>>(
        &mut self,
        auth_bundle: S,
    ) -> Result<Vec<u8>, EnclaveEncryptError> {
        self.encrypt_client.auth_decrypt(auth_bundle.as_ref())
    }
}

/// Abstraction over `EnclaveEncryptClient` for private key or wallet export flows.
pub struct ExportClient {
    encrypt_client: EnclaveEncryptClient,
}

impl ExportClient {
    /// Creates a new Export client. Takes in a Quorum public key (use `QuorumPublicKey::production_signer()`)
    ///
    /// # Panics
    /// Not expected, unless you are using an invalid quorum key
    #[must_use]
    pub fn new(quorum_public_key: &QuorumPublicKey) -> Self {
        let verifying_key = quorum_public_key
            .verifying_key()
            .expect("quorum public key should yield a valid verifying key");
        let encrypt_client = EnclaveEncryptClient::from_enclave_auth_key(verifying_key);
        Self { encrypt_client }
    }

    /// Note that these private bytes are NOT the same as the private key bytes of the target key pair.
    /// The private bytes are used as ikm (input key material) and used to derive a key pair.
    ///
    /// The size of the input key material SHOULD be >=32 bytes, so we enforce this here and return an error if the ikm is not long enough.
    ///
    /// WARNING: This interface can be misused if the same key material is provided over and over.
    /// Indeed, enclave-to-end-user secure channels assume that encryption is "one-shot". After decryption is done, the key should not be reused.
    /// You may use this interface to allow for ikm persistence and loading if it's not realistic or convenient to hold ikm in memory between initialization and decryption.
    ///
    /// # Panics
    /// Not expected, unless you are using an invalid quorum key
    pub fn dangerous_from_bytes<B: AsRef<[u8]>>(
        private: B,
        quorum_public_key: &QuorumPublicKey,
    ) -> Self {
        let (pair_private_key, pair_public_key) = Kem::derive_keypair(private.as_ref());
        let encrypt_client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            quorum_public_key
                .verifying_key()
                .expect("quorum public key should yield a valid verifying key"),
            pair_public_key,
            pair_private_key,
        );
        Self { encrypt_client }
    }

    /// Returns the target public key bytes, encoded as SEC1 bytes (04 || X || Y)
    ///
    /// This value is returned as a string, ready to be inserted in `EXPORT_WALLET` or `EXPORT_PRIVATE_KEY` activity params.
    pub fn target_public_key(&self) -> Result<String, EnclaveEncryptError> {
        Ok(hex::encode(self.encrypt_client.target_bytes()?))
    }

    /// Decrypts a private key bundle.
    ///
    /// Bundles are JSON encoded strings, e.g. "{\"version\":\"v1.0.0\",\"data\":\"7b22656e63617070656450...\"}"
    ///
    /// This function returns the raw private key bytes
    pub fn decrypt_private_key<S: AsRef<str>, T: AsRef<str>>(
        &mut self,
        export_bundle: S,
        organization_id: T,
    ) -> Result<Vec<u8>, EnclaveEncryptError> {
        let decrypted_bytes = self
            .encrypt_client
            .decrypt(export_bundle.as_ref().as_bytes(), organization_id.as_ref())?;
        // We expect 32 bytes exactly
        if decrypted_bytes.len() != EXPECTED_PRIVATE_KEY_BYTE_LENGTH {
            return Err(EnclaveEncryptError::InvalidPrivateKeyByteLength);
        }
        Ok(decrypted_bytes)
    }

    /// Decrypts a private key bundle.
    /// Bundles are JSON encoded strings, e.g. "{\"version\":\"v1.0.0\",\"data\":\"7b22656e63617070656450...\"}"
    /// This function returns the mnemonic phrase as a string.
    pub fn decrypt_wallet_mnemonic_phrase<S: AsRef<str>, T: AsRef<str>>(
        &mut self,
        export_bundle: S,
        organization_id: T,
    ) -> Result<String, EnclaveEncryptError> {
        let decrypted_bytes = self
            .encrypt_client
            .decrypt(export_bundle.as_ref().as_bytes(), organization_id.as_ref())?;
        // The text should be "whatever word goes here etc..." (valid UTF-8 bytes)
        let phrase = from_utf8(&decrypted_bytes)
            .map_err(|e| EnclaveEncryptError::InvalidUtf8Bytes(e.to_string()))?;
        Ok(phrase.to_string())
    }
}

/// Abstraction over `EnclaveEncryptClient` for private key or wallet import flows.
pub struct ImportClient {
    encrypt_client: EnclaveEncryptClient,
}

impl ImportClient {
    /// Creates a new Import client. Takes in a Quorum public key (use `QuorumPublicKey::production_signer()`)
    ///
    /// # Panics
    /// Not expected, unless you are using an invalid quorum key
    #[must_use]
    pub fn new(quorum_public_key: &QuorumPublicKey) -> Self {
        let verifying_key = quorum_public_key
            .verifying_key()
            .expect("quorum public key should yield a valid verifying key");
        let encrypt_client = EnclaveEncryptClient::from_enclave_auth_key(verifying_key);
        Self { encrypt_client }
    }

    /// Note that these private bytes are NOT the same as the private key bytes of the target key pair.
    /// The private bytes are used as ikm (input key material) and used to derive a key pair.
    ///
    /// The size of the input key material SHOULD be >=32 bytes, so we enforce this here and return an error if the ikm is not long enough.
    ///
    /// WARNING: This interface can be misused if the same key material is provided over and over.
    /// Indeed, enclave-to-end-user secure channels assume that encryption is "one-shot". After decryption is done, the key should not be reused.
    /// You may use this interface to allow for ikm persistence and loading if it's not realistic or convenient to hold ikm in memory between initialization and decryption.
    ///
    /// # Panics
    /// Not expected, unless you are using an invalid quorum key
    pub fn dangerous_from_bytes<B: AsRef<[u8]>>(
        private: B,
        quorum_public_key: &QuorumPublicKey,
    ) -> Self {
        let (pair_private_key, pair_public_key) = Kem::derive_keypair(private.as_ref());
        let encrypt_client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            quorum_public_key
                .verifying_key()
                .expect("quorum public key should yield a valid verifying key"),
            pair_public_key,
            pair_private_key,
        );
        Self { encrypt_client }
    }

    /// Encrypts a private key to the public key contained in an import bundle.
    ///
    /// - `private_key` is the key material (bytes) to import. Must be 32 bytes in length.
    /// - `import_bundle` is the import bundle as a string. Bundles are JSON-encoded strings (e.g ""{\"version\":\"v1.0.0\", ....")
    ///   bundles contain a signed public key. The signature over this public key is from Turnkey's signer enclave.
    /// - `organization_id` is the expected organization ID. This will be checked against the content of the bundle, which contains the organization ID where the import flow started (`INIT_IMPORT` activity)
    /// - `user_id` is the expected user ID. This will be checked against the content of the bundle, which contains the user ID who initiated import (`INIT_IMPORT`)
    ///
    /// Returns a string containing the JSON-encoded value, ready-to-use in an `IMPORT_PRIVATE_KEY` activity
    pub fn encrypt_private_key_with_bundle<
        B: AsRef<[u8]>,
        S: AsRef<str>,
        T: AsRef<str>,
        U: AsRef<str>,
    >(
        &mut self,
        private_key: B,
        import_bundle: S,
        organization_id: T,
        user_id: U,
    ) -> Result<String, EnclaveEncryptError> {
        if private_key.as_ref().len() != EXPECTED_PRIVATE_KEY_BYTE_LENGTH {
            return Err(EnclaveEncryptError::InvalidPrivateKeyByteLength);
        }

        let encrypted = self.encrypt_client.encrypt(
            private_key.as_ref(),
            import_bundle.as_ref().as_bytes(),
            organization_id.as_ref(),
            user_id.as_ref(),
        )?;

        serde_json::to_string(&encrypted)
            .map_err(|e| EnclaveEncryptError::CannotSerializeBundle(e.to_string()))
    }

    /// Encrypts a wallet mnemonic phrase to the public key contained in an import bundle.
    ///
    /// - `mnemonic_phrase` is the wallet mnemonic phrase to import, as a string.
    /// - `import_bundle` is the import bundle as a string. Bundles are JSON-encoded strings (e.g ""{\"version\":\"v1.0.0\", ....")
    ///   bundles contain a signed public key. The signature over this public key is from Turnkey's signer enclave.
    /// - `organization_id` is the expected organization ID. This will be checked against the content of the bundle, which contains the organization ID where the import flow started (`INIT_IMPORT` activity)
    /// - `user_id` is the expected user ID. This will be checked against the content of the bundle, which contains the user ID who initiated import (`INIT_IMPORT`)
    ///
    /// Returns a string containing the JSON-encoded value, ready-to-use in an `IMPORT_WALLET` activity
    pub fn encrypt_wallet_with_bundle<
        S: AsRef<str>,
        T: AsRef<str>,
        U: AsRef<str>,
        V: AsRef<str>,
    >(
        &mut self,
        mnemonic_phrase: S,
        import_bundle: T,
        organization_id: U,
        user_id: V,
    ) -> Result<String, EnclaveEncryptError> {
        let encrypted = self.encrypt_client.encrypt(
            mnemonic_phrase.as_ref().as_bytes(),
            import_bundle.as_ref().as_bytes(),
            organization_id.as_ref(),
            user_id.as_ref(),
        )?;

        serde_json::to_string(&encrypted)
            .map_err(|e| EnclaveEncryptError::CannotSerializeBundle(e.to_string()))
    }
}

/// An instance of the client side for `EnclaveEncrypt`. This should only be used for either
/// a SINGLE send or a single receive.
///
/// Use `AuthenticationClient`, `ExportClient` or `ImportClient` for safer interfaces.
pub struct EnclaveEncryptClient {
    enclave_auth_key: VerifyingKey,
    target_public: Option<<Kem as KemTrait>::PublicKey>,
    // The underlying type is zero on drop
    target_private: Option<<Kem as KemTrait>::PrivateKey>,
}

impl EnclaveEncryptClient {
    /// Create a client from the quorum public key.
    #[must_use]
    pub fn from_enclave_auth_key(enclave_auth_key: VerifyingKey) -> Self {
        let (target_private, target_public) = Kem::gen_keypair(&mut OsRng);
        Self {
            enclave_auth_key,
            target_public: Some(target_public),
            target_private: Some(target_private),
        }
    }

    /// Create a client from the quorum public key and the target key.
    #[must_use]
    pub fn from_enclave_auth_key_and_target_key(
        enclave_auth_key: VerifyingKey,
        target_public_key: <Kem as KemTrait>::PublicKey,
        target_private_key: <Kem as KemTrait>::PrivateKey,
    ) -> Self {
        Self {
            enclave_auth_key,
            target_public: Some(target_public_key),
            target_private: Some(target_private_key),
        }
    }

    /// Get the encryption target of the client
    pub fn target(&self) -> Result<P256Public, EnclaveEncryptError> {
        if let Some(target_public) = self.target_public.as_ref() {
            target_public.to_bytes().to_vec().try_into()
        } else {
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        }
    }

    /// Get the encryption target bytes for this client
    pub fn target_bytes(&self) -> Result<Vec<u8>, EnclaveEncryptError> {
        if let Some(target_public) = self.target_public.as_ref() {
            Ok(target_public.to_bytes().to_vec())
        } else {
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        }
    }

    /// Encrypt a message to the given server target.
    #[allow(clippy::unused_self)]
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        msg_bytes: &[u8],
        organization_id: &str,
        user_id: &str,
    ) -> Result<ClientSendMsg, EnclaveEncryptError> {
        let ciphertext;
        let encapped_public: <Kem as KemTrait>::EncappedKey;

        let msg: ServerTargetMsg = serde_json::from_slice(msg_bytes)
            .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

        match msg.version.as_ref() {
            None => {
                let msg_v0: ServerTargetMsgV0 = serde_json::from_slice(msg_bytes)
                    .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

                let signature = DerSignature::try_from(&msg_v0.target_public_signature[..])
                    .map_err(|_| EnclaveEncryptError::InvalidServerTargetSignature)?;

                self.enclave_auth_key
                    .verify(&*msg_v0.target_public, &signature)
                    .map_err(|_| EnclaveEncryptError::ServerTargetSignatureVerificationFail)?;

                let receiver_public =
                    <Kem as KemTrait>::PublicKey::from_bytes(&*msg_v0.target_public)
                        .map_err(EnclaveEncryptError::InvalidServerTarget)?;
                (ciphertext, encapped_public) =
                    encrypt(&receiver_public, plaintext, TURNKEY_HPKE_INFO)?;
            }
            Some(s) if s.as_str() == DATA_VERSION => {
                let msg_v1: ServerTargetMsgV1 = serde_json::from_slice(msg_bytes)
                    .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

                let signature = DerSignature::try_from(&msg_v1.data_signature[..])
                    .map_err(|_| EnclaveEncryptError::InvalidServerTargetSignature)?;

                let enclave_quorum_public = {
                    let public = PublicKey::from_sec1_bytes(&*msg_v1.enclave_quorum_public)
                        .map_err(|_| EnclaveEncryptError::InvalidEnclaveQuorumPublicKey)?;
                    VerifyingKey::from(public)
                };

                if !enclave_quorum_public.eq(&self.enclave_auth_key) {
                    return Err(EnclaveEncryptError::InvalidEnclaveQuorumPublicKey);
                }

                let parsed_data = serde_json::from_slice::<ServerTargetData>(&msg_v1.data)
                    .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;
                enclave_quorum_public
                    .verify(&msg_v1.data, &signature)
                    .map_err(|_| EnclaveEncryptError::ServerTargetSignatureVerificationFail)?;

                if !parsed_data.organization_id.eq(organization_id) {
                    return Err(EnclaveEncryptError::InvalidOrganization);
                }

                if !parsed_data.user_id.eq(user_id) {
                    return Err(EnclaveEncryptError::InvalidUser);
                }

                let receiver_public =
                    <Kem as KemTrait>::PublicKey::from_bytes(&*parsed_data.target_public)
                        .map_err(EnclaveEncryptError::InvalidServerTarget)?;
                (ciphertext, encapped_public) =
                    encrypt(&receiver_public, plaintext, TURNKEY_HPKE_INFO)?;
            }
            Some(_) => return Err(EnclaveEncryptError::InvalidDataVersion),
        }

        Ok(ClientSendMsg {
            encapped_public: encapped_public.to_bytes().to_vec().try_into()?,
            ciphertext,
        })
    }

    /// Decrypt a message from the server targeted at this client.
    pub fn decrypt(
        &mut self,
        msg_bytes: &[u8],
        organization_id: &str,
    ) -> Result<Vec<u8>, EnclaveEncryptError> {
        let ciphertext;
        let encapped_public: <Kem as KemTrait>::EncappedKey;

        let msg: ServerSendMsg = serde_json::from_slice(msg_bytes)
            .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

        match msg.version.as_ref() {
            None => {
                let msg_v0: ServerSendMsgV0 = serde_json::from_slice(msg_bytes)
                    .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

                let signature = DerSignature::try_from(&msg_v0.encapped_public_signature[..])
                    .map_err(|_| EnclaveEncryptError::InvalidSeverEncappedKeySignature)?;

                self.enclave_auth_key
                    .verify(&*msg_v0.encapped_public, &signature)
                    .map_err(|_| EnclaveEncryptError::ServerEncappedKeySignatureVerificationFail)?;

                encapped_public =
                    <Kem as KemTrait>::EncappedKey::from_bytes(&*msg_v0.encapped_public)
                        .map_err(EnclaveEncryptError::InvalidEncappedKey)?;

                ciphertext = msg_v0.ciphertext;
            }
            Some(s) if s.as_str() == DATA_VERSION => {
                let msg_v1: ServerSendMsgV1 = serde_json::from_slice(msg_bytes)
                    .map_err(|_| EnclaveEncryptError::FailedToDeserializeData)?;

                let signature = DerSignature::try_from(&msg_v1.data_signature[..])
                    .map_err(|_| EnclaveEncryptError::InvalidSeverEncappedKeySignature)?;

                let enclave_quorum_public = {
                    let public = PublicKey::from_sec1_bytes(&*msg_v1.enclave_quorum_public)
                        .map_err(|_| EnclaveEncryptError::InvalidEnclaveQuorumPublicKey)?;
                    VerifyingKey::from(public)
                };

                if !enclave_quorum_public.eq(&self.enclave_auth_key) {
                    return Err(EnclaveEncryptError::InvalidEnclaveQuorumPublicKey);
                }

                let parsed_data = serde_json::from_slice::<ServerSendData>(&msg_v1.data)
                    .map_err(|_| EnclaveEncryptError::FailedToSerializeData)?;
                enclave_quorum_public
                    .verify(&msg_v1.data, &signature)
                    .map_err(|_| EnclaveEncryptError::ServerEncappedKeySignatureVerificationFail)?;

                if !parsed_data.organization_id.eq(organization_id) {
                    return Err(EnclaveEncryptError::InvalidOrganization);
                }

                encapped_public =
                    <Kem as KemTrait>::EncappedKey::from_bytes(&*parsed_data.encapped_public)
                        .map_err(EnclaveEncryptError::InvalidEncappedKey)?;

                ciphertext = parsed_data.ciphertext;
            }
            Some(_) => return Err(EnclaveEncryptError::InvalidDataVersion),
        }

        if let (Some(target_private), Some(target_public)) =
            (self.target_private.as_ref(), self.target_public.as_ref())
        {
            let result = decrypt(
                &encapped_public,
                target_private,
                target_public,
                &ciphertext,
                TURNKEY_HPKE_INFO,
            );

            self.target_public = None;
            self.target_private = None;

            result
        } else {
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        }
    }

    /// Decrypt a base64 serialized email recovery or auth payload.
    pub fn auth_decrypt(&mut self, payload: &str) -> Result<Vec<u8>, EnclaveEncryptError> {
        let payload_bytes = bs58::decode(payload)
            .with_check(None)
            .into_vec()
            .map_err(|e| {
                EnclaveEncryptError::FailedToBase58Decode(format!(
                    "error when decoding payload: {e:?}"
                ))
            })?;
        let raw_encapped = payload_bytes
            .get(0..33)
            .ok_or(EnclaveEncryptError::InvalidEmailRecoveryPayload)?;
        let ciphertext = payload_bytes
            .get(33..)
            .ok_or(EnclaveEncryptError::InvalidEmailRecoveryPayload)?;

        let encapped_public = {
            let encapped_public_bytes = decompress_p256_public(raw_encapped)?;
            <Kem as KemTrait>::EncappedKey::from_bytes(&encapped_public_bytes)
                .map_err(EnclaveEncryptError::InvalidEncappedKey)?
        };

        if let (Some(target_private), Some(target_public)) =
            (self.target_private.as_ref(), self.target_public.as_ref())
        {
            let result = decrypt(
                &encapped_public,
                target_private,
                target_public,
                ciphertext,
                TURNKEY_HPKE_INFO,
            );

            self.target_public = None;
            self.target_private = None;

            result
        } else {
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use crate::server::EnclaveEncryptServer;

    use super::*;

    fn example_credential() -> Vec<u8> {
        hex::decode("67ee05fc3bdf4161bc70701c221d8d77180294cefcfcea64ba83c4d4c732fcb9").unwrap()
    }

    // Sample Quorum private key which we use to simulate enclave signatures on bundles for tests below
    fn test_quorum_private_key() -> SigningKey {
        SigningKey::from_slice(
            &hex::decode("28ebf311b27f34cdf078489584d336423e09c522342f5b067dea36823c2cc5ed")
                .unwrap(),
        )
        .unwrap()
    }

    fn test_quorum_public_key() -> QuorumPublicKey {
        let quorum_public_key = concat!(
            // first key -- does not matter
            "04", // uncompressed prefix
            "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899", // x
            "99887766554433221100ffeeddccbbaa99887766554433221100ffeeddccbbaa", // y
            // our _actual_ signing key
            // generated with https://r-n-o.github.io/p256-keygen/
            "04", // uncompressed prefix
            "8ee67fa8ae8e5fac0e343c84fa0921ecb3a31a67aee2e6a0880a09072eaaf2ae", // x
            "ffa43f73fa021fa4d0b550072ba1f9011ff7cf917e4bf2708670e5ac57a81c78"  // y
        );
        QuorumPublicKey::from_string(quorum_public_key).unwrap()
    }

    #[test]
    fn test_test_quorum_key() {
        // Sanity check: are test_quorum_private_key and test_quorum_public_key consistent?
        let derived_public_key = test_quorum_private_key()
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        let public_key = test_quorum_public_key()
            .verifying_key()
            .unwrap()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        assert_eq!(derived_public_key, public_key);
    }

    #[test]
    fn produce_and_decrypt_auth_bundle() {
        let mut customer = AuthenticationClient::new();
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let auth_bundle = EnclaveEncryptServer::auth_encrypt(
            &customer_target.try_into().unwrap(),
            &example_credential(),
        )
        .unwrap();

        let decrypted = customer.decrypt(auth_bundle).unwrap();
        assert_eq!(decrypted, example_credential());
    }

    // This test shows how to decrypt an auth bundle.
    // We simulate the case of a client with no access to encryption keys who receives a bundle
    // encrypted to their public key.
    #[test]
    fn static_auth_bundle_decrypt() {
        // Hardcode a random client IKM to get stable test vectors across runs.
        let client_ikm =
            hex::decode("c8e5d3ccbf8c4e62e3bcb984681ce6dda950939905754902a394c1fc2b5c6a9e")
                .unwrap();
        let (client_private_key, client_public_key) = Kem::derive_keypair(&client_ikm);

        // Unlikely but check just in case: same IKM should result in the same private key
        assert_eq!(
            hex::encode(client_private_key.to_bytes()),
            "d2d9239a4bb25d09a6d91822e1d0991f0b21a63b102191bb98b6b77ac6fc6c91"
        );
        // And the same public key
        let expected_target_public_key = "0406f6d27ae62d66358b2b5888a8ccb2a0f4f1f86a5d2e9683b61c418b49e57df446b1518cb1c370e30ee80c61266a56b342b424b26c6b86419001a404d1d5fcc5";
        assert_eq!(
            hex::encode(client_public_key.to_bytes()),
            expected_target_public_key
        );
        assert_eq!(
            AuthenticationClient::dangerous_from_bytes(&client_ikm)
                .target_public_key()
                .unwrap(),
            expected_target_public_key
        );

        // We fix the bundle to a static value, to simulate the case where it's e.g. coming from an activity result or an email
        // This also ensures that future updates to this crate don't break old bundles!
        let bundle = "skr1QFHrNyL7xcvzRpU4t9yhL8rEVPZgDcFM5YW8S1YwLYjedoagnNZwMCyJsxNYzuphKHqQBkZxt4fLWSVsMqnW9XdmLBsK2MhwC5WxuxZD9xE8ezQ";
        let decrypted = AuthenticationClient::dangerous_from_bytes(client_ikm)
            .decrypt(bundle)
            .expect("decryption should succeed");

        // Assert that what we decrypted is what was originally encrypted: our "example_credential"
        assert_eq!(decrypted, example_credential());
    }

    #[test]
    fn produce_and_decrypt_private_key_export_bundle() {
        let mut customer = ExportClient::new(&test_quorum_public_key());
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );

        let encrypt_bundle = enclave
            .encrypt(&customer_target.try_into().unwrap(), &example_credential())
            .unwrap();

        // This is what is sent over the wire to the customer
        let encoded_bundle = serde_json::to_string(&encrypt_bundle).unwrap();

        // Decrypt the private key and get the same bytes than encrypted earlier
        assert_eq!(
            customer
                .decrypt_private_key(encoded_bundle, "org-id")
                .unwrap(),
            example_credential()
        );
    }

    #[test]
    fn produce_and_decrypt_wallet_export_bundle() {
        let mut customer = ExportClient::new(&test_quorum_public_key());
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );
        let mnemonic = "remember wallet wallet remember";
        let encrypt_bundle = enclave
            .encrypt(&customer_target.try_into().unwrap(), mnemonic.as_bytes())
            .unwrap();

        // This is what is sent over the wire to the customer
        let encoded_bundle = serde_json::to_string(&encrypt_bundle).unwrap();

        // Decrypt the private key and get the same bytes than encrypted earlier
        assert_eq!(
            customer
                .decrypt_wallet_mnemonic_phrase(encoded_bundle, "org-id")
                .unwrap(),
            mnemonic
        );
    }

    #[test]
    fn bad_private_key_export_bundles_fail_decryption() {
        let mut customer = ExportClient::new(&test_quorum_public_key());
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );

        let encrypt_bundle = enclave
            .encrypt(
                &customer_target.try_into().unwrap(),
                &hex::decode("12345678").unwrap(),
            )
            .unwrap();

        // Trying to decrypt a private key, but we only have 4 bytes. Not 32!
        assert_eq!(
            customer
                .decrypt_private_key(serde_json::to_string(&encrypt_bundle).unwrap(), "org-id")
                .unwrap_err(),
            EnclaveEncryptError::InvalidPrivateKeyByteLength,
        );
    }

    #[test]
    fn bad_private_wallet_bundles_fail_decryption() {
        let mut customer = ExportClient::new(&test_quorum_public_key());
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );
        let encrypt_bundle = enclave
            .encrypt(&customer_target.try_into().unwrap(), &example_credential())
            .unwrap();

        // Trying to decrypt a wallet, but we have non-utf8 bytes (we encrypted private key bytes!)
        assert_eq!(
            customer
                .decrypt_wallet_mnemonic_phrase(
                    serde_json::to_string(&encrypt_bundle).unwrap(),
                    "org-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidUtf8Bytes(
                "invalid utf-8 sequence of 1 bytes from index 1".to_string()
            ),
        );
    }

    #[test]
    fn export_bundle_decryption_fails_with_incorrect_org_id() {
        let mut customer = ExportClient::new(&test_quorum_public_key());
        let customer_target = hex::decode(customer.target_public_key().unwrap()).unwrap();

        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            None,
        );

        let encrypt_bundle = enclave
            .encrypt(&customer_target.try_into().unwrap(), &example_credential())
            .unwrap();

        assert_eq!(
            customer
                .decrypt_private_key(
                    serde_json::to_string(&encrypt_bundle).unwrap(),
                    "wrong-org-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidOrganization,
        );
        assert_eq!(
            customer
                .decrypt_wallet_mnemonic_phrase(
                    serde_json::to_string(&encrypt_bundle).unwrap(),
                    "wrong-org-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidOrganization,
        );
    }

    #[test]
    fn import_private_key() {
        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            Some("user-id".to_string()),
        );
        let import_bundle = enclave.publish_target().unwrap();
        let encoded_bundle = serde_json::to_string(&import_bundle).unwrap();

        let mut customer = ImportClient::new(&test_quorum_public_key());
        let encrypted_key = customer
            .encrypt_private_key_with_bundle(
                example_credential(),
                encoded_bundle,
                "org-id",
                "user-id",
            )
            .unwrap();

        let mut enclave_receiver = enclave.into_recv();
        assert_eq!(
            enclave_receiver
                .decrypt(&serde_json::from_str(&encrypted_key).unwrap())
                .unwrap(),
            example_credential(),
        );
    }

    #[test]
    fn import_private_key_fails_with_incorrect_metadata_or_bad_bytes() {
        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            Some("user-id".to_string()),
        );
        let import_bundle = enclave.publish_target().unwrap();
        let encoded_bundle = serde_json::to_string(&import_bundle).unwrap();

        let mut customer = ImportClient::new(&test_quorum_public_key());
        assert_eq!(
            customer
                .encrypt_private_key_with_bundle(
                    example_credential(),
                    &encoded_bundle,
                    "wrong-org-id",
                    "user-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidOrganization,
        );
        assert_eq!(
            customer
                .encrypt_private_key_with_bundle(
                    example_credential(),
                    &encoded_bundle,
                    "org-id",
                    "wrong-user-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidUser,
        );
        assert_eq!(
            customer
                .encrypt_private_key_with_bundle(
                    hex::decode("0123").unwrap(),
                    &encoded_bundle,
                    "org-id",
                    "user-id"
                )
                .unwrap_err(),
            EnclaveEncryptError::InvalidPrivateKeyByteLength,
        );
    }

    #[test]
    fn import_wallet() {
        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            Some("user-id".to_string()),
        );
        let import_bundle = enclave.publish_target().unwrap();
        let encoded_bundle = serde_json::to_string(&import_bundle).unwrap();

        let mut customer = ImportClient::new(&test_quorum_public_key());
        let mnemonic = "remember wallet wallet remember";
        let encrypted_key = customer
            .encrypt_wallet_with_bundle(mnemonic, encoded_bundle, "org-id", "user-id")
            .unwrap();

        let mut enclave_receiver = enclave.into_recv();
        assert_eq!(
            enclave_receiver
                .decrypt(&serde_json::from_str(&encrypted_key).unwrap())
                .unwrap(),
            mnemonic.as_bytes(),
        );
    }

    #[test]
    fn import_wallet_fails_with_incorrect_metadata() {
        let enclave = EnclaveEncryptServer::from_enclave_auth_key(
            test_quorum_private_key(),
            "org-id".to_string(),
            Some("user-id".to_string()),
        );
        let import_bundle = enclave.publish_target().unwrap();
        let encoded_bundle = serde_json::to_string(&import_bundle).unwrap();

        let mut customer = ImportClient::new(&test_quorum_public_key());
        let mnemonic = "wallet remember remember wallet";
        assert_eq!(
            customer
                .encrypt_wallet_with_bundle(mnemonic, &encoded_bundle, "wrong-org-id", "user-id")
                .unwrap_err(),
            EnclaveEncryptError::InvalidOrganization,
        );
        assert_eq!(
            customer
                .encrypt_wallet_with_bundle(mnemonic, &encoded_bundle, "org-id", "wrong-user-id")
                .unwrap_err(),
            EnclaveEncryptError::InvalidUser,
        );
    }
}
