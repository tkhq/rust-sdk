//! YubiKey device management for TVC operator keys.
//!
//! [`DeviceOps`] is the TVC-owned boundary over concrete device access. Its
//! required methods are narrow per-device primitives; the reusable management
//! flows — idempotent provisioning and safe deletion — are provided methods
//! on the trait, so every implementation (hardware or an in-memory test
//! fake) gets the same semantics.
//!
//! Provisioning follows the QuorumOS conventions from `qos_client`: two
//! P-256 keys in the PIV signing and key-agreement slots, each under a
//! self-signed `CN=QuorumOS` certificate, PIN policy "always" and touch
//! policy "always". That certificate subject is how TVC recognizes its own
//! material; a slot holding any other certificate is never touched.
//!
//! Deletion removes the QuorumOS certificates. The `yubikey` crate offers no
//! way to delete a slot's private key short of a full PIV reset, which would
//! erase credentials TVC does not own — so the key material itself stays
//! behind until the slot is reprovisioned, and TVC never resets a device.

use crate::config::turnkey::{
    Config, OperatorRecordKind, QosOperatorPublicKey, QosOperatorPublicKeyParseError,
    YubiKeyRegistryEntry, YubiKeySerial,
};
use qos_client::{
    yubikey::{KEY_AGREEMENT_SLOT, SIGNING_SLOT, YubiKeyError},
    yubikey_crate as yubikey,
};
use std::fmt::{self, Display, Formatter};
use yubikey::{
    Error as PivError, MgmKey, Serial, TouchPolicy, YubiKey, certificate::Certificate, piv::SlotId,
    reader::Context,
};
use zeroize::Zeroizing;

/// The subject of the self-signed certificates `qos_client` provisions.
// TODO: qos_client keeps this constant private; have it exported (along with
// a re-export of the yubikey crate) and use it here.
const QOS_CERTIFICATE_SUBJECT: &str = "CN=QuorumOS";

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

/// What a QuorumOS slot on a device currently holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SlotStatus {
    Empty,
    QosProvisioned,
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

    fn foreign_slot(&self) -> Option<(QosSlot, &str)> {
        self.slots()
            .into_iter()
            .find_map(|(slot, status)| match status {
                SlotStatus::Foreign { subject } => Some((slot, subject.as_str())),
                SlotStatus::Empty | SlotStatus::QosProvisioned => None,
            })
    }

    fn slots_with(&self, wanted: SlotStatus) -> Vec<QosSlot> {
        self.slots()
            .into_iter()
            .filter(|(_, status)| **status == wanted)
            .map(|(slot, _)| slot)
            .collect()
    }
}

/// A YubiKey PIV PIN, held only for the duration of a provisioning call and
/// zeroized on drop. Never persisted.
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
    /// A slot holds material TVC does not manage; nothing was modified.
    #[error(
        "the {slot} slot holds a non-QuorumOS certificate ({subject}); refusing to touch this device"
    )]
    ForeignSlot { slot: QosSlot, subject: String },
    // qos_client's YubiKeyError implements neither Display nor Error, so it
    // is rendered by value instead of chained as a source.
    // TODO: derive those in qos_client and turn these fields into #[source].
    #[error("failed to provision the {slot} slot: {error:?}")]
    Provision { slot: QosSlot, error: YubiKeyError },
    #[error("failed to read the operator public key from the device: {error:?}")]
    ReadPairPublicKey { error: YubiKeyError },
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

/// Result of [`DeviceOps::ensure_provisioned`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Provisioned {
    /// The composite `encrypt_public ‖ sign_public` operator key.
    pub pair_public_key: QosOperatorPublicKey,
    /// Slots newly provisioned by this run, in provisioning order; empty
    /// when the expected key pair already existed.
    pub provisioned_slots: Vec<QosSlot>,
}

/// Result of [`DeviceOps::delete_qos_material`].
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DeletedMaterial {
    /// Slots whose QuorumOS certificates were deleted; empty when the device
    /// held none.
    pub cleared_slots: Vec<QosSlot>,
}

/// How [`Config::register_yubikey`] changed the registry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Registration {
    /// The serial was newly registered.
    Added,
    /// The serial was registered with a stale public key; the cache now
    /// matches the device.
    Updated,
    /// The registry already matched the device.
    #[default]
    Unchanged,
}

