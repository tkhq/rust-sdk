//! YubiKey deletion command - removes a device's QuorumOS certificates and
//! its registry entry.

use crate::{
    commands::Run,
    config::turnkey::{Config, OperatorRecordKind, YubiKeySerial},
    outcome::Outcome,
    output::StdCtx,
    prompts::{self, error_required_in_non_interactive},
    shell_eprintln,
    yubikey::{self, DeviceError, DeviceOps, QosSlot, SlotStatus},
};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// Delete a registered YubiKey's QuorumOS certificates and remove it from
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
    type Outcome = YubiKeyDeleted;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubiKeyDeleted> {
        // Validate inputs before any business logic: non-interactive mode
        // cannot prompt, so it requires --serial (which device) and --yes
        // (confirmation).
        let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

        if !can_prompt && self.serial.is_none() {
            return Err(error_required_in_non_interactive("--serial"));
        }

        if !can_prompt && !self.yes {
            return Err(error_required_in_non_interactive("--yes"));
        }

        let registered = config.yubikeys.serials().collect::<Vec<_>>();

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
        let referencing_orgs = {
            let mut aliases = config
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
                .collect::<Vec<_>>();
            aliases.sort_unstable();
            aliases
        };

        if !referencing_orgs.is_empty() {
            bail!(
                "YubiKey {serial} is an operator for organization(s) {}; \
                 remove those operator records first",
                referencing_orgs.join(", ")
            );
        }

        // Interactive confirmation, literal about what deletion can and
        // cannot do. A non-interactive run without --yes was rejected up
        // front, so reaching here with !self.yes means we can prompt.
        if !self.yes {
            shell_eprintln!(ctx, "")?;
            shell_eprintln!(
                ctx,
                "WARNING: This deletes the QuorumOS certificates on YubiKey {serial} and"
            )?;
            shell_eprintln!(
                ctx,
                "removes it from the tvc registry. PIV private keys cannot be deleted"
            )?;
            shell_eprintln!(
                ctx,
                "individually, so the key material stays in its slots until they are"
            )?;
            shell_eprintln!(ctx, "reprovisioned; tvc never performs a full PIV reset.")?;
            shell_eprintln!(ctx, "")?;
            prompts::confirm_or_bail(
                &format!("Delete the QuorumOS certificates on YubiKey {serial}?"),
                "deletion",
            )?;
        }

        // The one open handle: inspection and mutation see the same device.
        let mut yubikey = yubikey::open(serial)?;
        let status = yubikey.device_status()?;

        if let Some((slot, subject)) = status.foreign_slot() {
            return Err(DeviceError::ForeignSlot {
                slot,
                subject: subject.to_string(),
            }
            .into());
        }

        let cleared_slots = status.slots_with(SlotStatus::QosProvisioned);

        cleared_slots
            .iter()
            .try_for_each(|slot| yubikey.delete_qos_certificate(*slot))?;

        config.yubikeys.deregister(serial);
        config.save().await.with_context(|| {
            format!(
                "the device certificates were deleted but saving the config failed; \
                 manually remove the [[yubikeys]] entry whose serial is {serial}"
            )
        })?;

        Ok(YubiKeyDeleted {
            serial,
            cleared_slots,
        })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubiKeyDeleted {
    serial: YubiKeySerial,
    /// Slots whose QuorumOS certificates were deleted; empty when the device
    /// held none.
    cleared_slots: Vec<QosSlot>,
}

impl From<YubiKeyDeleted> for Outcome {
    fn from(deleted: YubiKeyDeleted) -> Self {
        Outcome::YubikeyDeleted(deleted)
    }
}

impl Display for YubiKeyDeleted {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let cleared = if self.cleared_slots.is_empty() {
            "none - the device held no QuorumOS certificates".to_string()
        } else {
            let slots = self
                .cleared_slots
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            slots.join(", ")
        };

        write!(
            f,
            r#"YubiKey removed from the registry.

Serial:               {}
Certificates deleted: {cleared}

PIV private keys cannot be deleted individually, so the key material stays in
its slots until they are reprovisioned. tvc never performs a full PIV reset,
because that would erase credentials it does not manage."#,
            self.serial
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_serializes_expected_json() {
        let deleted = YubiKeyDeleted {
            serial: YubiKeySerial::from(0x01c9_5c1f),
            cleared_slots: vec![QosSlot::Signing, QosSlot::KeyAgreement],
        };

        assert_eq!(
            serde_json::to_value(Outcome::from(deleted)).unwrap(),
            serde_json::json!({
                "reason": "yubikey_deleted",
                "serial": "01c95c1f",
                "clearedSlots": ["signing", "key_agreement"],
            })
        );
    }

    #[test]
    fn rendering_stays_literal_about_the_limitation() {
        let rendered = YubiKeyDeleted {
            serial: YubiKeySerial::from(0x01c9_5c1f),
            cleared_slots: Vec::new(),
        }
        .to_string();

        assert!(rendered.contains("none - the device held no QuorumOS certificates"));
        assert!(rendered.contains("PIV private keys cannot be deleted individually"));
        assert!(!rendered.contains("unusable"));
        assert!(!rendered.contains("unrecoverable"));
    }
}
