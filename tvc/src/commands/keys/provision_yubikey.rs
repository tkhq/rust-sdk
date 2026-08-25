//! YubiKey provisioning command - generates the QuorumOS operator key pair
//! on a device and registers its serial.

use crate::{
    commands::Run,
    config::turnkey::{Config, QosOperatorPublicKey, Registration, YubiKeySerial},
    outcome::Outcome,
    output::StdCtx,
    prompts,
    yubikey::{self, ConnectedYubiKeys, Pin, QosSlot},
};
use anyhow::{Context, Result, bail};
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
    type Outcome = YubiKeyProvisioned;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubiKeyProvisioned> {
        // No non-interactive fallback exists: the PIN is prompted and the
        // QuorumOS touch policy demands physical touches during generation.
        if ctx.is_non_interactive() || !prompts::stdin_can_prompt() {
            bail!(
                "provisioning a YubiKey is interactive: the PIN is prompted and \
                 the device must be touched during key generation"
            );
        }

        let serial = ConnectedYubiKeys::from(ctx.connected_yubikeys()?).choose(self.serial)?;
        let pin = Pin::from(prompts::password(
            "YubiKey PIV PIN (the factory default is 123456; touch the device each time it blinks)",
        )?);

        // The one open handle: inspection and mutation see the same device.
        let mut yubikey = yubikey::open(serial)?;
        let enrolled = config.enroll_yubikey(serial, &mut yubikey, &pin)?;

        if enrolled.registration != Registration::Unchanged {
            let recovery = config.to_toml().context(
                "the YubiKey is provisioned but the updated config could not be serialized",
            )?;

            config.save().await.with_context(|| {
                format!(
                    r#"the YubiKey is provisioned but saving the config failed; write this complete config to tvc.config.toml:

{recovery}"#
                )
            })?;
        }

        Ok(YubiKeyProvisioned {
            serial,
            operator_public_key: enrolled.public_key,
            provisioned_slots: enrolled.provisioned_slots,
            registration: enrolled.registration,
        })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubiKeyProvisioned {
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

impl From<YubiKeyProvisioned> for Outcome {
    fn from(provisioned: YubiKeyProvisioned) -> Self {
        Outcome::YubikeyProvisioned(provisioned)
    }
}

impl Display for YubiKeyProvisioned {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn provisioned() -> YubiKeyProvisioned {
        YubiKeyProvisioned {
            serial: YubiKeySerial::from(0x01c9_5c1f),
            operator_public_key: QosOperatorPublicKey::try_from([7u8; 130].as_slice()).unwrap(),
            provisioned_slots: vec![QosSlot::Signing, QosSlot::KeyAgreement],
            registration: Registration::Added,
        }
    }

    #[test]
    fn outcome_serializes_expected_json() {
        assert_eq!(
            serde_json::to_value(Outcome::from(provisioned())).unwrap(),
            serde_json::json!({
                "reason": "yubikey_provisioned",
                "serial": "01c95c1f",
                "operatorPublicKey": "07".repeat(130),
                "provisionedSlots": ["signing", "key_agreement"],
                "registration": "added",
            })
        );
    }

    #[test]
    fn rendering_distinguishes_fresh_and_existing_provisioning() {
        let fresh = provisioned().to_string();
        assert!(fresh.starts_with("YubiKey provisioned!"));
        assert!(fresh.contains("Serial registered in the tvc config."));

        let existing = YubiKeyProvisioned {
            provisioned_slots: Vec::new(),
            registration: Registration::Unchanged,
            ..provisioned()
        };
        let existing = existing.to_string();
        assert!(existing.starts_with("YubiKey already provisioned"));
        assert!(existing.contains("was already registered"));
    }
}