/// TVC's boundary over YubiKey hardware access.
///
/// Implementations supply the required per-device primitives; the provided
/// methods are the management flows the commands (and the future operator
/// backend) share.
pub(crate) trait DeviceOps {
    /// Serials of the connected YubiKeys, skipping smartcards that are not
    /// YubiKeys.
    fn connected_serials(&mut self) -> Result<Vec<YubiKeySerial>, DeviceError>;

    /// What both QuorumOS slots on the device hold.
    fn status(&mut self, serial: YubiKeySerial) -> Result<DeviceStatus, DeviceError>;

    /// Generate a P-256 key and QuorumOS certificate in one slot, using the
    /// standing QuorumOS policies. Fails on an occupied slot.
    fn provision_slot(
        &mut self,
        serial: YubiKeySerial,
        slot: QosSlot,
        pin: &Pin,
    ) -> Result<(), DeviceError>;

    /// The composite `encrypt_public ‖ sign_public` operator key, read from
    /// the two slot certificates.
    fn pair_public_key(
        &mut self,
        serial: YubiKeySerial,
    ) -> Result<QosOperatorPublicKey, DeviceError>;

    /// Delete one slot's certificate after re-verifying it is
    /// QuorumOS-issued. The slot's private key cannot be deleted (see the
    /// module docs) and stays behind.
    fn delete_qos_certificate(
        &mut self,
        serial: YubiKeySerial,
        slot: QosSlot,
    ) -> Result<(), DeviceError>;

    /// Bring the device to the fully provisioned state and return its
    /// operator key.
    ///
    /// Idempotent: already-provisioned slots are kept, so a re-run reports
    /// the existing key pair and a partial provision (one slot done, one
    /// empty) resumes where it stopped. Refuses before modifying anything
    /// when either slot holds foreign material.
    fn ensure_provisioned(
        &mut self,
        serial: YubiKeySerial,
        pin: &Pin,
    ) -> Result<Provisioned, DeviceError> {
        let status = self.status(serial)?;

        if let Some((slot, subject)) = status.foreign_slot() {
            return Err(DeviceError::ForeignSlot {
                slot,
                subject: subject.to_string(),
            });
        }

        let provisioned_slots = status.slots_with(SlotStatus::Empty);

        provisioned_slots
            .iter()
            .try_for_each(|slot| self.provision_slot(serial, *slot, pin))?;

        Ok(Provisioned {
            pair_public_key: self.pair_public_key(serial)?,
            provisioned_slots,
        })
    }

    /// Delete the device's QuorumOS certificates.
    ///
    /// Refuses before modifying anything when either slot holds foreign
    /// material. A partial failure reports the slot it died on; the already
    /// cleared slot is skipped by a re-run.
    fn delete_qos_material(
        &mut self,
        serial: YubiKeySerial,
    ) -> Result<DeletedMaterial, DeviceError> {
        let status = self.status(serial)?;

        if let Some((slot, subject)) = status.foreign_slot() {
            return Err(DeviceError::ForeignSlot {
                slot,
                subject: subject.to_string(),
            });
        }

        let cleared_slots = status.slots_with(SlotStatus::QosProvisioned);

        cleared_slots
            .iter()
            .try_for_each(|slot| self.delete_qos_certificate(serial, *slot))?;

        Ok(DeletedMaterial { cleared_slots })
    }
}

/// The machine's connected YubiKeys, reached over PC/SC through `qos_client`
/// and the `yubikey` crate.
pub(crate) struct PcscDevices;

impl PcscDevices {
    fn open(serial: YubiKeySerial) -> Result<YubiKey, DeviceError> {
        YubiKey::open_by_serial(Serial(serial.number())).map_err(|source| match source {
            PivError::NotFound => DeviceError::NotFound { serial },
            source => DeviceError::Open { serial, source },
        })
    }
}

fn slot_status(yubikey: &mut YubiKey, slot: QosSlot) -> Result<SlotStatus, DeviceError> {
    match Certificate::read(yubikey, slot.slot_id()) {
        Ok(certificate) => {
            let subject = certificate.subject();

            if subject == QOS_CERTIFICATE_SUBJECT {
                Ok(SlotStatus::QosProvisioned)
            } else {
                Ok(SlotStatus::Foreign { subject })
            }
        }
        // An empty slot surfaces as InvalidObject (the certificate object
        // reads back zero-length) or NotFound depending on firmware.
        Err(PivError::InvalidObject | PivError::NotFound) => Ok(SlotStatus::Empty),
        Err(source) => Err(DeviceError::ReadCertificate { slot, source }),
    }
}

