//! YubiKey device management for TVC operator keys.
//!
//! Concrete hardware access has three shapes: discovering connected devices
//! is an effect supplied by [`crate::output::Ctx`] (backed by
//! [`connected_serials`]), opening a serial is the ambient [`open`] entry
//! point, and per-device functionality hangs off the opened
//! [`yubikey::YubiKey`] through the [`DeviceOps`] extension trait. Commands
//! open a device once and drive inspection and mutation through that one
//! handle.
//!
//! Provisioning follows the QuorumOS conventions from `qos_client`: two
//! P-256 keys in the PIV signing and key-agreement slots, each under a
//! self-signed `CN=QuorumOS` certificate, PIN policy "always" and touch
//! policy "always". That certificate subject is how TVC recognizes its own
//! material; a slot holding anything else is never touched. A slot only
//! counts as empty when its emptiness is proven — no readable certificate
//! AND no key per the PIV metadata command (firmware 5.3+). Ambiguous
//! certificate-less slots are never guessed empty: ordinary flows refuse
//! them, while the explicit provisioning command may overwrite them only
//! after user confirmation.
//!
//! Deletion removes the QuorumOS certificates. The `yubikey` crate offers no
//! way to delete a slot's private key short of a full PIV reset, which would
//! erase credentials TVC does not own — so the key material itself stays
//! behind until the slot is reprovisioned, and TVC never resets a device.

use crate::config::turnkey::{
    Config, QosOperatorPublicKey, QosOperatorPublicKeyParseError, Registration, YubiKeySerial,
};
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
    Error as PivError, MgmKey, PinPolicy, Serial, TouchPolicy, YubiKey,
    certificate::{Certificate, yubikey_signer::Signer},
    piv::{self, AlgorithmId, ManagementAlgorithmId, Origin, SlotId, SlotMetadata},
    reader::Context,
};
use zeroize::Zeroizing;

/// The subject of the self-signed certificates `qos_client` provisions.
// TODO: qos_client keeps this constant private; have it exported (along with
// a re-export of the yubikey crate) and use it here.
const QOS_CERTIFICATE_SUBJECT: &str = "CN=QuorumOS";

/// Equivalent to about ten years, matching the existing QuorumOS YubiKey
/// certificates.
const CERTIFICATE_VALIDITY_SECS: u64 = 10 * 60 * 60 * 24 * 365;

/// Serials of the connected YubiKeys, skipping smartcards that are not
/// YubiKeys. The real implementation behind the discovery effect on
/// [`crate::output::Ctx`].
pub(crate) fn connected_serials() -> Result<Vec<YubiKeySerial>, DeviceError> {
    let mut context = Context::open().map_err(DeviceError::Discovery)?;
    let readers = context.iter().map_err(DeviceError::Discovery)?;

    Ok(readers
        .filter_map(|reader| reader.open().ok())
        .map(|yubikey| YubiKeySerial::from(yubikey.serial().0))
        .collect())
}

/// The connected YubiKey serials captured by one discovery pass.
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
    pub(crate) fn choose(&self, explicit: Option<YubiKeySerial>) -> anyhow::Result<YubiKeySerial> {
        let serials = || {
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        };

        match explicit {
            Some(serial) if self.0.contains(&serial) => Ok(serial),
            Some(serial) => {
                let connected = if self.0.is_empty() {
                    String::new()
                } else {
                    format!("; connected: {}", serials())
                };

                anyhow::bail!("YubiKey {serial} is not connected{connected}")
            }
            None => match self.0.as_slice() {
                [] => anyhow::bail!("no YubiKey is connected"),
                [sole] => Ok(*sole),
                _ => anyhow::bail!(
                    "multiple YubiKeys are connected (serials {}); unplug all but \
                     the one to use and try again, or pass --serial",
                    serials()
                ),
            },
        }
    }
}

