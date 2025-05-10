#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::unwrap_used)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use crate::errors::EnclaveEncryptError;

use std::ops::Deref;

use hpke::{Kem as KemTrait, OpModeR, OpModeS, Serializable};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use rand_core::OsRng;

pub mod client;
pub mod errors;
mod quorum_public_key;
pub mod server;

pub use client::AuthenticationClient;
pub use client::ExportClient;
pub use client::ImportClient;
pub use quorum_public_key::QuorumPublicKey;

/// See the [readme](README.md#hpke-configuration) for how to configure these value.
/// HPKE Key encapsulation mechanism
type Kem = hpke::kem::DhP256HkdfSha256;
/// HPKE Authenticated Encryption Scheme
type Aead = hpke::aead::AesGcm256;
/// HPKE Key Derivation Function
type Kdf = hpke::kdf::HkdfSha256;
/// HPKE info
const TURNKEY_HPKE_INFO: &[u8] = b"turnkey_hpke";
/// Version of data sent in messages from server.
const DATA_VERSION: &str = "v1.0.0";

fn compress_p256_public(uncompressed_public: &[u8]) -> Result<Box<[u8]>, EnclaveEncryptError> {
    Ok(p256::PublicKey::from_sec1_bytes(uncompressed_public)
        .map_err(|e| EnclaveEncryptError::InvalidP256PublicKeySec1Encoding(e.to_string()))?
        .to_encoded_point(true)
        .to_bytes())
}

fn decompress_p256_public(compressed_public: &[u8]) -> Result<Box<[u8]>, EnclaveEncryptError> {
    Ok(p256::PublicKey::from_sec1_bytes(compressed_public)
        .map_err(|e| EnclaveEncryptError::InvalidP256PublicKeySec1Encoding(e.to_string()))?
        .to_encoded_point(false)
        .to_bytes())
}

/// Typed wrapper for p256 uncompressed public key bytes.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library by serializing/deserializing
/// between a hex-encoded string and public key bytes.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct P256Public(#[serde(with = "hex::serde")] pub [u8; 65]);
impl TryFrom<Vec<u8>> for P256Public {
    type Error = EnclaveEncryptError;
    fn try_from(vec: Vec<u8>) -> Result<Self, EnclaveEncryptError> {
        let inner = vec
            .try_into()
            .map_err(|_| EnclaveEncryptError::InvalidP256PublicKeyLength)?;
        Ok(Self(inner))
    }
}
impl Deref for P256Public {
    type Target = [u8; 65];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Typed wrapper for p256 signature bytes.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library by serializing/deserializing
/// between a hex-encoded string and signature bytes.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct P256Signature(#[serde(with = "hex::serde")] pub Vec<u8>);
impl From<Vec<u8>> for P256Signature {
    fn from(vec: Vec<u8>) -> Self {
        Self(vec)
    }
}
impl Deref for P256Signature {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Message from the server.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSendMsg {
    /// Version of the data.
    version: Option<String>,
}

/// Message from the server with encapsulated key, quorum key signature over
/// encapsulated key and ciphertext.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSendMsgV0 {
    /// Encapsulation key used to generate the ciphertext.
    encapped_public: P256Public,
    /// Quorum key signature over the encapsulation key.
    encapped_public_signature: P256Signature,
    /// Ciphertext from the server.
    /// This attribute serializes/deserializes between a hex-encoded string and bytes.
    #[serde(with = "hex::serde")]
    ciphertext: Vec<u8>,
}

/// Message from the server with data, the data's version, enclave quorum key, and the enclave
/// quorum key signature over the data.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSendMsgV1 {
    /// Version of the data.
    version: String,
    /// Data sent by the enclave server. This is also the message used to produce `data_signature`.
    #[serde(with = "hex::serde")]
    data: Vec<u8>,
    /// Enclave quorum key signature over the data.
    data_signature: P256Signature,
    /// Enclave quorum key public key.
    enclave_quorum_public: P256Public,
}

