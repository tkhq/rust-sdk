//! YubiKey device management for TVC operator keys.
//!
//! Concrete hardware access has three shapes: discovering connected devices
//! is an effect supplied by [`crate::output::Ctx`] (backed by
//! [`connected_serials`]), opening a serial is the ambient [`open`] entry
//! point, and per-device functionality hangs off the opened
//! [`yubikey::YubiKey`] through the [`DeviceOps`] extension trait. Commands
//! open a device once and drive its read, signing, and key-agreement operations
//! through that one handle.
//!
//! TVC expects externally generated P-256 keys in the PIV signing and
//! key-agreement slots, each under a self-signed `CN=QuorumOS` certificate,
//! with PIN and touch policies set to "always". Device mutations remain
//! explicit `ykman` operations: TVC never generates or deletes device keys or
//! certificates and never authenticates with a PIV management key.

use crate::config::turnkey::{QosOperatorPublicKey, QosOperatorPublicKeyParseError, YubiKeySerial};
use itertools::Itertools;
use p256::{
    PublicKey,
    ecdsa::{DerSignature, VerifyingKey, signature::Verifier},
    elliptic_curve::sec1::ToEncodedPoint,
};
use qos_client::{
    yubikey::{KEY_AGREEMENT_SLOT, SIGNING_SLOT, YubiKeyError},
    yubikey_crate as yubikey,
};
use rand_core::{OsRng, RngCore};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
    time::Duration,
};
use x509_cert::{
    builder::{Builder, CertificateBuilder, Profile},
    der::{Encode, referenced::OwnedToRef},
    name::Name,
    serial_number::SerialNumber,
    spki::SubjectPublicKeyInfoOwned,
    time::Validity,
};
use yubikey::{
    Error as PivError, PinPolicy, Serial, TouchPolicy, YubiKey,
    certificate::{Certificate, yubikey_signer::Signer},
    piv::{self, AlgorithmId, ManagementAlgorithmId, Origin, SlotId, SlotMetadata},
    reader::Context,
};
use zeroize::Zeroizing;

/// The subject TVC requires on the externally installed certificates.
// TODO: qos_client keeps this constant private; have it exported (along with
// a re-export of the yubikey crate) and use it here.
const QOS_CERTIFICATE_SUBJECT: &str = "CN=QuorumOS";

/// Equivalent to about ten years, matching the existing QuorumOS YubiKey
/// certificates.
const CERTIFICATE_VALIDITY_SECS: u64 = 10 * 60 * 60 * 24 * 365;

/// Serials of the connected YubiKeys, skipping readers without the PIV
/// applet. The real implementation behind the discovery effect on
/// [`crate::output::Ctx`].
pub(crate) fn connected_serials() -> Result<Vec<YubiKeySerial>, DeviceError> {
    let mut context = Context::open().map_err(DeviceError::Discovery)?;
    let readers = context.iter().map_err(DeviceError::Discovery)?;

    Itertools::try_collect(readers.filter_map(|reader| match reader.open() {
        Ok(yubikey) => Some(Ok(YubiKeySerial::from(yubikey.serial().0))),
        // The dependency cannot distinguish a non-YubiKey smartcard from
        // a YubiKey with PIV disabled; both report a missing PIV applet.
        Err(PivError::AppletNotFound { applet_name: "PIV" }) => None,
        Err(source) => Some(Err(DeviceError::OpenReader {
            reader: reader.name().into_owned(),
            source,
        })),
    }))
}

/// The connected YubiKey serials captured by one discovery pass.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ConnectedYubiKeys(Vec<YubiKeySerial>);

impl From<Vec<YubiKeySerial>> for ConnectedYubiKeys {
    fn from(serials: Vec<YubiKeySerial>) -> Self {
        Self(serials)
    }
}

