//! YubiKey deletion command - removes a device's QuorumOS certificates and
//! its registry entry.

use crate::{
    commands::Run,
    config::turnkey::{Config, YubiKeySerial, config_file_path},
    outcome::Outcome,
    output::StdCtx,
    prompts::{self, error_required_in_non_interactive},
    shell_eprintln,
    yubikey::{DeviceOps, PcscDevices, QosSlot},
};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// Delete a registered YubiKey's QuorumOS key material and remove it from
/// the tvc config.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Serial (hex) of the registered YubiKey to delete.
    /// If not provided, will prompt interactively.
    #[arg(long, value_name = "SERIAL")]
    serial: Option<YubiKeySerial>,
    /// Skip the confirmation prompt (required to delete in non-interactive mode).
    #[arg(short, long)]
    yes: bool,
}

impl Run for Args {
    type Outcome = YubikeyDeleted;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubikeyDeleted> {
        // Validate inputs before any business logic: non-interactive mode
        // cannot prompt, so it requires --serial (which device) and --yes
        // (confirmation).
        let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

        if !can_prompt {
            if self.serial.is_none() {
                return Err(error_required_in_non_interactive("--serial"));
            }

            if !self.yes {
                return Err(error_required_in_non_interactive("--yes"));
            }
        }

        let registered: Vec<YubiKeySerial> =
            config.yubikeys.iter().map(|entry| entry.serial).collect();

        let serial = match self.serial {
            // The registry names the devices TVC manages; deletion refuses
            // to touch anything else.
            Some(serial) => {
                ensure!(
                    registered.contains(&serial),
                    "YubiKey {serial} is not in the registry"
                );
                serial
            }
            None => match registered.as_slice() {
                [] => bail!("no YubiKeys are registered"),
                [sole] => *sole,
                _ => prompts::select("YubiKey to delete", registered)?,
            },
        };

        // A deleted device must not leave an organization pointing at it.
        let referencing_orgs = config.orgs_referencing_yubikey(serial);

        if !referencing_orgs.is_empty() {
            bail!(
                "YubiKey {serial} is an operator for organization(s) {}; \
                 remove those operator records first",
                referencing_orgs.join(", ")
            );
        }

        // Interactive confirmation. A non-interactive run without --yes was
        // rejected up front, so reaching here with !self.yes means we can
        // prompt.
        if !self.yes {
            shell_eprintln!(ctx, "")?;
            shell_eprintln!(
                ctx,
                "WARNING: This deletes the QuorumOS certificates on YubiKey {serial} and"
            )?;
            shell_eprintln!(
                ctx,
                "removes it from the tvc registry. The on-device operator key becomes"
            )?;
            shell_eprintln!(ctx, "unusable and cannot be recovered.")?;
            shell_eprintln!(ctx, "")?;
            prompts::confirm_or_bail(
                &format!("Delete the QuorumOS key material on YubiKey {serial}?"),
                "deletion",
            )?;
        }

        let mut devices = PcscDevices;
        let deleted = devices.delete_qos_material(serial)?;

        config.deregister_yubikey(serial);
        let config_path = config_file_path()?;
        config.save().await.with_context(|| {
            format!(
                "the device material was deleted but saving the config failed; \
                 manually remove the [[yubikeys]] entry whose serial is {serial} from {}",
                config_path.display()
            )
        })?;

        Ok(YubikeyDeleted {
            serial,
            cleared_slots: deleted.cleared_slots,
        })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubikeyDeleted {
    serial: YubiKeySerial,
    /// Slots whose QuorumOS certificates were deleted; empty when the device
    /// held none.
    cleared_slots: Vec<QosSlot>,
}

impl From<YubikeyDeleted> for Outcome {
    fn from(deleted: YubikeyDeleted) -> Self {
        Outcome::YubikeyDeleted(deleted)
    }
}

impl Display for YubikeyDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cleared = if self.cleared_slots.is_empty() {
            "none - the device held no QuorumOS certificates".to_string()
        } else {
            let slots: Vec<String> = self.cleared_slots.iter().map(ToString::to_string).collect();
            slots.join(", ")
        };

        write!(
            f,
            r#"YubiKey removed from the registry.

Serial:        {}
Cleared slots: {cleared}

PIV private keys cannot be deleted individually, so the key material stays in
its slots until they are reprovisioned. tvc never performs a full PIV reset,
because that would erase credentials it does not manage."#,
            self.serial
        )
    }
}
