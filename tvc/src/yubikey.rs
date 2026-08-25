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
//! AND no key per the PIV metadata command (firmware 5.3+); anything
//! ambiguous is a refusal, not a guess.
//!
//! Deletion removes the QuorumOS certificates. The `yubikey` crate offers no
//! way to delete a slot's private key short of a full PIV reset, which would
//! erase credentials TVC does not own — so the key material itself stays
//! behind until the slot is reprovisioned, and TVC never resets a device.

use crate::config::turnkey::{
    Config, QosOperatorPublicKey, QosOperatorPublicKeyParseError, Registration, YubiKeyRegistry,
    YubiKeySerial,
};
use crate::prompts;
use p256::PublicKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use qos_client::{
    yubikey::{KEY_AGREEMENT_SLOT, SIGNING_SLOT, YubiKeyError},
    yubikey_crate as yubikey,
};
use std::fmt::{self, Display, Formatter};
use yubikey::{
    Error as PivError, MgmKey, Serial, TouchPolicy, YubiKey,
    certificate::Certificate,
    piv::{self, SlotId},
    reader::Context,
};
use zeroize::Zeroizing;

/// The subject of the self-signed certificates `qos_client` provisions.
// TODO: qos_client keeps this constant private; have it exported (along with
// a re-export of the yubikey crate) and use it here.
const QOS_CERTIFICATE_SUBJECT: &str = "CN=QuorumOS";

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

/// Source selected for a new YubiKey operator record.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum YubiKeySource {
    Registered(YubiKeySerial),
    Provision(YubiKeySerial),
}

impl YubiKeySource {
    /// Choose an existing registry entry or lazily discover the connected
    /// device to provision. Selecting a registered serial performs no
    /// hardware discovery.
    pub(crate) fn prompt<F>(yubikeys: &YubiKeyRegistry, discover: F) -> anyhow::Result<Self>
    where
        F: FnOnce() -> Result<Vec<YubiKeySerial>, DeviceError>,
    {
        enum Choice {
            Registered(YubiKeySerial),
            New,
        }

        impl Display for Choice {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    Self::Registered(serial) => write!(f, "{serial} (registered)"),
                    Self::New => f.write_str("[new] Provision a connected YubiKey"),
                }
            }
        }

        let choice = if yubikeys.is_empty() {
            Choice::New
        } else {
            let mut choices = yubikeys
                .serials()
                .map(Choice::Registered)
                .collect::<Vec<_>>();
            choices.push(Choice::New);
            prompts::select("YubiKey to use as the operator", choices)?
        };

        match choice {
            Choice::Registered(serial) => Ok(Self::Registered(serial)),
            Choice::New => Ok(Self::Provision(
                ConnectedYubiKeys::from(discover()?).choose(None)?,
            )),
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
                | SlotStatus::KeyWithoutCertificate => None,
            })
    }

    pub(crate) fn slots_with(&self, wanted: SlotStatus) -> Vec<QosSlot> {
        self.slots()
            .into_iter()
            .filter(|(_, status)| **status == wanted)
            .map(|(slot, _)| slot)
            .collect()
    }
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
    /// The slot state cannot be attributed: a key exists, but there is no
    /// readable QuorumOS certificate proving TVC manages it.
    #[error(
        "the {slot} slot holds a key but no readable QuorumOS certificate; refusing to touch \
         this device (clear the slot with an external PIV tool if the key is not in use)"
    )]
    OccupiedWithoutCertificate { slot: QosSlot },
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
            SlotStatus::Empty | SlotStatus::KeyWithoutCertificate => {
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
    ) -> Result<EnrolledYubiKey, DeviceError> {
        let status = device.device_status()?;

        if let Some(refusal) = status.unprovisionable_slot() {
            return Err(refusal);
        }

        let provisioned_slots = status.slots_with(SlotStatus::Empty);
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
    use p256::PublicKey;
    use p256::ecdh::diffie_hellman;
    use qos_p256::{P256Pair, P256Public};

    fn default_pin() -> Pin {
        Pin::from(String::from_utf8(PIN.to_vec()).unwrap())
    }

    #[test]
    fn enroll_provisions_and_registers_a_fresh_device() {
        let mut config = Config::default();
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let enrolled = config
            .enroll_yubikey(serial(), &mut device, &default_pin())
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
            .enroll_yubikey(serial(), &mut device, &default_pin())
            .unwrap();

        assert_eq!(enrolled.registration, Registration::Unchanged);
        assert_eq!(enrolled.provisioned_slots, Vec::new());
        assert_eq!(device.provision_calls, Vec::new());
    }

    #[test]
    fn an_empty_registry_provisions_the_connected_device_without_prompting() {
        let source = YubiKeySource::prompt(&Default::default(), || Ok(vec![serial()])).unwrap();

        assert_eq!(source, YubiKeySource::Provision(serial()));
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
    fn an_unprovable_key_state_is_a_metadata_error() {
        let error = classify_slot(QosSlot::Signing, Err(PivError::InvalidObject), || {
            Err(PivError::NotSupported)
        })
        .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ReadSlotMetadata {
                slot: QosSlot::Signing,
                ..
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
            .slots_with(SlotStatus::Empty)
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
            .slots_with(SlotStatus::Empty)
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