impl ConnectedYubiKeys {
    /// Choose an explicit connected serial or the sole connected device.
    /// Several devices are refused because a serial-only prompt cannot
    /// identify which physical stick the user intends to touch.
    pub(crate) fn choose(
        self,
        explicit: Option<YubiKeySerial>,
    ) -> Result<YubiKeySerial, YubiKeySelectionError> {
        match explicit {
            Some(serial) if self.0.contains(&serial) => Ok(serial),
            Some(requested) => Err(YubiKeySelectionError::NotConnected {
                requested,
                connected: self,
            }),
            None => match self.0.as_slice() {
                [] => Err(YubiKeySelectionError::NoneConnected),
                [sole] => Ok(*sole),
                _ => Err(YubiKeySelectionError::Ambiguous { connected: self }),
            },
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for ConnectedYubiKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

/// A device-selection outcome that command boundaries can render with their
/// own remediation.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum YubiKeySelectionError {
    #[error("no YubiKey is connected")]
    NoneConnected,
    #[error("YubiKey {requested} is not connected")]
    NotConnected {
        requested: YubiKeySerial,
        connected: ConnectedYubiKeys,
    },
    #[error("multiple YubiKeys are connected")]
    Ambiguous { connected: ConnectedYubiKeys },
}

/// Open the device with the given serial over PC/SC.
pub(crate) fn open(serial: YubiKeySerial) -> Result<YubiKey, DeviceError> {
    YubiKey::open_by_serial(Serial(serial.number())).map_err(|source| match source {
        PivError::NotFound => DeviceError::NotFound { serial },
        source => DeviceError::Open { serial, source },
    })
}

/// The two PIV slots QuorumOS uses on an operator YubiKey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum QosSlot {
    Signing,
    KeyAgreement,
}

impl QosSlot {
    fn slot_id(self) -> SlotId {
        match self {
            Self::Signing => SIGNING_SLOT,
            Self::KeyAgreement => KEY_AGREEMENT_SLOT,
        }
    }
}

impl Display for QosSlot {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signing => "signing".fmt(f),
            Self::KeyAgreement => "key-agreement".fmt(f),
        }
    }
}

/// Metadata proving one slot is suitable for an externally importable
/// QuorumOS certificate.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct CertificateSlot {
    slot: QosSlot,
    public_key_info: SubjectPublicKeyInfoOwned,
    // The parsed projection is retained so post-build verification cannot
    // accidentally use a certificate-derived key instead of device metadata.
    verifying_key: VerifyingKey,
}

impl TryFrom<(QosSlot, SlotMetadata)> for CertificateSlot {
    type Error = DeviceError;

    fn try_from((slot, metadata): (QosSlot, SlotMetadata)) -> Result<Self, Self::Error> {
        let SlotMetadata {
            algorithm,
            policy,
            origin,
            public,
            default: _,
            retries: _,
        } = metadata;

        if algorithm != ManagementAlgorithmId::Asymmetric(AlgorithmId::EccP256) {
            return Err(DeviceError::UnexpectedSlotAlgorithm { slot, algorithm });
        }

        let Some((pin_policy, touch_policy)) = policy else {
            return Err(DeviceError::MissingSlotPolicy { slot });
        };

        if pin_policy != PinPolicy::Always || touch_policy != TouchPolicy::Always {
            return Err(DeviceError::UnexpectedSlotPolicy {
                slot,
                pin_policy,
                touch_policy,
            });
        }

        let Some(origin) = origin else {
            return Err(DeviceError::MissingSlotOrigin { slot });
        };

        if origin != Origin::Generated {
            return Err(DeviceError::UnexpectedSlotOrigin { slot, origin });
        }

        let Some(public_key_info) = public else {
            return Err(DeviceError::MissingSlotPublicKey { slot });
        };
        let verifying_key = VerifyingKey::try_from(public_key_info.owned_to_ref())
            .map_err(|source| DeviceError::MalformedSlotPublicKey { slot, source })?;

        Ok(Self {
            slot,
            public_key_info,
            verifying_key,
        })
    }
}

impl CertificateSlot {
    /// Build and verify a self-signed certificate in memory. This submits only
    /// PIN verification and signing APDUs; it never authenticates a management
    /// key or writes an object to the device.
    pub(crate) fn create_certificate(
        self,
        yubikey: &mut YubiKey,
        pin: &Pin,
    ) -> Result<x509_cert::Certificate, DeviceError> {
        let Self {
            slot,
            public_key_info,
            verifying_key,
        } = self;

        let mut serial = [0u8; 20];
        OsRng
            .try_fill_bytes(&mut serial)
            .map_err(DeviceError::GenerateCertificateSerial)?;
        // ASN.1 INTEGERs are signed, so clear the sign bit to prevent this
        // 20-byte certificate serial from being interpreted as negative.
        serial[0] &= 0x7f;

        let serial_number = SerialNumber::new(&serial)
            .map_err(|source| DeviceError::EncodeCertificate { slot, source })?;
        let validity = Validity::from_now(Duration::from_secs(CERTIFICATE_VALIDITY_SECS))
            .map_err(|source| DeviceError::EncodeCertificate { slot, source })?;
        let subject = Name::from_str(QOS_CERTIFICATE_SUBJECT)
            .map_err(|source| DeviceError::EncodeCertificate { slot, source })?;

        yubikey.verify_pin(pin.as_bytes()).map_err(|source| {
            if let PivError::WrongPin { tries } = source {
                DeviceError::WrongPin { tries }
            } else {
                DeviceError::VerifyPin { slot, source }
            }
        })?;

        let signer =
            Signer::<p256::NistP256>::new(yubikey, slot.slot_id(), public_key_info.owned_to_ref())
                .map_err(|source| DeviceError::CreateCertificateSigner { slot, source })?;
        let builder = CertificateBuilder::new(
            Profile::Manual { issuer: None },
            serial_number,
            validity,
            subject,
            public_key_info,
            &signer,
        )
        .map_err(|source| DeviceError::BuildCertificate { slot, source })?;
        let certificate = builder
            .build::<DerSignature>()
            .map_err(|source| DeviceError::BuildCertificate { slot, source })?;

        let signed = certificate
            .tbs_certificate
            .to_der()
            .map_err(|source| DeviceError::EncodeCertificate { slot, source })?;
        let signature = DerSignature::from_bytes(certificate.signature.raw_bytes())
            .map_err(|source| DeviceError::InvalidCertificateSignature { slot, source })?;
        verifying_key
            .verify(&signed, &signature)
            .map_err(|source| DeviceError::InvalidCertificateSignature { slot, source })?;

        Ok(certificate)
    }
}

