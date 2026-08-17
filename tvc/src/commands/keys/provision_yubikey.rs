//! YubiKey provisioning command - generates the QuorumOS operator key pair
//! on a device and registers its serial.

use crate::{
    commands::Run,
    config::turnkey::{Config, QosOperatorPublicKey, YubiKeySerial, config_file_path},
    outcome::Outcome,
    output::StdCtx,
    prompts, shell_println,
    yubikey::{DeviceOps, PcscDevices, Pin, QosSlot, Registration},
};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// Provision a YubiKey with the QuorumOS operator key pair and register its
/// serial in the tvc config.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Serial (hex) of the YubiKey to provision.
    /// Defaults to the sole connected device.
    #[arg(long, value_name = "SERIAL")]
    serial: Option<YubiKeySerial>,
}

impl Run for Args {
    type Outcome = YubikeyProvisioned;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubikeyProvisioned> {
        // No non-interactive fallback exists: the PIN is prompted and the
        // QuorumOS touch policy demands physical touches during generation.
        if ctx.is_non_interactive() || !prompts::stdin_can_prompt() {
            bail!(
                "provisioning a YubiKey is interactive: the PIN is prompted and \
                 the device must be touched during key generation"
            );
        }

        let mut devices = PcscDevices;
        let connected = devices.connected_serials()?;

        let serial = match self.serial {
            Some(serial) => {
                ensure!(
                    connected.contains(&serial),
                    "YubiKey {serial} is not connected{}",
                    connected_list(&connected)
                );
                serial
            }
            None => match connected.as_slice() {
                [] => bail!("no YubiKey is connected"),
                [sole] => *sole,
                _ => prompts::select("YubiKey to provision", connected)?,
            },
        };

        let pin = Pin::from(prompts::password(
            "YubiKey PIV PIN (the factory default is 123456)",
        )?);

        shell_println!(ctx, "Touch the YubiKey each time it blinks.")?;

        let provisioned = devices.ensure_provisioned(serial, &pin)?;
        let registration = config.register_yubikey(serial, provisioned.pair_public_key);

        if registration != Registration::Unchanged {
            let config_path = config_file_path()?;
            config.save().await.with_context(|| {
                format!(
                    r#"the YubiKey is provisioned but saving the config failed; register it manually by adding this to {}:

[[yubikeys]]
serial = "{serial}"
public_key = "{}""#,
                    config_path.display(),
                    provisioned.pair_public_key
                )
            })?;
        }

        Ok(YubikeyProvisioned {
            serial,
            operator_public_key: provisioned.pair_public_key,
            provisioned_slots: provisioned.provisioned_slots,
            registration,
        })
    }
}

/// Renders `; connected: a, b` for a mismatch error, or nothing when no
/// device is present (the bare message already says it all).
fn connected_list(connected: &[YubiKeySerial]) -> String {
    if connected.is_empty() {
        return String::new();
    }

    let serials: Vec<String> = connected.iter().map(ToString::to_string).collect();
    format!("; connected: {}", serials.join(", "))
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubikeyProvisioned {
    serial: YubiKeySerial,
    /// The composite `encrypt_public ‖ sign_public` operator key.
    operator_public_key: QosOperatorPublicKey,
    /// Slots newly provisioned by this run; empty when the key pair already
    /// existed on the device.
    provisioned_slots: Vec<QosSlot>,
    /// How the registry changed: newly added, stale key refreshed, or
    /// already current.
    registration: Registration,
}

impl From<YubikeyProvisioned> for Outcome {
    fn from(provisioned: YubikeyProvisioned) -> Self {
        Outcome::YubikeyProvisioned(provisioned)
    }
}

impl Display for YubikeyProvisioned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let summary = if self.provisioned_slots.is_empty() {
            "YubiKey already provisioned - keeping the existing key pair."
        } else {
            "YubiKey provisioned!"
        };
        let registry = match self.registration {
            Registration::Added => "Serial registered in the tvc config.",
            Registration::Updated => "Registry entry refreshed - its cached public key was stale.",
            Registration::Unchanged => "Serial was already registered in the tvc config.",
        };

        write!(
            f,
            r#"{summary}

Serial:              {}
Operator public key: {}

{registry} The private keys exist only on the device and are
protected by its PIN and touch policies."#,
            self.serial, self.operator_public_key
        )
    }
}