impl DeviceOps for PcscDevices {
    fn connected_serials(&mut self) -> Result<Vec<YubiKeySerial>, DeviceError> {
        let mut context = Context::open().map_err(DeviceError::Discovery)?;
        let readers = context.iter().map_err(DeviceError::Discovery)?;

        // Readers holding non-YubiKey smartcards fail to open; they are not
        // ours to report.
        Ok(readers
            .filter_map(|reader| reader.open().ok())
            .map(|yubikey| YubiKeySerial::from(yubikey.serial().0))
            .collect())
    }

    fn status(&mut self, serial: YubiKeySerial) -> Result<DeviceStatus, DeviceError> {
        let mut yubikey = Self::open(serial)?;

        Ok(DeviceStatus {
            signing: slot_status(&mut yubikey, QosSlot::Signing)?,
            key_agreement: slot_status(&mut yubikey, QosSlot::KeyAgreement)?,
        })
    }

    fn provision_slot(
        &mut self,
        serial: YubiKeySerial,
        slot: QosSlot,
        pin: &Pin,
    ) -> Result<(), DeviceError> {
        let mut yubikey = Self::open(serial)?;

        qos_client::yubikey::generate_signed_certificate(
            &mut yubikey,
            slot.slot_id(),
            pin.as_bytes(),
            MgmKey::default(),
            TouchPolicy::Always,
        )
        .map(drop)
        .map_err(|error| DeviceError::Provision { slot, error })
    }

    fn pair_public_key(
        &mut self,
        serial: YubiKeySerial,
    ) -> Result<QosOperatorPublicKey, DeviceError> {
        let mut yubikey = Self::open(serial)?;
        let bytes = qos_client::yubikey::pair_public_key(&mut yubikey)
            .map_err(|error| DeviceError::ReadPairPublicKey { error })?;

        QosOperatorPublicKey::try_from(bytes.as_slice())
            .map_err(DeviceError::MalformedPairPublicKey)
    }

    fn delete_qos_certificate(
        &mut self,
        serial: YubiKeySerial,
        slot: QosSlot,
    ) -> Result<(), DeviceError> {
        let mut yubikey = Self::open(serial)?;

        // The provided flow inspected the device, but through a separate
        // handle: re-verify against this one so a swapped or reprovisioned
        // device cannot lose a certificate TVC does not manage.
        if let SlotStatus::Foreign { subject } = slot_status(&mut yubikey, slot)? {
            return Err(DeviceError::ForeignSlot { slot, subject });
        }

        yubikey
            .authenticate(MgmKey::default())
            .map_err(DeviceError::Authenticate)?;

        Certificate::delete(&mut yubikey, slot.slot_id())
            .map_err(|source| DeviceError::DeleteCertificate { slot, source })
    }
}

/// YubiKey semantics of the registry [`Config`] holds; the methods live
/// here, with the rest of device management, rather than in the config
/// module.
impl Config {
    /// Record a device and its operator public key in the registry.
    ///
    /// The device is authoritative: when the serial is already registered
    /// with a different key (reprovisioned outside tvc, or a hand-edited
    /// config), the cached key is replaced.
    pub(crate) fn register_yubikey(
        &mut self,
        serial: YubiKeySerial,
        public_key: QosOperatorPublicKey,
    ) -> Registration {
        let Some(entry) = self
            .yubikeys
            .iter_mut()
            .find(|entry| entry.serial == serial)
        else {
            self.yubikeys.push(YubiKeyRegistryEntry {
                serial,
                public_key,
                extra: toml::Table::new(),
            });
            return Registration::Added;
        };

        if entry.public_key == public_key {
            Registration::Unchanged
        } else {
            entry.public_key = public_key;
            Registration::Updated
        }
    }

    /// Remove a serial from the device registry. Returns `false` when it was
    /// not registered.
    pub(crate) fn deregister_yubikey(&mut self, serial: YubiKeySerial) -> bool {
        let count_before = self.yubikeys.len();
        self.yubikeys.retain(|entry| entry.serial != serial);
        self.yubikeys.len() < count_before
    }