/// Read and narrow the PIV metadata needed to build certificates without
/// changing device state.
pub(crate) trait CertificateDeviceOps {
    fn certificate_slot(&mut self, slot: QosSlot) -> Result<CertificateSlot, DeviceError>;
}

impl CertificateDeviceOps for YubiKey {
    fn certificate_slot(&mut self, slot: QosSlot) -> Result<CertificateSlot, DeviceError> {
        let metadata = piv::metadata(self, slot.slot_id())
            .map_err(|source| DeviceError::ReadSlotMetadata { slot, source })?;

        CertificateSlot::try_from((slot, metadata))
    }
}

/// What a QuorumOS slot on a device provably holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlotStatus {
    /// No readable certificate and, per the metadata command, no key.
    Empty,
    QosProvisioned,
    /// A key without a readable certificate: what deletion leaves behind
    /// (the private key cannot be deleted), and also any state whose
    /// ownership cannot be proven. Never provisioned over, nothing to
    /// delete.
    KeyWithoutCertificate,
    /// No readable certificate, and the dependency could not determine
    /// whether a key exists. `yubikey` 0.8 reports PIV status `0x6a88`
    /// (referenced data not found) as `GenericError`, so this is also how a
    /// genuinely empty slot appears on affected firmware.
    UnknownWithoutCertificate {
        metadata_error: PivError,
    },
    /// The slot holds a certificate that QuorumOS did not issue.
    Foreign {
        subject: String,
    },
}

/// Status of both QuorumOS slots on one device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeviceStatus {
    pub signing: SlotStatus,
    pub key_agreement: SlotStatus,
}

/// A YubiKey PIV PIN, held only for the duration of device-backed operations
/// and zeroized on drop. Never persisted.
pub(crate) struct Pin(Zeroizing<Vec<u8>>);

impl From<String> for Pin {
    fn from(pin: String) -> Self {
        Self(Zeroizing::new(pin.into_bytes()))
    }
}