/// Open the device with the given serial over PC/SC.
pub(crate) fn open(serial: YubiKeySerial) -> Result<YubiKey, DeviceError> {
    YubiKey::open_by_serial(Serial(serial.number())).map_err(|source| match source {
        PivError::NotFound => DeviceError::NotFound { serial },
        source => DeviceError::Open { serial, source },
    })
}

/// The two PIV slots QuorumOS provisions on an operator YubiKey.
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

impl DeviceStatus {
    fn slots(&self) -> [(QosSlot, &SlotStatus); 2] {
        [
            (QosSlot::Signing, &self.signing),
            (QosSlot::KeyAgreement, &self.key_agreement),
        ]
    }

    /// The first slot state that rules out provisioning the device: foreign
    /// material, or a key whose ownership cannot be proven.
    pub(crate) fn unprovisionable_slot(&self) -> Option<DeviceError> {
        self.slots()
            .into_iter()
            .find_map(|(slot, status)| match status {
                SlotStatus::Foreign { subject } => Some(DeviceError::ForeignSlot {
                    slot,
                    subject: subject.clone(),
                }),
                SlotStatus::KeyWithoutCertificate => {
                    Some(DeviceError::OccupiedWithoutCertificate { slot })
                }
                SlotStatus::UnknownWithoutCertificate { metadata_error } => {
                    Some(DeviceError::UnknownWithoutCertificate {
                        slot,
                        metadata_error: *metadata_error,
                    })
                }
                SlotStatus::Empty | SlotStatus::QosProvisioned => None,
            })
    }

    /// The first slot holding a certificate TVC must not delete. A bare key
    /// is not a deletion obstacle — there is no certificate to remove.
    pub(crate) fn foreign_slot(&self) -> Option<(QosSlot, &str)> {
        self.slots()
            .into_iter()
            .find_map(|(slot, status)| match status {
                SlotStatus::Foreign { subject } => Some((slot, subject.as_str())),
                SlotStatus::Empty
                | SlotStatus::QosProvisioned
                | SlotStatus::KeyWithoutCertificate
                | SlotStatus::UnknownWithoutCertificate { .. } => None,
            })
    }

    /// Slots whose existing key material, if any, would be replaced by
    /// provisioning and therefore require explicit user confirmation.
    pub(crate) fn slots_requiring_overwrite(&self) -> Vec<QosSlot> {
        self.slots()
            .into_iter()
            .filter_map(|(slot, status)| match status {
                SlotStatus::KeyWithoutCertificate
                | SlotStatus::UnknownWithoutCertificate { .. } => Some(slot),
                SlotStatus::Empty | SlotStatus::QosProvisioned | SlotStatus::Foreign { .. } => None,
            })
            .collect()
    }

    fn slots_to_provision(
        &self,
        overwrite: CertlessSlotOverwrite,
    ) -> Result<Vec<QosSlot>, DeviceError> {
        if overwrite == CertlessSlotOverwrite::Refuse {
            if let Some(error) = self.unprovisionable_slot() {
                return Err(error);
            }
        } else if let Some((slot, subject)) = self.foreign_slot() {
            return Err(DeviceError::ForeignSlot {
                slot,
                subject: subject.to_string(),
            });
        }

        Ok(self
            .slots()
            .into_iter()
            .filter_map(|(slot, status)| match status {
                SlotStatus::Empty => Some(slot),
                SlotStatus::KeyWithoutCertificate
                | SlotStatus::UnknownWithoutCertificate { .. }
                    if overwrite == CertlessSlotOverwrite::Confirmed =>
                {
                    Some(slot)
                }
                SlotStatus::QosProvisioned
                | SlotStatus::KeyWithoutCertificate
                | SlotStatus::UnknownWithoutCertificate { .. }
                | SlotStatus::Foreign { .. } => None,
            })
            .collect())
    }

    pub(crate) fn slots_with(&self, wanted: SlotStatus) -> Vec<QosSlot> {
        self.slots()
            .into_iter()
            .filter(|(_, status)| **status == wanted)
            .map(|(slot, _)| slot)
            .collect()
    }
}

