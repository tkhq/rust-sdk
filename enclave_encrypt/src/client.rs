//! Enclave Encrypt Client
use hpke::{Deserializable, Kem as KemTrait, Serializable};
use p256::{
    ecdsa::{signature::Verifier, DerSignature, VerifyingKey},
    PublicKey,
};
use rand_core::OsRng;

use crate::{
    decompress_p256_public, decrypt, encrypt, errors::EnclaveEncryptError, ClientSendMsg, Kem,
    P256Public, ServerSendData, ServerSendMsg, ServerSendMsgV0, ServerSendMsgV1, ServerTargetData,
    ServerTargetMsg, ServerTargetMsgV0, ServerTargetMsgV1, DATA_VERSION, TURNKEY_HPKE_INFO,
};

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