impl Pin {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Device-layer failures, carrying enough state for callers to recover
/// safely: which slot an operation died on, and what a refused slot holds.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DeviceError {
    #[error("failed to list YubiKey readers")]
    Discovery(#[source] PivError),
    #[error("failed to inspect smartcard reader {reader}")]
    OpenReader {
        reader: String,
        #[source]
        source: PivError,
    },
    #[error("no connected YubiKey has serial {serial}")]
    NotFound { serial: YubiKeySerial },
    #[error("failed to open YubiKey {serial}")]
    Open {
        serial: YubiKeySerial,
        #[source]
        source: PivError,
    },
    #[error("failed to read the certificate in the {slot} slot")]
    ReadCertificate {
        slot: QosSlot,
        #[source]
        source: PivError,
    },
    #[error("failed to read the {slot} slot's key metadata")]
    ReadSlotMetadata {
        slot: QosSlot,
        #[source]
        source: PivError,
    },
    #[error("the {slot} slot uses {algorithm:?}; expected ECC P-256")]
    UnexpectedSlotAlgorithm {
        slot: QosSlot,
        algorithm: ManagementAlgorithmId,
    },
    #[error("the {slot} slot metadata does not report PIN and touch policies")]
    MissingSlotPolicy { slot: QosSlot },
    #[error(
        "the {slot} slot uses PIN policy {pin_policy:?} and touch policy {touch_policy:?}; expected Always for both"
    )]
    UnexpectedSlotPolicy {
        slot: QosSlot,
        pin_policy: PinPolicy,
        touch_policy: TouchPolicy,
    },
    #[error("the {slot} slot metadata does not report whether its key was generated or imported")]
    MissingSlotOrigin { slot: QosSlot },
    #[error("the {slot} slot key is {origin:?}; expected a key generated on the YubiKey")]
    UnexpectedSlotOrigin { slot: QosSlot, origin: Origin },
    #[error("the {slot} slot metadata does not include a public key")]
    MissingSlotPublicKey { slot: QosSlot },
    #[error("the {slot} slot metadata contains a malformed P-256 public key")]
    MalformedSlotPublicKey {
        slot: QosSlot,
        #[source]
        source: x509_cert::spki::Error,
    },
    /// The slot state cannot be attributed: a key exists, but there is no
    /// readable QuorumOS certificate proving TVC manages it.
    #[error(
        "the {slot} slot holds a key but no readable QuorumOS certificate; install the matching \
         certificate with `ykman` before using this device with TVC"
    )]
    OccupiedWithoutCertificate { slot: QosSlot },
    /// The metadata command did not distinguish an empty slot from an
    /// existing key.
    #[error(
        "the {slot} slot has no readable QuorumOS certificate, and its key presence could not be \
         determined; confirm firmware 5.3 or later and install the matching certificate with \
         `ykman` before using this device with TVC"
    )]
    UnknownWithoutCertificate {
        slot: QosSlot,
        #[source]
        metadata_error: PivError,
    },
    /// A slot holds material TVC does not manage; nothing was modified.
    #[error(
        "the {slot} slot holds a non-QuorumOS certificate ({subject}); refusing to use this device"
    )]
    ForeignSlot { slot: QosSlot, subject: String },
    #[error(
        "the {slot} slot's QuorumOS certificate is issued by {issuer}; expected a self-signed \
         certificate issued by {subject}"
    )]
    CertificateNotSelfIssued {
        slot: QosSlot,
        subject: String,
        issuer: String,
    },
    #[error("the {slot} slot's QuorumOS certificate contains a malformed P-256 public key")]
    MalformedCertificatePublicKey {
        slot: QosSlot,
        #[source]
        source: x509_cert::spki::Error,
    },
    #[error("the {slot} slot's QuorumOS certificate does not match the device-generated key")]
    CertificatePublicKeyMismatch { slot: QosSlot },
    /// The slot holds no key; the device is not fully configured.
    #[error("the {slot} slot holds no QuorumOS key")]
    EmptySlot { slot: QosSlot },
    #[error("the YubiKey PIN was rejected; {tries} attempts remain before the PIN locks")]
    WrongPin { tries: u8 },
    #[error("failed to verify the YubiKey PIN before signing the {slot} certificate")]
    VerifyPin {
        slot: QosSlot,
        #[source]
        source: PivError,
    },
    #[error("failed to generate a random X.509 certificate serial number")]
    GenerateCertificateSerial(#[source] rand_core::Error),
    #[error("failed to encode the {slot} certificate")]
    EncodeCertificate {
        slot: QosSlot,
        #[source]
        source: x509_cert::der::Error,
    },
    #[error("failed to initialize the {slot} certificate signer")]
    CreateCertificateSigner {
        slot: QosSlot,
        #[source]
        source: PivError,
    },
    #[error("failed to build the {slot} certificate")]
    BuildCertificate {
        slot: QosSlot,
        #[source]
        source: x509_cert::builder::Error,
    },
    #[error("the {slot} certificate signature did not verify against its metadata public key")]
    InvalidCertificateSignature {
        slot: QosSlot,
        #[source]
        source: p256::ecdsa::Error,
    },
    // qos_client's YubiKeyError implements neither Display nor Error, so it
    // is rendered by value instead of chained as a source.
    // TODO: derive those in qos_client and turn these fields into #[source].
    #[error("failed to read the operator public key from the device: {error:?}")]
    ReadPairPublicKey { error: YubiKeyError },
    #[error(
        "failed to sign with the YubiKey (a missed touch while it blinks times out): {error:?}"
    )]
    Sign { error: YubiKeyError },
    #[error(
        "failed to compute the YubiKey shared secret (a missed touch while it blinks times out): {error:?}"
    )]
    KeyAgreement { error: YubiKeyError },
    #[error("device returned a malformed operator public key")]
    MalformedPairPublicKey(#[source] QosOperatorPublicKeyParseError),
}

/// Per-device operations TVC needs, as an extension of [`YubiKey`] itself so
/// inspection and private-key operations use the caller's one open handle.
pub(crate) trait DeviceOps {
    /// What one QuorumOS slot provably holds.
    fn slot_status(&mut self, slot: QosSlot) -> Result<SlotStatus, DeviceError>;

    /// What both QuorumOS slots provably hold.
    fn device_status(&mut self) -> Result<DeviceStatus, DeviceError>;

