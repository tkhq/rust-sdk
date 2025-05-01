//! Enclave Encryption Server
use hpke::{Deserializable, Kem as KemTrait, Serializable};
use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use rand_core::OsRng;

use crate::{
    compress_p256_public, decrypt, encrypt, errors::EnclaveEncryptError, ClientSendMsg, Kem,
    P256Public, ServerSendData, ServerSendMsgV1, ServerTargetData, ServerTargetMsgV1, DATA_VERSION,
    TURNKEY_HPKE_INFO,
};

/// An instance of the server side for `EnclaveEncrypt`. This should only be used for either
/// a SINGLE send or a single receive.
pub struct EnclaveEncryptServer {
    // The underlying type is zero on drop
    enclave_auth_key: SigningKey,
    // The underlying type is zero on drop
    target_public: <Kem as KemTrait>::PublicKey,
    target_private: <Kem as KemTrait>::PrivateKey,
    // Organization
    organization_id: String,
    // User
    user_id: Option<String>,
}

impl EnclaveEncryptServer {
    /// This should be the quorum signing secret derived from the quorum
    /// master seed.
    #[must_use]
    pub fn from_enclave_auth_key(
        enclave_auth_key: SigningKey,
        organization_id: String,
        user_id: Option<String>,
    ) -> Self {
        let (target_private, target_public) = Kem::gen_keypair(&mut OsRng);
        Self {
            enclave_auth_key,
            target_public,
            target_private,
            organization_id,
            user_id,
        }
    }

    /// Create a server from the enclave quorum public key and the target key.
    #[must_use]
    pub fn from_enclave_auth_key_and_target_key(
        enclave_auth_key: SigningKey,
        target_public_key: <Kem as KemTrait>::PublicKey,
        target_private_key: <Kem as KemTrait>::PrivateKey,
        organization_id: String,
        user_id: Option<String>,
    ) -> Self {
        Self {
            enclave_auth_key,
            target_public: target_public_key,
            target_private: target_private_key,
            organization_id,
            user_id,
        }
    }

    /// Encrypt a message to the `client_target_public` key.
    pub fn encrypt(
        &self,
        client_target: &P256Public,
        plaintext: &[u8],
    ) -> Result<ServerSendMsgV1, EnclaveEncryptError> {
        let client_target = <Kem as KemTrait>::PublicKey::from_bytes(&**client_target)
            .map_err(EnclaveEncryptError::InvalidClientTarget)?;
        let (ciphertext, encapped_public) = encrypt(&client_target, plaintext, TURNKEY_HPKE_INFO)?;

        let data = ServerSendData {
            encapped_public: encapped_public.to_bytes().to_vec().try_into()?,
            ciphertext,
            organization_id: self.organization_id.clone(),
        };
        let data_bytes = serde_json::to_string(&data)
            .map_err(|_| EnclaveEncryptError::FailedToSerializeData)?
            .into_bytes();
        let data_signature: Signature = self.enclave_auth_key.sign(&data_bytes);
        let enclave_quorum_public_bytes = self
            .enclave_auth_key
            .verifying_key()
            .to_encoded_point(false)
            .to_bytes()
            .to_vec();
        Ok(ServerSendMsgV1 {
            version: DATA_VERSION.to_string(),
            data: data_bytes,
            data_signature: data_signature.to_der().to_bytes().to_vec().into(),
            enclave_quorum_public: enclave_quorum_public_bytes.try_into()?,
        })
    }

