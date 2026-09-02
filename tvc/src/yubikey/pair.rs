//! The YubiKey-backed operator pair.
//!
//! [`YubiKeyPair`] implements the [`Signer`] and [`Pair`] ports over a
//! externally configured device: signing uses the signing-slot key, and decryption
//! runs the key-agreement slot's ECDH under the qos p256 envelope scheme.
//! PC/SC calls block, so they run on the blocking thread pool, and a mutex
//! serializes access to the one device.

use super::{DeviceOps, Pin, SlotsError};
use crate::config::turnkey::{Config, QosOperatorPublicKey, YubiKeySerial};
use crate::pair::{Pair, QosP256Error, Signer, SignerFuture};
use anyhow::{Context, anyhow, ensure};
use p256::PublicKey;
use qos_p256::encrypt::{Envelope, P256EncryptPublic};
use std::fmt::{self, Debug, Formatter};
use std::sync::{Arc, Mutex, PoisonError};
use tokio::task::spawn_blocking;
use zeroize::Zeroizing;

/// A resolved, device-verified YubiKey operator pair.
pub(crate) struct YubiKeyPair<D> {
    device: Arc<Mutex<D>>,
    serial: YubiKeySerial,
    /// The device-verified composite `encrypt_public ‖ sign_public` key.
    public_key: QosOperatorPublicKey,
    /// The composite's encryption half, parsed once at resolution — holding
    /// the pair is proof its key is a valid P-256 point.
    encrypt_public: P256EncryptPublic,
    pin: Arc<Pin>,
}

// The PIN must never reach debug output, so the derive is off the table.
impl<D> Debug for YubiKeyPair<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("YubiKeyPair")
            .field("serial", &self.serial)
            .field("public_key", &self.public_key)
            .finish_non_exhaustive()
    }
}

impl<D: DeviceOps + Send + 'static> YubiKeyPair<D> {
    /// Run one blocking device operation on the blocking thread pool; the
    /// mutex serializes access to the device.
    async fn device_op<T, E>(
        device: Arc<Mutex<D>>,
        operation: impl FnOnce(&mut D) -> Result<T, E> + Send + 'static,
    ) -> anyhow::Result<T>
    where
        T: Send + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let result = spawn_blocking(move || {
            let mut device = device.lock().unwrap_or_else(PoisonError::into_inner);
            operation(&mut device)
        })
        .await
        .context("YubiKey device task failed")?;

        Ok(result?)
    }
}

impl<D: DeviceOps + Send + 'static> Signer for YubiKeyPair<D> {
    fn sign(&self, message: &[u8]) -> SignerFuture<'_, anyhow::Result<Vec<u8>>> {
        let device = Arc::clone(&self.device);
        let pin = Arc::clone(&self.pin);
        let message = message.to_vec();

        Box::pin(
            async move { Self::device_op(device, move |device| device.sign(&pin, &message)).await },
        )
    }

    fn public_key(&self) -> Vec<u8> {
        self.public_key.as_bytes().to_vec()
    }
}

impl<D: DeviceOps + Send + 'static> Pair for YubiKeyPair<D> {
    fn decrypt(&self, ciphertext: &[u8]) -> SignerFuture<'_, anyhow::Result<Zeroizing<Vec<u8>>>> {
        let device = Arc::clone(&self.device);
        let pin = Arc::clone(&self.pin);
        let encrypt_public = &self.encrypt_public;
        let ciphertext = ciphertext.to_vec();

        Box::pin(async move {
            // Parse everything before any device I/O, so malformed input
            // fails before the device demands its PIN and a touch.
            let envelope = borsh::from_slice::<Envelope>(&ciphertext)
                .context("ciphertext is not a qos p256 encryption envelope")?;
            let sender_public = PublicKey::from_sec1_bytes(&envelope.ephemeral_sender_public)
                .context("the envelope's ephemeral sender key is not a valid P-256 point")?;

            let shared_secret = Self::device_op(device, move |device| {
                device.key_agreement(&pin, sender_public)
            })
            .await?;

            encrypt_public
                .decrypt_from_shared_secret(&ciphertext, &shared_secret)
                .map_err(QosP256Error)
                .context("failed to decrypt the envelope with the YubiKey shared secret")
        })
    }
}