    /// The composite `encrypt_public ‖ sign_public` operator key, read from
    /// the two slot certificates.
    fn pair_public_key(&mut self) -> Result<QosOperatorPublicKey, DeviceError>;

    /// Sign `message` with the signing-slot key: the message is SHA-256
    /// digested on the way in and the signature comes back as raw 64-byte
    /// `r ‖ s`, verified against the slot certificate before returning.
    /// Requires the PIN and a physical touch.
    fn sign(&mut self, pin: &Pin, message: &[u8]) -> Result<Vec<u8>, DeviceError>;

    /// Raw ECDH between the key-agreement slot key and `sender_public`.
    /// Requires the PIN and a physical touch.
    fn key_agreement(
        &mut self,
        pin: &Pin,
        sender_public: PublicKey,
    ) -> Result<Zeroizing<Vec<u8>>, DeviceError>;

    /// Verify both QuorumOS slots hold configured QuorumOS keys and return the
    /// device's composite operator key. Refuses on foreign or empty slots.
    fn verified_pair_public_key(&mut self) -> Result<QosOperatorPublicKey, DeviceError> {
        let DeviceStatus {
            signing,
            key_agreement,
        } = self.device_status()?;

        match (signing, key_agreement) {
            (SlotStatus::Foreign { subject }, _) => Err(DeviceError::ForeignSlot {
                slot: QosSlot::Signing,
                subject,
            }),
            (SlotStatus::KeyWithoutCertificate, _) => {
                Err(DeviceError::OccupiedWithoutCertificate {
                    slot: QosSlot::Signing,
                })
            }
            (SlotStatus::UnknownWithoutCertificate { metadata_error }, _) => {
                Err(DeviceError::UnknownWithoutCertificate {
                    slot: QosSlot::Signing,
                    metadata_error,
                })
            }
            (_, SlotStatus::Foreign { subject }) => Err(DeviceError::ForeignSlot {
                slot: QosSlot::KeyAgreement,
                subject,
            }),
            (_, SlotStatus::KeyWithoutCertificate) => {
                Err(DeviceError::OccupiedWithoutCertificate {
                    slot: QosSlot::KeyAgreement,
                })
            }
            (_, SlotStatus::UnknownWithoutCertificate { metadata_error }) => {
                Err(DeviceError::UnknownWithoutCertificate {
                    slot: QosSlot::KeyAgreement,
                    metadata_error,
                })
            }
            (SlotStatus::Empty, _) => Err(DeviceError::EmptySlot {
                slot: QosSlot::Signing,
            }),
            (_, SlotStatus::Empty) => Err(DeviceError::EmptySlot {
                slot: QosSlot::KeyAgreement,
            }),
            (SlotStatus::QosProvisioned, SlotStatus::QosProvisioned) => self.pair_public_key(),
        }
    }
}

impl DeviceOps for YubiKey {
    fn slot_status(&mut self, slot: QosSlot) -> Result<SlotStatus, DeviceError> {
        match Certificate::read(self, slot.slot_id()) {
            Ok(certificate) => {
                let subject = certificate.subject();

                if subject != QOS_CERTIFICATE_SUBJECT {
                    return Ok(SlotStatus::Foreign { subject });
                }

                let metadata = piv::metadata(self, slot.slot_id())
                    .map_err(|source| DeviceError::ReadSlotMetadata { slot, source })?;
                let CertificateSlot {
                    slot: _,
                    public_key_info: _,
                    verifying_key,
                } = CertificateSlot::try_from((slot, metadata))?;
                let issuer = certificate.issuer();

                if issuer != subject {
                    return Err(DeviceError::CertificateNotSelfIssued {
                        slot,
                        subject,
                        issuer,
                    });
                }

                let certificate_key =
                    VerifyingKey::try_from(certificate.subject_pki()).map_err(|source| {
                        DeviceError::MalformedCertificatePublicKey { slot, source }
                    })?;

                if certificate_key != verifying_key {
                    return Err(DeviceError::CertificatePublicKeyMismatch { slot });
                }

                let signed = certificate
                    .cert
                    .tbs_certificate
                    .to_der()
                    .map_err(|source| DeviceError::EncodeCertificate { slot, source })?;
                let signature = DerSignature::from_bytes(certificate.cert.signature.raw_bytes())
                    .map_err(|source| DeviceError::InvalidCertificateSignature { slot, source })?;
                verifying_key
                    .verify(&signed, &signature)
                    .map_err(|source| DeviceError::InvalidCertificateSignature { slot, source })?;

                Ok(SlotStatus::QosProvisioned)
            }
            Err(PivError::InvalidObject | PivError::NotFound) => {
                match piv::metadata(self, slot.slot_id()) {
                    Ok(_) => Ok(SlotStatus::KeyWithoutCertificate),
                    Err(PivError::NotFound) => Ok(SlotStatus::Empty),
                    Err(metadata_error @ (PivError::GenericError | PivError::NotSupported)) => {
                        Ok(SlotStatus::UnknownWithoutCertificate { metadata_error })
                    }
                    Err(source) => Err(DeviceError::ReadSlotMetadata { slot, source }),
                }
            }
            Err(source) => Err(DeviceError::ReadCertificate { slot, source }),
        }
    }