/// Whether the caller has obtained explicit permission to replace keys in
/// slots that have no readable QuorumOS certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CertlessSlotOverwrite {
    Refuse,
    Confirmed,
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
        "the {slot} slot holds a key but no readable QuorumOS certificate; refusing to touch \
         this device without confirmation (run `tvc keys provision-yubikey` to inspect and \
         confirm replacing it)"
    )]
    OccupiedWithoutCertificate { slot: QosSlot },
    /// The metadata command did not distinguish an empty slot from an
    /// existing key. Only the explicit provisioning command can turn this
    /// into a confirmed overwrite.
    #[error(
        "the {slot} slot has no readable QuorumOS certificate, and its key presence could not be \
         determined ({metadata_error}); refusing to overwrite it without confirmation (run \
         `tvc keys provision-yubikey` to inspect and confirm)"
    )]
    UnknownWithoutCertificate {
        slot: QosSlot,
        metadata_error: PivError,
    },
    /// A slot holds material TVC does not manage; nothing was modified.
    #[error(
        "the {slot} slot holds a non-QuorumOS certificate ({subject}); refusing to touch this device"
    )]
    ForeignSlot { slot: QosSlot, subject: String },
    /// The slot holds no key; the device is not fully provisioned.
    #[error("the {slot} slot holds no QuorumOS key")]
    EmptySlot { slot: QosSlot },
    #[error(
        "the {slot} slot no longer holds a QuorumOS certificate; the device changed since it was inspected"
    )]
    ChangedSinceInspection { slot: QosSlot },
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
    #[error("failed to provision the {slot} slot: {error:?}")]
    Provision { slot: QosSlot, error: YubiKeyError },
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
    #[error("failed to authenticate with the default PIV management key")]
    Authenticate(#[source] PivError),
    #[error("failed to delete the certificate in the {slot} slot")]
    DeleteCertificate {
        slot: QosSlot,
        #[source]
        source: PivError,
    },
}

/// Decision table mapping raw certificate and key-metadata outcomes to a
/// slot classification.
///
/// `Certificate::read` reports an absent, unreadable, and malformed
/// certificate identically, so those outcomes alone never prove emptiness;
/// the (lazily consulted) key metadata must also show no key before a slot
/// counts as [`SlotStatus::Empty`]. Pure so the ambiguous cases stay
/// regression-tested without hardware.
fn classify_slot(
    slot: QosSlot,
    certificate: Result<String, PivError>,
    key_presence: impl FnOnce() -> Result<bool, PivError>,
) -> Result<SlotStatus, DeviceError> {
    match certificate {
        Ok(subject) if subject == QOS_CERTIFICATE_SUBJECT => Ok(SlotStatus::QosProvisioned),
        Ok(subject) => Ok(SlotStatus::Foreign { subject }),
        Err(PivError::InvalidObject | PivError::NotFound) => match key_presence() {
            Ok(false) => Ok(SlotStatus::Empty),
            Ok(true) => Ok(SlotStatus::KeyWithoutCertificate),
            Err(metadata_error @ (PivError::GenericError | PivError::NotSupported)) => {
                Ok(SlotStatus::UnknownWithoutCertificate { metadata_error })
            }
            Err(source) => Err(DeviceError::ReadSlotMetadata { slot, source }),
        },
        Err(source) => Err(DeviceError::ReadCertificate { slot, source }),
    }
}

/// Per-device operations TVC needs, as an extension of [`YubiKey`] itself so
/// inspection and mutation always go through the caller's one open handle.
pub(crate) trait DeviceOps {
    /// What one QuorumOS slot provably holds.
    fn slot_status(&mut self, slot: QosSlot) -> Result<SlotStatus, DeviceError>;

    /// What both QuorumOS slots provably hold.
    fn device_status(&mut self) -> Result<DeviceStatus, DeviceError>;

    /// Generate a P-256 key and QuorumOS certificate in one slot, using the
    /// standing QuorumOS policies. Fails on an occupied slot.
    fn provision_slot(&mut self, slot: QosSlot, pin: &Pin) -> Result<(), DeviceError>;

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