    /// Encrypt `plaintext`, returning a payload that omits the enclave auth key signature. This
    /// should only be used for email recovery and auth since other use cases will want to verify enclave
    /// authentication. Enclave authentication doesn't matter for email recovery or auth because an
    /// inauthentic ciphertext will just result in being unable to register a new authenticator.
    ///
    /// The returned payload has has the goal of minimizing payload size.
    /// In email recovery and auth we don't need to verify the payload came from the
    /// enclave, so we forego the enclave auth signature. Additionally we use base64 encoding.
    ///
    /// The payload is in the format of `CompressedEncappedPublicKey||Ciphertext`.
    pub fn auth_encrypt(
        client_target: &P256Public,
        plaintext: &[u8],
    ) -> Result<String, EnclaveEncryptError> {
        let client_target = <Kem as KemTrait>::PublicKey::from_bytes(&**client_target)
            .map_err(EnclaveEncryptError::InvalidClientTarget)?;
        let (ciphertext, encapped_public) = encrypt(&client_target, plaintext, TURNKEY_HPKE_INFO)?;

        let encapped_public_bytes = encapped_public.to_bytes().to_vec();
        let compressed_encapped_public = compress_p256_public(&encapped_public_bytes)?;

        let payload_bytes: Vec<_> = compressed_encapped_public
            .iter()
            .copied()
            .chain(ciphertext)
            .collect();

        Ok(bs58::encode(&payload_bytes).with_check().into_string())
    }

    /// Return the servers encryption target key and a signature over it from
    /// the quorum key.
    pub fn publish_target(&self) -> Result<ServerTargetMsgV1, EnclaveEncryptError> {
        let user_id = self
            .user_id
            .as_ref()
            .ok_or(EnclaveEncryptError::InvalidUser)?
            .to_string();
        let data = ServerTargetData {
            target_public: self.target_public.to_bytes().to_vec().try_into()?,
            organization_id: self.organization_id.clone(),
            user_id,
        };
        let data_bytes = serde_json::to_string(&data)
            .map_err(|_| EnclaveEncryptError::FailedToSerializeData)?
            .into_bytes();
        let data_signature: Signature = self.enclave_auth_key.sign(&data_bytes);
        let enclave_quorum_public_bytes = self
            .enclave_auth_key
            .verifying_key()
            .to_encoded_point(false)
            .to_bytes()
            .to_vec();
        Ok(ServerTargetMsgV1 {
            version: DATA_VERSION.to_string(),
            data: data_bytes,
            data_signature: data_signature.to_der().to_bytes().to_vec().into(),
            enclave_quorum_public: enclave_quorum_public_bytes.try_into()?,
        })
    }

    /// Convert into `EnclaveEncryptServerRecv`. We expect `EnclaveEncryptServerRecv` will be serialized,
    /// encrypted, and persisted in org data while waiting for the client to send a message.
    #[must_use]
    pub fn into_recv(self) -> EnclaveEncryptServerRecv {
        EnclaveEncryptServerRecv {
            target_private: Some(self.target_private),
            target_public: Some(self.target_public),
        }
    }
}

/// The receiving side of the server. After the server published its target key,
/// this should be serialized & encrypted; later being decrypted and used to
/// decrypt the clients message.
///
/// N.B. Encrypt may only be called once; after encrypt is called the target key pair is wiped.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct EnclaveEncryptServerRecv {
    // The underlying type is zero on drop
    target_private: Option<<Kem as KemTrait>::PrivateKey>,
    target_public: Option<<Kem as KemTrait>::PublicKey>,
}

impl EnclaveEncryptServerRecv {
    /// Decrypt a message from a client.
    ///
    /// *N.B.* We assume the authenticity of the message contents is verified
    /// out of band in the Ump policy engine.
    pub fn decrypt(&mut self, msg: &ClientSendMsg) -> Result<Vec<u8>, EnclaveEncryptError> {
        let encapped_public = <Kem as KemTrait>::EncappedKey::from_bytes(&*msg.encapped_public)
            .map_err(EnclaveEncryptError::InvalidEncappedKey)?;

        if let (Some(target_private), Some(target_public)) =
            (self.target_private.as_ref(), self.target_public.as_ref())
        {
            let result = decrypt(
                &encapped_public,
                target_private,
                target_public,
                &msg.ciphertext,
                TURNKEY_HPKE_INFO,
            );

            self.target_public = None;
            self.target_private = None;

            result
        } else {
            Err(EnclaveEncryptError::ServerAlreadyUsedToDecrypt)
        }
    }
}
