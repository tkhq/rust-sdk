//! errors for this crate
use thiserror::Error;

/// Errors for enclave encryption
#[allow(missing_docs)]
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EnclaveEncryptError {
    /// Receiver or Encap key is likely wrong.
    #[error("Failed to set up receiver context")]
    ReceiveCtxSetupFail,

    /// Something is likely wrong with the receiver public key.
    #[error("Failed to set up receiver context")]
    FailedToSetupSendCtx,

    #[error("Failed to encrypt plaintext")]
    FailedToEncrypt,

    #[error("Failed to decrypt ciphertext")]
    FailedToDecrypt,

    #[error("Failed to serialize data")]
    FailedToSerializeData,

    #[error("Failed to deserialize data")]
    FailedToDeserializeData,

    #[error("Could not verify the quorum key signature over the servers given the target public")]
    ServerTargetSignatureVerificationFail,

    #[error("Could not verify the quorum key signature over the encapsulation key")]
    ServerEncappedKeySignatureVerificationFail,

    #[error("Error while deserializing encapped key")]
    InvalidEncappedKey(hpke::HpkeError),

    #[error("Error while deserializing the server target key")]
    InvalidServerTarget(hpke::HpkeError),

    #[error("Error while deserializing the client target key")]
    InvalidClientTarget(hpke::HpkeError),

    #[error("Error while deserializing the server encapsulated key")]
    InvalidSeverEncappedKeySignature,

    #[error("Error while deserializing signature over the server target key")]
    InvalidServerTargetSignature,

    #[error("Count not use quorum secret as a valid signing key")]
    InvalidQuorumSecret,

    #[error("This server has already been used to decrypt a message")]
    ServerAlreadyUsedToDecrypt,

    #[error("This client has already been used to decrypt a message")]
    ClientAlreadyUsedToDecrypt,

    #[error("P256 public key could not be coerced into fixed length array")]
    InvalidP256PublicKeyLength,

    #[error("P256 signature could not be coerced into fixed length array")]
    InvalidP256SignatureLength,

    #[error("Could not deserialize P-256 public key: invalid SEC1 encoding")]
    InvalidP256PublicKeySec1Encoding(String),

    #[error("Invalid base58 encoding")]
    FailedToBase58Decode(String),

    #[error("Email recovery payload is shorter than")]
    InvalidEmailRecoveryPayload,

    #[error("Invalid enclave quorum public key")]
    InvalidEnclaveQuorumPublicKey,

    #[error("Invalid data version")]
    InvalidDataVersion,

    #[error("Organization from bundle does not match the expected organization ID")]
    InvalidOrganization,

    #[error("User from bundle does not match the expected user ID")]
    InvalidUser,

    #[error("Provided public key bytes are not sized correctly for a Quorum public key")]
    IncorrectQuorumPublicKeyBytesLength(usize),

    #[error("Error while decoding hex-encoded string: {0}")]
    HexDecode(String),

    #[error("Cannot create a verifying key from invalid")]
    InvalidVerifyingKeyBytes,

    #[error("Bytes contain invalid UTF-8")]
    InvalidUtf8Bytes(String),

    #[error("Invalid byte length for private key")]
    InvalidPrivateKeyByteLength,

    #[error("Cannot JSON-serialize bundle: {0}")]
    CannotSerializeBundle(String),
}