    fn device_status(&mut self) -> Result<DeviceStatus, DeviceError> {
        Ok(DeviceStatus {
            signing: self.slot_status(QosSlot::Signing)?,
            key_agreement: self.slot_status(QosSlot::KeyAgreement)?,
        })
    }

    fn pair_public_key(&mut self) -> Result<QosOperatorPublicKey, DeviceError> {
        let bytes = qos_client::yubikey::pair_public_key(self)
            .map_err(|error| DeviceError::ReadPairPublicKey { error })?;

        QosOperatorPublicKey::try_from(bytes.as_slice())
            .map_err(DeviceError::MalformedPairPublicKey)
    }

    fn sign(&mut self, pin: &Pin, message: &[u8]) -> Result<Vec<u8>, DeviceError> {
        qos_client::yubikey::sign_data(self, message, pin.as_bytes()).map_err(|error| match error {
            YubiKeyError::FailedToVerifyPin(PivError::WrongPin { tries }) => {
                DeviceError::WrongPin { tries }
            }
            error => DeviceError::Sign { error },
        })
    }

    fn key_agreement(
        &mut self,
        pin: &Pin,
        sender_public: PublicKey,
    ) -> Result<Zeroizing<Vec<u8>>, DeviceError> {
        let sender_point = sender_public.to_encoded_point(false);

        qos_client::yubikey::key_agreement(self, sender_point.as_bytes(), pin.as_bytes()).map_err(
            |error| match error {
                YubiKeyError::FailedToVerifyPin(PivError::WrongPin { tries }) => {
                    DeviceError::WrongPin { tries }
                }
                error => DeviceError::KeyAgreement { error },
            },
        )
    }
}

pub(crate) mod pair;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests {
    use super::test_support::{FakeDevice, PIN};
    use super::*;
    use p256::ecdh::diffie_hellman;
    use p256::{PublicKey, ecdsa::SigningKey};
    use qos_p256::{P256Pair, P256Public};

    fn certificate_metadata() -> SlotMetadata {
        let signing_key = SigningKey::from_bytes((&[42u8; 32]).into()).unwrap();
        let public = SubjectPublicKeyInfoOwned::from_key(*signing_key.verifying_key()).unwrap();

        SlotMetadata {
            algorithm: ManagementAlgorithmId::Asymmetric(AlgorithmId::EccP256),
            policy: Some((PinPolicy::Always, TouchPolicy::Always)),
            origin: Some(Origin::Generated),
            public: Some(public),
            default: None,
            retries: None,
        }
    }

    #[test]
    fn certificate_slot_accepts_required_metadata() {
        let slot = CertificateSlot::try_from((QosSlot::Signing, certificate_metadata())).unwrap();

        assert_eq!(slot.slot, QosSlot::Signing);
    }