    /// Aliases of organizations with an operator record referencing the
    /// serial, sorted for deterministic messages.
    pub(crate) fn orgs_referencing_yubikey(&self, serial: YubiKeySerial) -> Vec<&str> {
        let mut aliases: Vec<&str> = self
            .orgs
            .iter()
            .filter(|(_, org)| {
                org.operators.iter().any(|operator| {
                    matches!(
                        &operator.kind,
                        OperatorRecordKind::Yubikey(record) if record.serial == serial
                    )
                })
            })
            .map(|(alias, _)| alias.as_str())
            .collect();
        aliases.sort_unstable();
        aliases
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{OperatorRecord, OrgConfig, YubiKeyOperatorRecord};
    use std::collections::HashMap;

    fn serial() -> YubiKeySerial {
        YubiKeySerial::from(0x01c9_5c1f)
    }

    fn pair_key() -> QosOperatorPublicKey {
        QosOperatorPublicKey::try_from([7u8; 130].as_slice()).unwrap()
    }

    /// In-memory [`DeviceOps`] implementation: per-slot state plus
    /// scriptable primitive failures, recording every mutating call.
    struct FakeDevice {
        devices: Vec<(YubiKeySerial, DeviceStatus)>,
        fail_provision: Option<QosSlot>,
        fail_delete: Option<QosSlot>,
        provision_calls: Vec<QosSlot>,
        delete_calls: Vec<QosSlot>,
    }

    impl FakeDevice {
        fn new(signing: SlotStatus, key_agreement: SlotStatus) -> Self {
            Self {
                devices: vec![(
                    serial(),
                    DeviceStatus {
                        signing,
                        key_agreement,
                    },
                )],
                fail_provision: None,
                fail_delete: None,
                provision_calls: Vec::new(),
                delete_calls: Vec::new(),
            }
        }

        fn device_status(
            &mut self,
            serial: YubiKeySerial,
        ) -> Result<&mut DeviceStatus, DeviceError> {
            self.devices
                .iter_mut()
                .find(|(candidate, _)| *candidate == serial)
                .map(|(_, status)| status)
                .ok_or(DeviceError::NotFound { serial })
        }

        fn slot_status(status: &mut DeviceStatus, slot: QosSlot) -> &mut SlotStatus {
            match slot {
                QosSlot::Signing => &mut status.signing,
                QosSlot::KeyAgreement => &mut status.key_agreement,
            }
        }
    }

    impl DeviceOps for FakeDevice {
        fn connected_serials(&mut self) -> Result<Vec<YubiKeySerial>, DeviceError> {
            Ok(self.devices.iter().map(|(serial, _)| *serial).collect())
        }

        fn status(&mut self, serial: YubiKeySerial) -> Result<DeviceStatus, DeviceError> {
            self.device_status(serial).map(|status| status.clone())
        }

        fn provision_slot(
            &mut self,
            serial: YubiKeySerial,
            slot: QosSlot,
            _pin: &Pin,
        ) -> Result<(), DeviceError> {
            self.provision_calls.push(slot);

            if self.fail_provision == Some(slot) {
                return Err(DeviceError::Provision {
                    slot,
                    error: YubiKeyError::WillNotOverwriteSlot,
                });
            }

            let status = self.device_status(serial)?;
            *Self::slot_status(status, slot) = SlotStatus::QosProvisioned;
            Ok(())
        }

        fn pair_public_key(
            &mut self,
            serial: YubiKeySerial,
        ) -> Result<QosOperatorPublicKey, DeviceError> {
            let status = self.device_status(serial)?;

            if *status
                == (DeviceStatus {
                    signing: SlotStatus::QosProvisioned,
                    key_agreement: SlotStatus::QosProvisioned,
                })
            {
                Ok(pair_key())
            } else {
                Err(DeviceError::ReadPairPublicKey {
                    error: YubiKeyError::CannotFindSigningKey,
                })
            }
        }

        fn delete_qos_certificate(
            &mut self,
            serial: YubiKeySerial,
            slot: QosSlot,
        ) -> Result<(), DeviceError> {
            self.delete_calls.push(slot);

            if self.fail_delete == Some(slot) {
                return Err(DeviceError::DeleteCertificate {
                    slot,
                    source: PivError::GenericError,
                });
            }

            let status = self.device_status(serial)?;
            *Self::slot_status(status, slot) = SlotStatus::Empty;
            Ok(())
        }
    }

    fn foreign() -> SlotStatus {
        SlotStatus::Foreign {
            subject: "CN=SomeoneElse".to_string(),
        }
    }

    #[test]
    fn provisions_both_slots_of_a_fresh_device() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let provisioned = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap();

        assert_eq!(
            provisioned,
            Provisioned {
                pair_public_key: pair_key(),
                provisioned_slots: vec![QosSlot::Signing, QosSlot::KeyAgreement],
            }
        );
        assert_eq!(
            device.status(serial()).unwrap(),
            DeviceStatus {
                signing: SlotStatus::QosProvisioned,
                key_agreement: SlotStatus::QosProvisioned,
            }
        );
    }

