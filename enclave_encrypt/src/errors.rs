//! errors for this crate

/// Errors for enclave encryption
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnclaveEncryptError {
    /// Receiver or Encap key is likely wrong.
    ReceiveCtxSetupFail,
    /// Something is likely wrong with the receiver public key.
    FailedToSetupSendCtx,
    /// Failed to encrypt the given plaintext.
    FailedToEncrypt,
    /// Failed to decrypt the given ciphertext.
    FailedToDecrypt,
    /// Failed to serialize the data from the server.
    FailedToSerializeData,
    /// Failed to deserialize the data from the server.
    FailedToDeserializeData,
    /// Could not verify the quorum key signature over the servers given
    /// target public key.
    ServerTargetSignatureVerificationFail,
    /// Could not verify the quorum key signature over the encapsulation key.
    ServerEncappedKeySignatureVerificationFail,
    /// Encapsulation key could not be deserialized.
    InvalidEncappedKey(hpke::HpkeError),
    /// Could not deserialize server's target key.
    InvalidServerTarget(hpke::HpkeError),
    /// Could not deserialize client target key.
    InvalidClientTarget(hpke::HpkeError),
    /// Could not deserialize server's encapsulated key.
    InvalidSeverEncappedKeySignature,
    /// Could not deserialize signature over server target key.
    InvalidServerTargetSignature,
    /// Could not use secret as signing key.
    InvalidQuorumSecret,
    /// Server target key has already been used to decrypt a message.
    ServerAlreadyUsedToDecrypt,
    /// Client target key has already been used to decrypt a message.
    ClientAlreadyUsedToDecrypt,
    /// P256 public key could not be coerced into fixed length array.
    InvalidP256PublicKeyLength,
    /// P256 signature could not be coerced into fixed length array.
    InvalidP256SignatureLength,
    /// Could not deserialize p256 public key as sec1 encoded.
    InvalidP256PublicKeySec1Encoding(String),
    /// Failed to decode a base58-encoded string.
    FailedToBase58Decode(String),
    /// Email recovery payload was shorter then expected.
    InvalidEmailRecoveryPayload,
    /// Invalid enclave quorum public key.
    InvalidEnclaveQuorumPublicKey,
    /// Invalid version of the data object sent from the server.
    InvalidDataVersion,
    /// Invalid organization ID in the data object sent from the server.
    InvalidOrganization,
    /// Invalid user ID in the data object sent from the server.
    InvalidUser,
    /// The provided public key bytes aren't sized correctly.
    IncorrectQuorumPublicKeyBytesLength(usize),
    /// Error while decoding hex bytes
    HexDecode(String),
    /// Invalid bytes were used to create a `VerifyingKey`
    InvalidVerifyingKeyBytes,
    /// Invalid Utf8 bytes
    InvalidUtf8Bytes(String),
    /// Invalid exported private key -- does not start with 0x...
    InvalidExportedPrivateKey,
    /// Invalid private key length
    InvalidPrivateKeyByteLength,
    /// Unable to serialize encrypted bundle (JSON serialization)
    CannotSerializeBundle(String),
}
