//! YubiKey registry refresh command - re-reads a configured device's
//! operator key and updates the cached registry entry.

use crate::{
    commands::Run,
    config::turnkey::{
        Config, QosOperatorPublicKey, Registration, YubiKeySerial, config_file_path,
    },
    outcome::Outcome,
    output::{Ctx, StdCtx},
    yubikey::{self, DeviceError, DeviceOps, YubiKeySelectionError},
};
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use std::io::Write;

/// Refresh the registry's cached operator key from the device itself.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Serial (hex) of the YubiKey to refresh.
    /// Defaults to the sole connected device.
    #[arg(long, value_name = "SERIAL")]
    serial: Option<YubiKeySerial>,
}

impl Run for Args {
    type Outcome = YubiKeyRefreshed;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubiKeyRefreshed> {
        let refreshed = self.refresh(ctx, yubikey::open, &mut config)?;

        // Remediation renders from the typed registration outcome: a fresh
        // serial is appended, a stale entry is edited in place.
        let manual_fix = match refreshed.registration {
            Registration::Unchanged => None,
            Registration::Added => Some(format!(
                r#"register it manually by adding this to {}:

[[yubikeys]]
serial = "{}"
public_key = "{}""#,
                config_file_path()?.display(),
                refreshed.serial,
                refreshed.operator_public_key,
            )),
            Registration::Updated => Some(format!(
                r#"the cached key is stale; update the existing [[yubikeys]] entry for serial "{}" in {} to:

public_key = "{}""#,
                refreshed.serial,
                config_file_path()?.display(),
                refreshed.operator_public_key,
            )),
        };

        if let Some(manual_fix) = manual_fix {
            config.save().await.with_context(|| {
                format!("the key was read but saving the config failed; {manual_fix}")
            })?;
        }

        Ok(refreshed)
    }
}

impl Args {
    /// The command flow over scripted discovery, any device opener, and any
    /// shell; [`Run::run`] supplies PC/SC and the real terminal, then
    /// persists the config.
    fn refresh<W, W2, D, O>(
        self,
        ctx: &mut Ctx<W, W2>,
        open_device: O,
        config: &mut Config,
    ) -> Result<YubiKeyRefreshed>
    where
        W: Write,
        W2: Write,
        D: DeviceOps,
        O: FnOnce(YubiKeySerial) -> Result<D, DeviceError>,
    {
        let serial = match ctx.connected_yubikeys()?.choose(self.serial) {
            Ok(serial) => serial,
            Err(YubiKeySelectionError::NoneConnected) => bail!("no YubiKey is connected"),
            Err(YubiKeySelectionError::NotConnected {
                requested,
                connected,
            }) => {
                if connected.is_empty() {
                    bail!("YubiKey {requested} is not connected");
                }

                bail!("YubiKey {requested} is not connected; connected: {connected}")
            }
            Err(YubiKeySelectionError::Ambiguous { connected }) => {
                bail!(
                    "multiple YubiKeys are connected (serials {connected}); unplug all but the one \
                     to use and try again, or pass --serial"
                )
            }
        };

        // Reading the slot certificates needs neither the PIN nor a touch,
        // so the refresh itself also runs non-interactively.
        let mut device = open_device(serial)?;
        let operator_public_key = device.verified_pair_public_key()?;
        let registration = config.yubikeys.register(serial, operator_public_key);

        Ok(YubiKeyRefreshed {
            serial,
            operator_public_key,
            registration,
        })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubiKeyRefreshed {
    serial: YubiKeySerial,
    /// The composite `encrypt_public ‖ sign_public` operator key, as read
    /// from the device.
    operator_public_key: QosOperatorPublicKey,
    /// How the registry changed: newly added, stale key refreshed, or
    /// already current.
    registration: Registration,
}

impl From<YubiKeyRefreshed> for Outcome {
    fn from(refreshed: YubiKeyRefreshed) -> Self {
        Outcome::YubikeyRefreshed(refreshed)
    }
}

impl Display for YubiKeyRefreshed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let summary = match self.registration {
            Registration::Added => "Serial was not yet registered - added it to the tvc config.",
            Registration::Updated => "Registry entry refreshed - its cached public key was stale.",
            Registration::Unchanged => "Registry already matches the device - nothing to update.",
        };

        write!(
            f,
            r#"{summary}

Serial:              {}
Operator public key: {}"#,
            self.serial, self.operator_public_key
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::TestShell;
    use crate::yubikey::SlotStatus;
    use crate::yubikey::test_support::{FakeDevice, serial};

    fn test_ctx() -> Ctx<Vec<u8>, Vec<u8>> {
        Ctx::new(TestShell::default(), false).with_yubikey_discovery(|| Ok(vec![serial()].into()))
    }

    fn provisioned_device() -> FakeDevice {
        FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned)
    }

    fn open_fake(
        device: FakeDevice,
    ) -> impl FnOnce(YubiKeySerial) -> Result<FakeDevice, DeviceError> {
        move |requested| {
            if requested == serial() {
                Ok(device)
            } else {
                Err(DeviceError::NotFound { serial: requested })
            }
        }
    }

    #[test]
    fn refresh_adds_an_unregistered_serial() {
        let mut config = Config::default();
        let device = provisioned_device();
        let composite = device.operator_public_key();

        let refreshed = Args { serial: None }
            .refresh(&mut test_ctx(), open_fake(device), &mut config)
            .unwrap();

        assert_eq!(refreshed.serial, serial());
        assert_eq!(refreshed.operator_public_key, composite);
        assert_eq!(refreshed.registration, Registration::Added);
        assert_eq!(config.yubikeys.get(serial()).unwrap().public_key, composite);
    }

    #[test]
    fn refresh_replaces_a_stale_cached_key() {
        let mut config = Config::default();
        let stale = QosOperatorPublicKey::try_from([9u8; 130].as_slice()).unwrap();
        config.yubikeys.register(serial(), stale);
        let device = provisioned_device();
        let composite = device.operator_public_key();

        let refreshed = Args { serial: None }
            .refresh(&mut test_ctx(), open_fake(device), &mut config)
            .unwrap();

        assert_eq!(refreshed.registration, Registration::Updated);
        assert_eq!(config.yubikeys.get(serial()).unwrap().public_key, composite);
    }

    #[test]
    fn refresh_reports_a_current_registry_as_unchanged() {
        let mut config = Config::default();
        let device = provisioned_device();
        config
            .yubikeys
            .register(serial(), device.operator_public_key());

        let refreshed = Args { serial: None }
            .refresh(&mut test_ctx(), open_fake(device), &mut config)
            .unwrap();

        assert_eq!(refreshed.registration, Registration::Unchanged);
    }

    #[test]
    fn an_unconnected_serial_is_refused_with_the_connected_list() {
        let error = Args {
            serial: Some(YubiKeySerial::from(0xdead_beef)),
        }
        .refresh(
            &mut test_ctx(),
            open_fake(provisioned_device()),
            &mut Config::default(),
        )
        .err()
        .unwrap();

        assert_eq!(
            error.to_string(),
            "YubiKey deadbeef is not connected; connected: 01c95c1f"
        );
    }

    #[test]
    fn an_unprovisioned_device_is_refused() {
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let error = Args { serial: None }
            .refresh(&mut test_ctx(), open_fake(device), &mut Config::default())
            .err()
            .unwrap();

        assert!(format!("{error:#}").contains("holds no QuorumOS key"));
    }
}