/// Data object from the server with the encapsulated public key, ciphertext,
/// and an organization ID.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSendData {
    /// Encapsulation key used to generate the ciphertext.
    encapped_public: P256Public,
    /// Ciphertext from the server.
    /// This attribute serializes/deserializes between a hex-encoded string and bytes.
    #[serde(with = "hex::serde")]
    ciphertext: Vec<u8>,
    /// Organization making the request
    organization_id: String,
}

/// Message from the server.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTargetMsg {
    /// Version of the data.
    version: Option<String>,
}

/// Message from the server with a encryption target key  and a quorum key
/// signature over it.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTargetMsgV0 {
    /// Target public key for client to encrypt to.
    pub target_public: P256Public,
    /// Signature over the servers public target key.
    pub target_public_signature: P256Signature,
}

/// Message from the server with data, the data's version, enclave quorum key, and the enclave
/// quorum key signature over the data.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTargetMsgV1 {
    /// Version of the data.
    pub version: String,
    /// Data sent by the enclave server.
    #[serde(with = "hex::serde")]
    pub data: Vec<u8>,
    /// Enclave quorum key signature over the data.
    pub data_signature: P256Signature,
    /// Enclave quorum key public key.
    pub enclave_quorum_public: P256Public,
}

/// Data object from the server with the target public key, organization ID,
/// and a user ID field.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTargetData {
    /// Target public key for client to encrypt to.
    pub target_public: P256Public,
    /// Organization making the request
    pub organization_id: String,
    /// User making the request
    pub user_id: String,
}

/// Message from the client containing ciphertext and the associated
/// encapsulated key.
/// These attributes make the struct interoperable with JSON objects created
/// by the Go implementation of this library.
#[derive(PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSendMsg {
    /// We assume this public key can be trusted because the request went through
    /// checks in the policy engine.
    encapped_public: P256Public,
    /// The encrypted message from the client.
    /// This attribute serializes/deserializes between a hex-encoded string and bytes.
    #[serde(with = "hex::serde")]
    ciphertext: Vec<u8>,
}

fn encrypt(
    receiver_public: &<Kem as KemTrait>::PublicKey,
    plaintext: &[u8],
    info_string: &[u8],
) -> Result<(Vec<u8>, <Kem as KemTrait>::EncappedKey), EnclaveEncryptError> {
    let (encapped_public, mut sender_ctx) = hpke::setup_sender::<Aead, Kdf, Kem, _>(
        &OpModeS::Base,
        receiver_public,
        info_string,
        &mut OsRng,
    )
    .map_err(|_| EnclaveEncryptError::FailedToSetupSendCtx)?;

    let aad = additional_associated_data(receiver_public, &encapped_public);
    let ciphertext = sender_ctx
        .seal(plaintext, &aad)
        .map_err(|_| EnclaveEncryptError::FailedToEncrypt)?;

    Ok((ciphertext, encapped_public))
}

fn decrypt(
    encapped_public: &<Kem as KemTrait>::EncappedKey,
    receiver_private: &<Kem as KemTrait>::PrivateKey,
    receiver_public: &<Kem as KemTrait>::PublicKey,
    ciphertext: &[u8],
    info_string: &[u8],
) -> Result<Vec<u8>, EnclaveEncryptError> {
    let mut receiver_ctx = hpke::setup_receiver::<Aead, Kdf, Kem>(
        &OpModeR::Base,
        receiver_private,
        encapped_public,
        info_string,
    )
    .map_err(|_| EnclaveEncryptError::ReceiveCtxSetupFail)?;

    let aad = additional_associated_data(receiver_public, encapped_public);
    receiver_ctx
        .open(ciphertext, &aad)
        .map_err(|_| EnclaveEncryptError::FailedToDecrypt)
}