    #[test]
    fn reprovisioning_is_idempotent() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);

        let provisioned = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap();

        assert_eq!(
            provisioned,
            Provisioned {
                pair_public_key: pair_key(),
                provisioned_slots: Vec::new(),
            }
        );
        assert_eq!(device.provision_calls, Vec::new());
    }

    #[test]
    fn resumes_a_partial_provision() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::Empty);

        let provisioned = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap();

        assert_eq!(provisioned.provisioned_slots, vec![QosSlot::KeyAgreement]);
        assert_eq!(device.provision_calls, vec![QosSlot::KeyAgreement]);
    }

    #[test]
    fn refuses_to_provision_over_a_foreign_slot() {
        let mut device = FakeDevice::new(SlotStatus::Empty, foreign());

        let error = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ForeignSlot {
                slot: QosSlot::KeyAgreement,
                ..
            }
        ));
        assert_eq!(device.provision_calls, Vec::new());
    }

    #[test]
    fn a_failed_provision_reports_its_slot_and_resumes() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        device.fail_provision = Some(QosSlot::KeyAgreement);

        let error = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap_err();

        assert!(matches!(
            error,
            DeviceError::Provision {
                slot: QosSlot::KeyAgreement,
                ..
            }
        ));

        // The signing slot kept its key; a re-run finishes the rest.
        device.fail_provision = None;
        let provisioned = device
            .ensure_provisioned(serial(), &Pin::from("123456".to_string()))
            .unwrap();

        assert_eq!(provisioned.provisioned_slots, vec![QosSlot::KeyAgreement]);
    }

    #[test]
    fn deletes_both_qos_certificates() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);

        let deleted = device.delete_qos_material(serial()).unwrap();

        assert_eq!(
            deleted.cleared_slots,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
        assert_eq!(
            device.status(serial()).unwrap(),
            DeviceStatus {
                signing: SlotStatus::Empty,
                key_agreement: SlotStatus::Empty,
            }
        );
    }

    #[test]
    fn deleting_an_unprovisioned_device_clears_nothing() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let deleted = device.delete_qos_material(serial()).unwrap();

        assert_eq!(deleted.cleared_slots, Vec::new());
        assert_eq!(device.delete_calls, Vec::new());
    }

    #[test]
    fn refuses_to_delete_around_a_foreign_slot() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, foreign());

        let error = device.delete_qos_material(serial()).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::ForeignSlot {
                slot: QosSlot::KeyAgreement,
                ..
            }
        ));
        assert_eq!(device.delete_calls, Vec::new());
    }

    #[test]
    fn a_failed_delete_reports_its_slot_and_resumes() {
        let mut device = FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned);
        device.fail_delete = Some(QosSlot::KeyAgreement);

        let error = device.delete_qos_material(serial()).unwrap_err();

        assert!(matches!(
            error,
            DeviceError::DeleteCertificate {
                slot: QosSlot::KeyAgreement,
                ..
            }
        ));

        // The signing certificate is already gone; a re-run clears the rest.
        device.fail_delete = None;
        let deleted = device.delete_qos_material(serial()).unwrap();

        assert_eq!(deleted.cleared_slots, vec![QosSlot::KeyAgreement]);
    }

    #[test]
    fn missing_device_is_a_not_found_error() {
        let mut device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        let absent = YubiKeySerial::from(0xdead_beef);

        let error = device
            .ensure_provisioned(absent, &Pin::from("123456".to_string()))
            .unwrap_err();

        assert!(matches!(error, DeviceError::NotFound { serial } if serial == absent));
    }

    fn org_with_operators(operators: Vec<OperatorRecord>) -> OrgConfig {
        OrgConfig {
            id: "org-id".to_string(),
            api_key_path: "api-key.json".into(),
            api_base_url: "https://api.turnkey.com".to_string(),
            default_operator_kind: Default::default(),
            operators,
            extra: toml::Table::new(),
        }
    }

    fn yubikey_operator(serial: YubiKeySerial) -> OperatorRecord {
        OperatorRecord {
            name: "yubikey".to_string(),
            kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                serial,
                extra: toml::Table::new(),
            }),
        }
    }

    #[test]
    fn registration_is_idempotent() {
        let mut config = Config::default();

        assert_eq!(
            config.register_yubikey(serial(), pair_key()),
            Registration::Added
        );
        assert_eq!(
            config.register_yubikey(serial(), pair_key()),
            Registration::Unchanged
        );
        assert_eq!(config.yubikeys.len(), 1);
        assert_eq!(config.yubikeys[0].serial, serial());
        assert_eq!(config.yubikeys[0].public_key, pair_key());
    }

    #[test]
    fn registration_refreshes_a_stale_cached_key() {
        let mut config = Config::default();
        let stale = QosOperatorPublicKey::try_from([9u8; 130].as_slice()).unwrap();
        config.register_yubikey(serial(), stale);

        assert_eq!(
            config.register_yubikey(serial(), pair_key()),
            Registration::Updated
        );
        assert_eq!(config.yubikeys[0].public_key, pair_key());
    }

    #[test]
    fn deregistration_removes_the_entry() {
        let mut config = Config::default();
        config.register_yubikey(serial(), pair_key());

        assert!(config.deregister_yubikey(serial()));
        assert!(!config.deregister_yubikey(serial()));
        assert_eq!(config.yubikeys, Vec::new());
    }

    #[test]
    fn referencing_orgs_are_found_and_sorted() {
        let config = Config {
            orgs: HashMap::from([
                (
                    "bravo".to_string(),
                    org_with_operators(vec![yubikey_operator(serial())]),
                ),
                (
                    "alpha".to_string(),
                    org_with_operators(vec![yubikey_operator(serial())]),
                ),
                ("other".to_string(), org_with_operators(Vec::new())),
            ]),
            ..Config::default()
        };

        assert_eq!(
            config.orgs_referencing_yubikey(serial()),
            vec!["alpha", "bravo"]
        );
        assert_eq!(
            config.orgs_referencing_yubikey(YubiKeySerial::from(0xdead_beef)),
            Vec::<&str>::new()
        );
    }

    /// Full provision-inspect-delete cycle against real hardware.
    ///
    /// DESTRUCTIVE: generates keys in, and deletes the certificates from,
    /// the PIV signing and key-agreement slots of the sole connected
    /// YubiKey. Requires the factory-default PIN and management key. Run
    /// manually: `cargo test -p tvc --lib -- --ignored hardware_`
    #[test]
    #[ignore = "requires a connected YubiKey with default PIN/management key; overwrites its QuorumOS slots"]
    fn hardware_provision_and_delete_cycle() {
        let mut devices = PcscDevices;

        let connected = devices.connected_serials().unwrap();
        let [serial] = connected.as_slice() else {
            panic!("connect exactly one YubiKey; found {connected:?}");
        };
        let serial = *serial;
        let pin = Pin::from(String::from_utf8(qos_client::yubikey::DEFAULT_PIN.to_vec()).unwrap());

        let provisioned = devices.ensure_provisioned(serial, &pin).unwrap();
        assert_eq!(
            devices.status(serial).unwrap(),
            DeviceStatus {
                signing: SlotStatus::QosProvisioned,
                key_agreement: SlotStatus::QosProvisioned,
            }
        );

        // Idempotent re-run keeps the same key pair.
        let reprovisioned = devices.ensure_provisioned(serial, &pin).unwrap();
        assert_eq!(reprovisioned.pair_public_key, provisioned.pair_public_key);
        assert_eq!(reprovisioned.provisioned_slots, Vec::new());

        let deleted = devices.delete_qos_material(serial).unwrap();
        assert_eq!(
            deleted.cleared_slots,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
        assert_eq!(
            devices.status(serial).unwrap(),
            DeviceStatus {
                signing: SlotStatus::Empty,
                key_agreement: SlotStatus::Empty,
            }
        );
    }
}