/// YubiKey resolution semantics of the registry [`Config`] holds; the method
/// lives here, with the pair it produces.
impl Config {
    /// Resolve a registered serial into a usable operator pair.
    ///
    /// Refusals come in resolution order: an unregistered serial, a missing
    /// or incompletely configured device, and a registry cache that no longer matches
    /// the device. The endpoint supplies the PIN before dispatch.
    pub(crate) async fn resolve_yubikey<D: DeviceOps + Send + 'static>(
        &self,
        serial: YubiKeySerial,
        device: D,
        pin: Pin,
    ) -> anyhow::Result<YubiKeyPair<D>> {
        let cached_public_key = self
            .yubikeys
            .get(serial)
            .map(|entry| entry.public_key)
            .ok_or_else(|| {
                anyhow!(
                    "YubiKey {serial} is not in the device registry; install its certificates \
                     and run `tvc keys refresh-yubikey --serial {serial}` first"
                )
            })?;

        let device = Arc::new(Mutex::new(device));
        let public_key = {
            let device = Arc::clone(&device);

            YubiKeyPair::device_op(device, DeviceOps::verified_pair_public_key)
                .await
                .map_err(|error| match error.downcast_ref::<SlotsError>() {
                    Some(slots) if slots.contains_empty_slot() => error.context(format!(
                        "YubiKey {serial} is not fully configured; generate its keys and install \
                         its certificates with `ykman`, then run \
                         `tvc keys refresh-yubikey --serial {serial}`"
                    )),
                    _ => error,
                })?
        };

        ensure!(
            public_key == cached_public_key,
            "YubiKey {serial} holds operator key {public_key} but the registry caches \
             {cached_public_key}; run `tvc keys refresh-yubikey --serial {serial}` to refresh it"
        );

        let encrypt_public = P256EncryptPublic::from_bytes(public_key.encrypt_public_bytes())
            .map_err(QosP256Error)
            .context("the device's operator key is not a valid encryption point")?;

        Ok(YubiKeyPair {
            device,
            serial,
            public_key,
            encrypt_public,
            pin: Arc::new(pin),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yubikey::SlotStatus;
    use crate::yubikey::test_support::{FakeDevice, PIN, serial};
    use qos_p256::P256Public;

    fn provisioned_device() -> FakeDevice {
        FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned)
    }

    fn registered_config(device: &FakeDevice) -> Config {
        let mut config = Config::default();
        config
            .yubikeys
            .register(serial(), device.operator_public_key());
        config
    }

    fn fixed_pin() -> Pin {
        Pin::from(String::from_utf8(PIN.to_vec()).unwrap())
    }

    fn wrong_pin() -> Pin {
        Pin::from("999999".to_string())
    }

    fn rendered(error: &anyhow::Error) -> String {
        format!("{error:#}")
    }

    #[tokio::test]
    async fn resolves_a_registered_provisioned_device() {
        let device = provisioned_device();
        let composite = device.operator_public_key();
        let config = registered_config(&device);

        let pair = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap();

        assert_eq!(pair.public_key(), composite.as_bytes());
    }

    #[tokio::test]
    async fn an_unregistered_serial_points_at_registration() {
        let error = Config::default()
            .resolve_yubikey(serial(), provisioned_device(), fixed_pin())
            .await
            .unwrap_err();

        assert!(rendered(&error).contains("not in the device registry"));
        assert!(rendered(&error).contains("refresh-yubikey"));
    }

    #[tokio::test]
    async fn an_incompletely_configured_device_points_at_external_setup() {
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        let config = registered_config(&device);

        let error = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap_err();

        assert!(rendered(&error).contains("not fully configured"));
        assert!(rendered(&error).contains("refresh-yubikey"));
    }

    #[tokio::test]
    async fn a_foreign_slot_is_refused() {
        let device = FakeDevice::new(
            SlotStatus::Foreign {
                subject: "CN=SomeoneElse".to_string(),
            },
            SlotStatus::QosProvisioned,
        );
        let config = registered_config(&device);

        let error = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap_err();

        assert!(rendered(&error).contains("non-QuorumOS certificate"));
    }

    #[tokio::test]
    async fn a_stale_registry_cache_is_a_hard_error_naming_the_refresh_command() {
        let device = provisioned_device();
        let mut config = Config::default();
        let stale = QosOperatorPublicKey::try_from([9u8; 130].as_slice()).unwrap();
        config.yubikeys.register(serial(), stale);

        let error = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap_err();

        assert!(rendered(&error).contains("refresh-yubikey"));
    }

    #[tokio::test]
    async fn a_wrong_pin_surfaces_on_first_use_with_the_retry_count() {
        let device = provisioned_device();
        let config = registered_config(&device);
        let pair = config
            .resolve_yubikey(serial(), device, wrong_pin())
            .await
            .unwrap();

        let error = pair.sign(b"message").await.unwrap_err();

        assert!(rendered(&error).contains("PIN was rejected"));
    }

    #[tokio::test]
    async fn signs_a_message_verifiable_with_the_composite_key() {
        let device = provisioned_device();
        let composite = device.operator_public_key();
        let config = registered_config(&device);
        let pair = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap();
        let message = b"manifest hash stand-in";

        let signature = pair.sign(message).await.unwrap();

        P256Public::from_bytes(composite.as_bytes())
            .unwrap()
            .verify(message, &signature)
            .unwrap();
    }

    #[tokio::test]
    async fn decrypts_an_envelope_addressed_to_the_operator() {
        let device = provisioned_device();
        let composite = device.operator_public_key();
        let config = registered_config(&device);
        let pair = config
            .resolve_yubikey(serial(), device, fixed_pin())
            .await
            .unwrap();
        let plaintext = b"quorum share bytes";
        let ciphertext = P256Public::from_bytes(composite.as_bytes())
            .unwrap()
            .encrypt(plaintext)
            .unwrap();

        let decrypted = pair.decrypt(&ciphertext).await.unwrap();

        assert_eq!(decrypted.as_slice(), plaintext);
    }

    /// Resolution and the full pair round-trip against real hardware.
    ///
    /// Requires an externally configured YubiKey with the factory-default
    /// PIN. Sign and decrypt each need a touch while the device blinks. Run
    /// manually:
    /// `cargo test -p tvc --lib -- --ignored hardware_`
    #[tokio::test]
    #[ignore = "requires an externally configured YubiKey with default PIN; sign and decrypt each need a touch"]
    async fn hardware_resolve_sign_and_decrypt_roundtrip() {
        use crate::yubikey::{connected_serials, open};

        let connected = connected_serials().unwrap();

        let [serial] = connected.as_slice() else {
            panic!("connect exactly one YubiKey; found {connected:?}");
        };

        let serial = *serial;
        let mut device = open(serial).unwrap();
        let key = device.verified_pair_public_key().unwrap();
        let mut config = Config::default();
        config.yubikeys.register(serial, key);

        let pin = Pin::from(String::from_utf8(qos_client::yubikey::DEFAULT_PIN.to_vec()).unwrap());
        let pair = config.resolve_yubikey(serial, device, pin).await.unwrap();

        let message = b"tvc hardware pair test";
        let signature = pair.sign(message).await.unwrap();
        P256Public::from_bytes(key.as_bytes())
            .unwrap()
            .verify(message, &signature)
            .unwrap();

        let ciphertext = P256Public::from_bytes(key.as_bytes())
            .unwrap()
            .encrypt(b"share")
            .unwrap();
        let decrypted = pair.decrypt(&ciphertext).await.unwrap();
        assert_eq!(decrypted.as_slice(), b"share");
    }

    #[tokio::test]
    async fn a_malformed_envelope_fails_before_the_device_is_touched() {
        // The pair holds a wrong PIN: had decryption reached the device, the
        // PIN rejection would surface instead of the envelope parse error.
        let device = provisioned_device();
        let config = registered_config(&device);
        let pair = config
            .resolve_yubikey(serial(), device, wrong_pin())
            .await
            .unwrap();

        let error = pair.decrypt(b"not an envelope").await.unwrap_err();

        assert!(rendered(&error).contains("envelope"));
    }
}