fn additional_associated_data(
    receiver_public: &<Kem as KemTrait>::PublicKey,
    sender_public: &<Kem as KemTrait>::EncappedKey,
) -> Vec<u8> {
    sender_public
        .to_bytes()
        .iter()
        .chain(receiver_public.to_bytes().iter())
        .copied()
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};

    use crate::{client::EnclaveEncryptClient, server::EnclaveEncryptServer};

    use super::*;
    const FAKE_SEED: &[u8] = &[42; 32];

    fn quorum_pub() -> VerifyingKey {
        *SigningKey::from_bytes(FAKE_SEED).unwrap().verifying_key()
    }

    fn quorum_priv() -> SigningKey {
        SigningKey::from_bytes(FAKE_SEED).unwrap()
    }
    fn random_signature() -> P256Signature {
        let key = quorum_priv();
        let signature: Signature = key.sign(b"random");
        signature.to_der().to_bytes().to_vec().into()
    }

    // Returns a realistic secret byte vec: an API private key (P256)
    fn example_credential() -> Vec<u8> {
        hex::decode("67ee05fc3bdf4161bc70701c221d8d77180294cefcfcea64ba83c4d4c732fcb9").unwrap()
    }

    #[test]
    fn client_to_server_e2e() {
        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let user_id = "ef7e305c-f085-4a32-accf-939d8373f2ac";
        let server = EnclaveEncryptServer::from_enclave_auth_key(
            quorum_priv(),
            organization_id.to_string(),
            Some(user_id.to_string()),
        );
        // Message with the servers encryption target
        let server_target = server.publish_target().unwrap();
        let server_target_bytes = serde_json::to_vec(&server_target).unwrap();
        // Persist server receiving side
        let mut server_recv = server.into_recv();

        let client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        let client_ciphertext = client
            .encrypt(
                &example_credential(),
                &server_target_bytes,
                organization_id,
                user_id,
            )
            .unwrap();

        assert_eq!(
            server_recv.decrypt(&client_ciphertext).unwrap(),
            example_credential()
        );
        assert_eq!(
            server_recv.decrypt(&client_ciphertext),
            Err(EnclaveEncryptError::ServerAlreadyUsedToDecrypt)
        );
    }

    #[test]
    fn client_to_server_e2e_existing_target_key() {
        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let user_id = "ef7e305c-f085-4a32-accf-939d8373f2ac";
        let server = EnclaveEncryptServer::from_enclave_auth_key(
            quorum_priv(),
            organization_id.to_string(),
            Some(user_id.to_string()),
        );
        // Message with the servers encryption target
        let server_target = server.publish_target().unwrap();
        let server_target_bytes = serde_json::to_vec(&server_target).unwrap();
        // Persist server receiving side
        let mut server_recv = server.into_recv();

        let (target_private, target_public) = Kem::gen_keypair(&mut OsRng);
        let client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            quorum_pub(),
            target_public,
            target_private,
        );
        let client_ciphertext = client
            .encrypt(
                &example_credential(),
                &server_target_bytes,
                organization_id,
                user_id,
            )
            .unwrap();

        assert_eq!(
            server_recv.decrypt(&client_ciphertext).unwrap(),
            example_credential()
        );
        assert_eq!(
            server_recv.decrypt(&client_ciphertext),
            Err(EnclaveEncryptError::ServerAlreadyUsedToDecrypt)
        );
    }

    #[test]
    fn client_to_server_reject_bad_server_target_signature() {
        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let user_id = "ef7e305c-f085-4a32-accf-939d8373f2ac";
        let server = EnclaveEncryptServer::from_enclave_auth_key(
            quorum_priv(),
            organization_id.to_string(),
            Some(user_id.to_string()),
        );

        let mut server_target = server.publish_target().unwrap();
        server_target.data_signature = random_signature();
        let server_target_bytes = serde_json::to_vec(&server_target).unwrap();

        let client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        assert_eq!(
            client.encrypt(
                &example_credential(),
                &server_target_bytes,
                organization_id,
                user_id
            ),
            Err(EnclaveEncryptError::ServerTargetSignatureVerificationFail),
        );
    }

    #[test]
    fn server_to_client_e2e() {
        let mut client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        let client_target = client.target().unwrap();

        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let (target_private, target_public) = Kem::gen_keypair(&mut OsRng);
        let server = EnclaveEncryptServer::from_enclave_auth_key_and_target_key(
            quorum_priv(),
            target_public,
            target_private,
            organization_id.to_string(),
            None,
        );
        let server_ciphertext = server
            .encrypt(&client_target, &example_credential())
            .unwrap();
        let server_ciphertext_bytes = serde_json::to_vec(&server_ciphertext).unwrap();
        assert_eq!(
            client
                .decrypt(&server_ciphertext_bytes, organization_id)
                .unwrap(),
            example_credential()
        );

        assert_eq!(
            client.decrypt(&server_ciphertext_bytes, organization_id),
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        );
    }

    #[test]
    fn server_to_client_e2e_existing_target_key() {
        let mut client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        let client_target = client.target().unwrap();

        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let server = EnclaveEncryptServer::from_enclave_auth_key(
            quorum_priv(),
            organization_id.to_string(),
            None,
        );
        let server_ciphertext = server
            .encrypt(&client_target, &example_credential())
            .unwrap();
        let server_ciphertext_bytes = serde_json::to_vec(&server_ciphertext).unwrap();

        assert_eq!(
            client
                .decrypt(&server_ciphertext_bytes, organization_id)
                .unwrap(),
            example_credential()
        );

        assert_eq!(
            client.decrypt(&server_ciphertext_bytes, organization_id),
            Err(EnclaveEncryptError::ClientAlreadyUsedToDecrypt)
        );
    }

    #[test]
    fn server_to_client_reject_bad_encapped_public_signature() {
        let mut client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        let client_target = client.target().unwrap();

        let organization_id = "b676ee7c-7eb4-47f1-8e1c-ff0e68e376cd";
        let server = EnclaveEncryptServer::from_enclave_auth_key(
            quorum_priv(),
            organization_id.to_string(),
            None,
        );
        let mut server_ciphertext = server
            .encrypt(&client_target, &example_credential())
            .unwrap();
        server_ciphertext.data_signature = random_signature();
        let server_ciphertext_bytes = serde_json::to_vec(&server_ciphertext).unwrap();

        assert_eq!(
            client.decrypt(&server_ciphertext_bytes, organization_id),
            Err(EnclaveEncryptError::ServerEncappedKeySignatureVerificationFail)
        );
    }

    // This test helps with debugging iframe code as well as isolating encrypt functionality.
    // If you need a recovery or auth bundle, place your public key in here and print the payload.
    #[test]
    fn produce_valid_base58check_auth_bundles() {
        let target_public_key = P256Public(hex::decode("04e866df39454a9942a110834b42d7ef50c2442bd625aa3b99af8fb665039d91e885a731d75f2f8cf1c9cc0e0e6720cf632e1a85182d39ac48f6efa551d5250733").unwrap().try_into().unwrap());
        let payload =
            EnclaveEncryptServer::auth_encrypt(&target_public_key, &example_credential()).unwrap();
        assert!(bs58::decode(payload).with_check(None).into_vec().is_ok());
    }

    #[test]
    fn auth_encrypt_e2e() {
        let mut client = EnclaveEncryptClient::from_enclave_auth_key(quorum_pub());
        let client_target = client.target().unwrap();

        let payload =
            EnclaveEncryptServer::auth_encrypt(&client_target, &example_credential()).unwrap();
        println!("{}", payload);
        assert_eq!(client.auth_decrypt(&payload).unwrap(), example_credential());
    }

    // This might seem too specific but is necessary to test the base58 encoder we use.
    // If something changes when the crate is updated, affecting the payload, we should know about it.
    // It ensures the crate returns sane results for known test vectors.
    #[test]
    fn base58_payload_encoding() {
        assert_eq!(
            bs58::encode(hex::decode("04305e2b2473f058").unwrap()).into_string(),
            // Test vector obtained from https://www.better-converter.com/Encoders-Decoders/Base58Check-to-Hexadecimal-Decoder
            "he11owor1d".to_string(),
        );
        assert_eq!(
            bs58::encode(
                hex::decode("0062E907B15CBF27D5425399EBF6F0FB50EBB88F18C29B7D93").unwrap()
            )
            .into_string(),
            // Satoshi's wallet
            // Test vector obtained from http://lenschulwitz.com/base58
            "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string(),
        );
    }
}