    /// Verify both QuorumOS slots hold TVC-provisioned keys and return the
    /// device's composite operator key. Refuses on foreign or empty slots.
    fn verified_pair_public_key(&mut self) -> Result<QosOperatorPublicKey, DeviceError> {
        let status = self.device_status()?;

        if let Some(error) = status.unprovisionable_slot() {
            return Err(error);
        }

        if let Some(slot) = status.slots_with(SlotStatus::Empty).into_iter().next() {
            return Err(DeviceError::EmptySlot { slot });
        }

        self.pair_public_key()
    }

    /// Delete one slot's certificate.
    ///
    /// Gated on what this handle reads immediately beforehand: the slot must
    /// still hold exactly the QuorumOS certificate. The slot's private key
    /// cannot be deleted (see the module docs) and stays behind.
    fn delete_qos_certificate(&mut self, slot: QosSlot) -> Result<(), DeviceError>;
}

impl DeviceOps for YubiKey {
    fn slot_status(&mut self, slot: QosSlot) -> Result<SlotStatus, DeviceError> {
        let certificate =
            Certificate::read(self, slot.slot_id()).map(|certificate| certificate.subject());

        classify_slot(slot, certificate, || {
            match piv::metadata(self, slot.slot_id()) {
                Ok(_) => Ok(true),
                Err(PivError::NotFound) => Ok(false),
                Err(source) => Err(source),
            }
        })
    }

    fn device_status(&mut self) -> Result<DeviceStatus, DeviceError> {
        Ok(DeviceStatus {
            signing: self.slot_status(QosSlot::Signing)?,
            key_agreement: self.slot_status(QosSlot::KeyAgreement)?,
        })
    }

    fn provision_slot(&mut self, slot: QosSlot, pin: &Pin) -> Result<(), DeviceError> {
        qos_client::yubikey::generate_signed_certificate(
            self,
            slot.slot_id(),
            pin.as_bytes(),
            MgmKey::default(),
            TouchPolicy::Always,
        )
        .map(drop)
        .map_err(|error| DeviceError::Provision { slot, error })
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

    fn delete_qos_certificate(&mut self, slot: QosSlot) -> Result<(), DeviceError> {
        match self.slot_status(slot)? {
            SlotStatus::QosProvisioned => {}
            SlotStatus::Foreign { subject } => {
                return Err(DeviceError::ForeignSlot { slot, subject });
            }
            SlotStatus::Empty
            | SlotStatus::KeyWithoutCertificate
            | SlotStatus::UnknownWithoutCertificate { .. } => {
                return Err(DeviceError::ChangedSinceInspection { slot });
            }
        }

        self.authenticate(MgmKey::default())
            .map_err(DeviceError::Authenticate)?;

        Certificate::delete(self, slot.slot_id())
            .map_err(|source| DeviceError::DeleteCertificate { slot, source })
    }
}

/// Result of [`Config::enroll_yubikey`]: the device key and changes made to
/// the device and registry.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct EnrolledYubiKey {
    pub(crate) public_key: QosOperatorPublicKey,
    pub(crate) provisioned_slots: Vec<QosSlot>,
    pub(crate) registration: Registration,
}

impl Config {
    /// Bring one opened device to the fully provisioned state and make its
    /// in-memory registry entry authoritative. The caller owns input
    /// acquisition, device opening, persistence, and save recovery.
    pub(crate) fn enroll_yubikey<D: DeviceOps>(
        &mut self,
        serial: YubiKeySerial,
        device: &mut D,
        pin: &Pin,
        overwrite: CertlessSlotOverwrite,
    ) -> Result<EnrolledYubiKey, DeviceError> {
        let status = device.device_status()?;
        let provisioned_slots = status.slots_to_provision(overwrite)?;
        provisioned_slots
            .iter()
            .try_for_each(|slot| device.provision_slot(*slot, pin))?;

        let public_key = device.pair_public_key()?;
        let registration = self.yubikeys.register(serial, public_key);

        Ok(EnrolledYubiKey {
            public_key,
            provisioned_slots,
            registration,
        })
    }
}

