//! In-memory device fake shared by unit tests across the crate.

use super::{DeviceError, DeviceOps, DeviceStatus, Pin, QosSlot, SlotStatus};
use crate::config::turnkey::{QosOperatorPublicKey, YubiKeySerial};
use p256::PublicKey;
use p256::ecdh::diffie_hellman;
use qos_client::yubikey::YubiKeyError;
use qos_p256::P256Pair;
use zeroize::Zeroizing;

/// The PIN the fake accepts — the factory default, matching the hardware
/// tests.
pub(crate) const PIN: &[u8] = qos_client::yubikey::DEFAULT_PIN;

/// Stable serial used by tests that register the fake as a device.
pub(crate) fn serial() -> YubiKeySerial {
    YubiKeySerial::from(0x01c9_5c1f)
}

/// In-memory [`DeviceOps`] implementation: per-slot state and a software
/// P-256 pair standing in for the on-device keys, plus scriptable primitive
/// failures, recording every mutating call.
pub(crate) struct FakeDevice {
    status: DeviceStatus,
    pair: P256Pair,
    pub(crate) fail_provision: Option<QosSlot>,
    pub(crate) fail_delete: Option<QosSlot>,
    pub(crate) provision_calls: Vec<QosSlot>,
    pub(crate) delete_calls: Vec<QosSlot>,
}

impl FakeDevice {
    pub(crate) fn new(signing: SlotStatus, key_agreement: SlotStatus) -> Self {
        Self {
            status: DeviceStatus {
                signing,
                key_agreement,
            },
            pair: P256Pair::generate().expect("software key generation"),
            fail_provision: None,
            fail_delete: None,
            provision_calls: Vec::new(),
            delete_calls: Vec::new(),
        }
    }

    /// The composite operator key of the fake's on-device pair.
    pub(crate) fn operator_public_key(&self) -> QosOperatorPublicKey {
        QosOperatorPublicKey::try_from(self.pair.public_key().to_bytes().as_slice())
            .expect("software composite key is well-formed")
    }

    fn slot_status_mut(&mut self, slot: QosSlot) -> &mut SlotStatus {
        match slot {
            QosSlot::Signing => &mut self.status.signing,
            QosSlot::KeyAgreement => &mut self.status.key_agreement,
        }
    }

    fn checked_slot(&mut self, pin: &Pin, slot: QosSlot) -> Result<(), DeviceError> {
        match self.slot_status(slot)? {
            SlotStatus::QosProvisioned => {}
            SlotStatus::Empty => return Err(DeviceError::EmptySlot { slot }),
            SlotStatus::KeyWithoutCertificate => {
                return Err(DeviceError::OccupiedWithoutCertificate { slot });
            }
            SlotStatus::Foreign { subject } => {
                return Err(DeviceError::ForeignSlot { slot, subject });
            }
        }

        if pin.as_bytes() != PIN {
            return Err(DeviceError::WrongPin { tries: 3 });
        }

        Ok(())
    }
}

impl DeviceOps for FakeDevice {
    fn slot_status(&mut self, slot: QosSlot) -> Result<SlotStatus, DeviceError> {
        Ok(self.slot_status_mut(slot).clone())
    }

    fn device_status(&mut self) -> Result<DeviceStatus, DeviceError> {
        Ok(self.status.clone())
    }

    fn provision_slot(&mut self, slot: QosSlot, _pin: &Pin) -> Result<(), DeviceError> {
        self.provision_calls.push(slot);

        if self.fail_provision == Some(slot) {
            return Err(DeviceError::Provision {
                slot,
                error: YubiKeyError::WillNotOverwriteSlot,
            });
        }

        *self.slot_status_mut(slot) = SlotStatus::QosProvisioned;
        Ok(())
    }

    fn pair_public_key(&mut self) -> Result<QosOperatorPublicKey, DeviceError> {
        if self.status
            == (DeviceStatus {
                signing: SlotStatus::QosProvisioned,
                key_agreement: SlotStatus::QosProvisioned,
            })
        {
            Ok(self.operator_public_key())
        } else {
            Err(DeviceError::ReadPairPublicKey {
                error: YubiKeyError::CannotFindSigningKey,
            })
        }
    }

    fn sign(&mut self, pin: &Pin, message: &[u8]) -> Result<Vec<u8>, DeviceError> {
        self.checked_slot(pin, QosSlot::Signing)?;
        Ok(self.pair.sign(message).expect("software P-256 signing"))
    }

    fn key_agreement(
        &mut self,
        pin: &Pin,
        sender_public: &[u8],
    ) -> Result<Zeroizing<Vec<u8>>, DeviceError> {
        self.checked_slot(pin, QosSlot::KeyAgreement)?;

        let sender =
            PublicKey::from_sec1_bytes(sender_public).map_err(|_| DeviceError::KeyAgreement {
                error: YubiKeyError::KeyAgreementFailed,
            })?;
        let secret = diffie_hellman(
            self.pair.encryption_key().to_nonzero_scalar(),
            sender.as_affine(),
        );

        Ok(Zeroizing::new(secret.raw_secret_bytes().to_vec()))
    }

    fn delete_qos_certificate(&mut self, slot: QosSlot) -> Result<(), DeviceError> {
        self.delete_calls.push(slot);

        if self.fail_delete == Some(slot) {
            return Err(DeviceError::DeleteCertificate {
                slot,
                source: yubikey::Error::GenericError,
            });
        }

        match self.slot_status(slot)? {
            SlotStatus::QosProvisioned => {
                *self.slot_status_mut(slot) = SlotStatus::KeyWithoutCertificate;
                Ok(())
            }
            SlotStatus::Foreign { subject } => Err(DeviceError::ForeignSlot { slot, subject }),
            SlotStatus::Empty | SlotStatus::KeyWithoutCertificate => {
                Err(DeviceError::ChangedSinceInspection { slot })
            }
        }
    }
}