    #[test]
    fn certificate_slot_requires_ecc_p256() {
        let mut metadata = certificate_metadata();
        metadata.algorithm = ManagementAlgorithmId::Asymmetric(AlgorithmId::EccP384);

        let error = CertificateSlot::try_from((QosSlot::Signing, metadata)).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::UnexpectedSlotAlgorithm {
                slot: QosSlot::Signing,
                algorithm: ManagementAlgorithmId::Asymmetric(AlgorithmId::EccP384),
            }
        ));
    }

    #[test]
    fn certificate_slot_requires_pin_and_touch_always() {
        let mut metadata = certificate_metadata();
        metadata.policy = Some((PinPolicy::Once, TouchPolicy::Cached));

        let error = CertificateSlot::try_from((QosSlot::KeyAgreement, metadata)).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::UnexpectedSlotPolicy {
                slot: QosSlot::KeyAgreement,
                pin_policy: PinPolicy::Once,
                touch_policy: TouchPolicy::Cached,
            }
        ));
    }

    #[test]
    fn certificate_slot_requires_a_device_generated_key() {
        let mut metadata = certificate_metadata();
        metadata.origin = Some(Origin::Imported);

        let error = CertificateSlot::try_from((QosSlot::Signing, metadata)).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::UnexpectedSlotOrigin {
                slot: QosSlot::Signing,
                origin: Origin::Imported,
            }
        ));
    }

    #[test]
    fn certificate_slot_requires_the_metadata_public_key() {
        let mut metadata = certificate_metadata();
        metadata.public = None;

        let error = CertificateSlot::try_from((QosSlot::KeyAgreement, metadata)).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::MissingSlotPublicKey {
                slot: QosSlot::KeyAgreement,
            }
        ));
    }

    #[test]
    fn certificate_slot_requires_a_well_formed_p256_public_key() {
        let mut metadata = certificate_metadata();
        metadata.public.as_mut().unwrap().subject_public_key =
            x509_cert::der::asn1::BitString::from_bytes(&[0x04, 0x01, 0x02]).unwrap();

        let error = CertificateSlot::try_from((QosSlot::Signing, metadata)).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::MalformedSlotPublicKey {
                slot: QosSlot::Signing,
                ..
            }
        ));
    }

    fn default_pin() -> Pin {
        Pin::from(String::from_utf8(PIN.to_vec()).unwrap())
    }

    fn connected(serials: &[u32]) -> ConnectedYubiKeys {
        ConnectedYubiKeys::from(
            serials
                .iter()
                .map(|&serial| YubiKeySerial::from(serial))
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn chooses_an_explicit_connected_serial() {
        let chosen = connected(&[0x01c9_5c1f, 0xdead_beef])
            .choose(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap();

        assert_eq!(chosen, YubiKeySerial::from(0xdead_beef));
    }

    #[test]
    fn an_unconnected_explicit_serial_is_refused_with_the_connected_list() {
        let error = connected(&[0x01c9_5c1f])
            .choose(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap_err();

        assert_eq!(
            error,
            YubiKeySelectionError::NotConnected {
                requested: YubiKeySerial::from(0xdead_beef),
                connected: connected(&[0x01c9_5c1f]),
            }
        );
    }

    #[test]
    fn an_explicit_serial_with_nothing_connected_is_a_bare_refusal() {
        let error = connected(&[])
            .choose(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap_err();

        assert_eq!(
            error,
            YubiKeySelectionError::NotConnected {
                requested: YubiKeySerial::from(0xdead_beef),
                connected: connected(&[]),
            }
        );
    }

    #[test]
    fn chooses_the_sole_connected_device() {
        let chosen = connected(&[0x01c9_5c1f]).choose(None).unwrap();

        assert_eq!(chosen, YubiKeySerial::from(0x01c9_5c1f));
    }

    #[test]
    fn no_connected_device_is_refused() {
        let error = connected(&[]).choose(None).unwrap_err();

        assert_eq!(error, YubiKeySelectionError::NoneConnected);
    }

    #[test]
    fn several_connected_devices_ask_for_unplugging_or_a_serial() {
        let error = connected(&[0x01c9_5c1f, 0xdead_beef])
            .choose(None)
            .unwrap_err();

        assert_eq!(
            error,
            YubiKeySelectionError::Ambiguous {
                connected: connected(&[0x01c9_5c1f, 0xdead_beef]),
            }
        );
    }

    fn sole_connected_serial() -> YubiKeySerial {
        let connected = connected_serials().unwrap();

        let [serial] = connected.as_slice() else {
            panic!("connect exactly one YubiKey; found {connected:?}");
        };

        *serial
    }

    fn foreign() -> SlotStatus {
        SlotStatus::Foreign {
            subject: "CN=SomeoneElse".to_string(),
        }
    }

    #[test]
    fn sign_produces_a_signature_verifiable_with_the_composite_key() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);
        let message = b"manifest hash stand-in";

        let signature = device.sign(&default_pin(), message).unwrap();

        let composite = hex::decode(device.operator_public_key().to_string()).unwrap();
        P256Public::from_bytes(&composite)
            .unwrap()
            .verify(message, &signature)
            .unwrap();
    }

    #[test]
    fn sign_rejects_a_wrong_pin_with_the_retry_count() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);

        let error = device
            .sign(&Pin::from("999999".to_string()), b"message")
            .unwrap_err();

        assert!(matches!(error, DeviceError::WrongPin { tries: 3 }));
    }

    #[test]
    fn sign_requires_a_provisioned_signing_slot() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::QosProvisioned);

        let error = device.sign(&default_pin(), b"message").unwrap_err();

        assert!(matches!(
            error,
            DeviceError::EmptySlot {
                slot: QosSlot::Signing
            }
        ));
    }

    #[test]
    fn key_agreement_matches_a_software_shared_secret() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);
        let sender = P256Pair::generate().unwrap();

        let secret = device
            .key_agreement(&default_pin(), sender.encryption_key().public_key())
            .unwrap();

        let composite = hex::decode(device.operator_public_key().to_string()).unwrap();
        let device_encrypt_public = PublicKey::from_sec1_bytes(&composite[..65]).unwrap();
        let expected = diffie_hellman(
            sender.encryption_key().to_nonzero_scalar(),
            device_encrypt_public.as_affine(),
        );

        assert_eq!(secret.as_slice(), expected.raw_secret_bytes().as_slice());
    }

    #[test]
    fn key_agreement_requires_a_provisioned_key_agreement_slot() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::Empty);
        let sender = P256Pair::generate().unwrap();

        let error = device
            .key_agreement(&default_pin(), sender.encryption_key().public_key())
            .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::EmptySlot {
                slot: QosSlot::KeyAgreement
            }
        ));
    }

    #[test]
    fn verified_pair_public_key_returns_the_composite() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);

        let key = device.verified_pair_public_key().unwrap();

        assert_eq!(key, device.operator_public_key());
    }

    #[test]
    fn verified_pair_public_key_refuses_a_foreign_slot() {
        let mut device = FakeDevice::new(foreign(), SlotStatus::QosProvisioned);

        let error = device.verified_pair_public_key().unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ForeignSlot {
                slot: QosSlot::Signing,
                ..
            }
        ));
    }

    #[test]
    fn verified_pair_public_key_prioritizes_an_unproven_key_over_an_empty_slot() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::KeyWithoutCertificate);

        let error = device.verified_pair_public_key().unwrap_err();

        assert!(matches!(
            error,
            DeviceError::OccupiedWithoutCertificate {
                slot: QosSlot::KeyAgreement
            }
        ));
    }

    #[test]
    fn verified_pair_public_key_preserves_an_unknown_slots_metadata_error() {
        let mut device = FakeDevice::new(
            SlotStatus::UnknownWithoutCertificate {
                metadata_error: PivError::GenericError,
            },
            SlotStatus::QosProvisioned,
        );

        let error = device.verified_pair_public_key().unwrap_err();

        assert_eq!(
            std::error::Error::source(&error).and_then(|source| source.downcast_ref::<PivError>()),
            Some(&PivError::GenericError)
        );
    }

    #[test]
    fn verified_pair_public_key_refuses_an_empty_slot() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::Empty);

        let error = device.verified_pair_public_key().unwrap_err();

        assert!(matches!(
            error,
            DeviceError::EmptySlot {
                slot: QosSlot::KeyAgreement
            }
        ));
    }

    /// Signing and key agreement against real hardware.
    ///
    /// Requires an externally configured YubiKey with QuorumOS certificates
    /// in both slots and the factory-default PIN. Sign and key agreement each
    /// need a touch while the device blinks. Run manually:
    /// `cargo test -p tvc --lib -- --ignored hardware_`
    #[test]
    #[ignore = "requires an externally configured YubiKey with the default PIN; sign and key agreement each need a touch"]
    fn hardware_sign_and_key_agreement() {
        let mut yubikey = open(sole_connected_serial()).unwrap();
        let pin = default_pin();

        let absent = YubiKeySerial::from(0x0000_0001);
        assert!(matches!(open(absent), Err(DeviceError::NotFound { .. })));

        let verified = yubikey.verified_pair_public_key().unwrap();
        let composite = hex::decode(verified.to_string()).unwrap();

        let message = b"tvc hardware signing test";
        let signature = yubikey.sign(&pin, message).unwrap();
        P256Public::from_bytes(&composite)
            .unwrap()
            .verify(message, &signature)
            .unwrap();

        let sender = P256Pair::generate().unwrap();
        let secret = yubikey
            .key_agreement(&pin, sender.encryption_key().public_key())
            .unwrap();
        let device_encrypt_public = PublicKey::from_sec1_bytes(&composite[..65]).unwrap();
        let expected = diffie_hellman(
            sender.encryption_key().to_nonzero_scalar(),
            device_encrypt_public.as_affine(),
        );
        assert_eq!(secret.as_slice(), expected.raw_secret_bytes().as_slice());
    }

    /// Wrong-PIN reporting against real hardware.
    ///
    /// Burns one PIN retry with a deliberately wrong PIN, then restores the
    /// counter by signing with the factory-default PIN (one touch). Requires
    /// an externally configured device.
    #[test]
    #[ignore = "requires an externally configured YubiKey with default PIN; burns and restores one PIN retry"]
    fn hardware_wrong_pin_reports_retries() {
        let mut yubikey = open(sole_connected_serial()).unwrap();

        let error = yubikey
            .sign(&Pin::from("999999".to_string()), b"wrong pin probe")
            .unwrap_err();
        assert!(matches!(error, DeviceError::WrongPin { .. }));

        yubikey
            .sign(&default_pin(), b"restore the retry counter")
            .unwrap();
    }
}