pub(crate) mod pair;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(test)]
mod tests {
    use super::test_support::{FakeDevice, PIN, serial};
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

    #[test]
    fn enroll_provisions_and_registers_a_fresh_device() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let enrolled = config
            .enroll_yubikey(
                serial(),
                &mut device,
                &default_pin(),
                CertlessSlotOverwrite::Refuse,
            )
            .unwrap();

        assert_eq!(enrolled.public_key, device.operator_public_key());
        assert_eq!(
            enrolled.provisioned_slots,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
        assert_eq!(enrolled.registration, Registration::Added);
        assert_eq!(
            config.yubikeys.get(serial()).unwrap().public_key,
            device.operator_public_key()
        );
    }

    #[test]
    fn re_enrolling_a_registered_device_changes_nothing() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);
        config
            .yubikeys
            .register(serial(), device.operator_public_key());

        let enrolled = config
            .enroll_yubikey(
                serial(),
                &mut device,
                &default_pin(),
                CertlessSlotOverwrite::Refuse,
            )
            .unwrap();

        assert_eq!(enrolled.registration, Registration::Unchanged);
        assert_eq!(enrolled.provisioned_slots, Vec::new());
        assert_eq!(device.provision_calls, Vec::new());
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
            error.to_string(),
            "YubiKey deadbeef is not connected; connected: 01c95c1f"
        );
    }

    #[test]
    fn an_explicit_serial_with_nothing_connected_is_a_bare_refusal() {
        let error = connected(&[])
            .choose(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap_err();

        assert_eq!(error.to_string(), "YubiKey deadbeef is not connected");
    }

    #[test]
    fn chooses_the_sole_connected_device() {
        let chosen = connected(&[0x01c9_5c1f]).choose(None).unwrap();

        assert_eq!(chosen, YubiKeySerial::from(0x01c9_5c1f));
    }

    #[test]
    fn no_connected_device_is_refused() {
        let error = connected(&[]).choose(None).unwrap_err();

        assert_eq!(error.to_string(), "no YubiKey is connected");
    }

    #[test]
    fn several_connected_devices_ask_for_unplugging_or_a_serial() {
        let error = connected(&[0x01c9_5c1f, 0xdead_beef])
            .choose(None)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "multiple YubiKeys are connected (serials 01c95c1f, deadbeef); \
             unplug all but the one to use and try again, or pass --serial"
        );
    }

    fn sole_connected_serial() -> YubiKeySerial {
        let connected = connected_serials().unwrap();

        let [serial] = connected.as_slice() else {
            panic!("connect exactly one YubiKey; found {connected:?}");
        };

        *serial
    }

    fn qos_subject() -> Result<String, PivError> {
        Ok(QOS_CERTIFICATE_SUBJECT.to_string())
    }

    fn metadata_untouched() -> Result<bool, PivError> {
        panic!("a readable certificate must classify without a metadata query");
    }

    #[test]
    fn a_quorumos_certificate_is_provisioned() {
        let status = classify_slot(QosSlot::Signing, qos_subject(), metadata_untouched).unwrap();

        assert_eq!(status, SlotStatus::QosProvisioned);
    }

    #[test]
    fn another_issuers_certificate_is_foreign() {
        let status = classify_slot(
            QosSlot::Signing,
            Ok("CN=SomeoneElse".to_string()),
            metadata_untouched,
        )
        .unwrap();

        assert_eq!(
            status,
            SlotStatus::Foreign {
                subject: "CN=SomeoneElse".to_string()
            }
        );
    }

    #[test]
    fn no_certificate_and_no_key_is_proven_empty() {
        for absent in [PivError::NotFound, PivError::InvalidObject] {
            let status = classify_slot(QosSlot::Signing, Err(absent), || Ok(false)).unwrap();

            assert_eq!(status, SlotStatus::Empty);
        }
    }

    #[test]
    fn a_key_without_a_readable_certificate_is_distinguished_from_empty() {
        for unreadable in [PivError::NotFound, PivError::InvalidObject] {
            let status =
                classify_slot(QosSlot::KeyAgreement, Err(unreadable), || Ok(true)).unwrap();

            assert_eq!(status, SlotStatus::KeyWithoutCertificate);
        }
    }

    #[test]
    fn an_unproven_key_rules_out_provisioning() {
        let status = DeviceStatus {
            signing: SlotStatus::Empty,
            key_agreement: SlotStatus::KeyWithoutCertificate,
        };

        assert!(matches!(
            status.unprovisionable_slot(),
            Some(DeviceError::OccupiedWithoutCertificate {
                slot: QosSlot::KeyAgreement
            })
        ));
        // But it is not a deletion obstacle: there is no certificate there.
        assert_eq!(status.foreign_slot(), None);
    }

    #[test]
    fn generic_metadata_failure_is_an_unknown_certless_slot() {
        let status = classify_slot(QosSlot::Signing, Err(PivError::InvalidObject), || {
            Err(PivError::GenericError)
        })
        .unwrap();

        assert!(matches!(
            status,
            SlotStatus::UnknownWithoutCertificate {
                metadata_error: PivError::GenericError
            }
        ));
    }

    #[test]
    fn a_specific_metadata_failure_is_not_interpreted() {
        let error = classify_slot(QosSlot::Signing, Err(PivError::InvalidObject), || {
            Err(PivError::WrongPin { tries: 2 })
        })
        .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ReadSlotMetadata {
                slot: QosSlot::Signing,
                source: PivError::WrongPin { tries: 2 }
            }
        ));
    }

    #[test]
    fn a_certificate_read_failure_is_not_interpreted() {
        let error = classify_slot(QosSlot::Signing, Err(PivError::GenericError), || {
            panic!("a transport failure must not consult metadata")
        })
        .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ReadCertificate {
                slot: QosSlot::Signing,
                ..
            }
        ));
    }

    fn foreign() -> SlotStatus {
        SlotStatus::Foreign {
            subject: "CN=SomeoneElse".to_string(),
        }
    }

    #[test]
    fn foreign_slot_reports_the_first_foreign_entry() {
        let status = DeviceStatus {
            signing: SlotStatus::QosProvisioned,
            key_agreement: foreign(),
        };

        assert_eq!(
            status.foreign_slot(),
            Some((QosSlot::KeyAgreement, "CN=SomeoneElse"))
        );
        assert_eq!(
            DeviceStatus {
                signing: SlotStatus::Empty,
                key_agreement: SlotStatus::QosProvisioned,
            }
            .foreign_slot(),
            None
        );
    }

    #[test]
    fn slots_with_selects_by_state_in_slot_order() {
        let status = DeviceStatus {
            signing: SlotStatus::QosProvisioned,
            key_agreement: SlotStatus::Empty,
        };

        assert_eq!(
            status.slots_with(SlotStatus::Empty),
            vec![QosSlot::KeyAgreement]
        );
        assert_eq!(
            status.slots_with(SlotStatus::QosProvisioned),
            vec![QosSlot::Signing]
        );
    }

    #[test]
    fn enrollment_refuses_a_certless_key_without_confirmation() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(
            SlotStatus::KeyWithoutCertificate,
            SlotStatus::QosProvisioned,
        );

        let error = config
            .enroll_yubikey(
                serial(),
                &mut device,
                &default_pin(),
                CertlessSlotOverwrite::Refuse,
            )
            .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::OccupiedWithoutCertificate {
                slot: QosSlot::Signing
            }
        ));
        assert_eq!(device.provision_calls, Vec::new());
    }

    #[test]
    fn enrollment_refuses_unknown_key_presence_without_confirmation() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(
            SlotStatus::UnknownWithoutCertificate {
                metadata_error: PivError::GenericError,
            },
            SlotStatus::QosProvisioned,
        );

        let error = config
            .enroll_yubikey(
                serial(),
                &mut device,
                &default_pin(),
                CertlessSlotOverwrite::Refuse,
            )
            .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::UnknownWithoutCertificate {
                slot: QosSlot::Signing,
                metadata_error: PivError::GenericError,
            }
        ));
        assert_eq!(device.provision_calls, Vec::new());
    }

    #[test]
    fn confirmed_enrollment_overwrites_known_and_unknown_certless_slots() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(
            SlotStatus::KeyWithoutCertificate,
            SlotStatus::UnknownWithoutCertificate {
                metadata_error: PivError::GenericError,
            },
        );

        let enrolled = config
            .enroll_yubikey(
                serial(),
                &mut device,
                &default_pin(),
                CertlessSlotOverwrite::Confirmed,
            )
            .unwrap();

        assert_eq!(
            enrolled.provisioned_slots,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
        assert_eq!(
            device.provision_calls,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
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

    /// Full provision-inspect-delete cycle against real hardware.
    ///
    /// DESTRUCTIVE: generates keys in, and deletes the certificates from,
    /// the PIV signing and key-agreement slots of the sole connected
    /// YubiKey. Requires the factory-default PIN and management key, and
    /// firmware 5.3+ (the metadata command proves slot emptiness). Run
    /// manually: `cargo test -p tvc --lib -- --ignored hardware_`
    #[test]
    #[ignore = "requires a connected YubiKey with default PIN/management key; overwrites its QuorumOS slots"]
    fn hardware_provision_and_delete_cycle() {
        let mut yubikey = open(sole_connected_serial()).unwrap();
        let pin = default_pin();

        let status = yubikey.device_status().unwrap();

        status
            .slots_to_provision(CertlessSlotOverwrite::Confirmed)
            .unwrap()
            .iter()
            .try_for_each(|slot| yubikey.provision_slot(*slot, &pin))
            .unwrap();
        assert_eq!(
            yubikey.device_status().unwrap(),
            DeviceStatus {
                signing: SlotStatus::QosProvisioned,
                key_agreement: SlotStatus::QosProvisioned,
            }
        );

        let pair_public_key = yubikey.pair_public_key().unwrap();
        assert_eq!(pair_public_key.to_string().len(), 260);

        yubikey.delete_qos_certificate(QosSlot::Signing).unwrap();
        yubikey
            .delete_qos_certificate(QosSlot::KeyAgreement)
            .unwrap();

        // The keys remain (no per-slot key deletion), so the slots read as
        // key-without-certificate rather than empty.
        assert_eq!(
            yubikey.device_status().unwrap(),
            DeviceStatus {
                signing: SlotStatus::KeyWithoutCertificate,
                key_agreement: SlotStatus::KeyWithoutCertificate,
            }
        );
    }

    /// Signing and key agreement against real hardware.
    ///
    /// DESTRUCTIVE: provisions the QuorumOS slots of the sole connected
    /// YubiKey when they are empty, and leaves them provisioned. Requires
    /// the factory-default PIN and management key; sign and key agreement
    /// each need a touch while the device blinks. Run manually:
    /// `cargo test -p tvc --lib -- --ignored hardware_`
    #[test]
    #[ignore = "requires a connected YubiKey with default PIN/management key; provisions its QuorumOS slots"]
    fn hardware_sign_and_key_agreement() {
        let mut yubikey = open(sole_connected_serial()).unwrap();
        let pin = default_pin();

        let absent = YubiKeySerial::from(0x0000_0001);
        assert!(matches!(open(absent), Err(DeviceError::NotFound { .. })));

        let status = yubikey.device_status().unwrap();

        status
            .slots_to_provision(CertlessSlotOverwrite::Confirmed)
            .unwrap()
            .iter()
            .try_for_each(|slot| yubikey.provision_slot(*slot, &pin))
            .unwrap();

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
    /// a provisioned device — run `hardware_sign_and_key_agreement` first.
    #[test]
    #[ignore = "requires a provisioned YubiKey with default PIN; burns and restores one PIN retry"]
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
