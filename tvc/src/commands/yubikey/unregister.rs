//! Remove a YubiKey from the local TVC registry without modifying the device.

use crate::{
    commands::Run,
    config::turnkey::{Config, OperatorRecordKind, YubiKeySerial},
    outcome::Outcome,
    output::StdCtx,
    prompts::{self, error_required_in_non_interactive},
    shell_eprintln,
};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

/// Unregister a YubiKey from the local TVC configuration.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Serial (hex) of the registered YubiKey to unregister.
    /// If not provided, prompts interactively.
    #[arg(long, value_name = "SERIAL")]
    serial: Option<YubiKeySerial>,
    /// Skip the confirmation prompt (required in non-interactive mode).
    #[arg(short, long)]
    yes: bool,
}

impl Run for Args {
    type Outcome = YubiKeyUnregistered;

    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<YubiKeyUnregistered> {
        let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

        if !can_prompt && self.serial.is_none() {
            return Err(error_required_in_non_interactive("--serial"));
        }

        if !can_prompt && !self.yes {
            return Err(error_required_in_non_interactive("--yes"));
        }

        let registered = config.yubikeys.serials().collect::<Vec<_>>();
        let serial = match self.serial {
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
                _ => prompts::select("YubiKey to unregister", registered)?,
            },
        };

        let referencing_orgs = {
            let mut names = config
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
                .map(|(org_id, _)| config.display_name(*org_id))
                .collect::<Vec<_>>();
            names.sort_unstable();
            names
        };

        if !referencing_orgs.is_empty() {
            bail!(
                "YubiKey {serial} is an operator for organization(s) {}; \
                 remove those operator records first",
                referencing_orgs.join(", ")
            );
        }

        if !self.yes {
            shell_eprintln!(ctx, "")?;
            shell_eprintln!(
                ctx,
                "WARNING: This only removes YubiKey {serial} from the local TVC configuration."
            )?;
            shell_eprintln!(
                ctx,
                "It does not erase the keys or certificates on the YubiKey, and it does not"
            )?;
            shell_eprintln!(
                ctx,
                "revoke the YubiKey from any organization. The YubiKey remains able to act as"
            )?;
            shell_eprintln!(
                ctx,
                "an operator for every organization that still trusts its public keys."
            )?;
            shell_eprintln!(ctx, "")?;
            prompts::confirm_or_bail(&format!("Unregister YubiKey {serial}?"), "unregistration")?;
        }

        ensure!(
            config.yubikeys.deregister(serial),
            "YubiKey {serial} is not in the registry"
        );
        config
            .save()
            .await
            .with_context(|| format!("failed to unregister YubiKey {serial}"))?;

        Ok(YubiKeyUnregistered { serial })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubiKeyUnregistered {
    serial: YubiKeySerial,
}

impl From<YubiKeyUnregistered> for Outcome {
    fn from(unregistered: YubiKeyUnregistered) -> Self {
        Outcome::YubikeyUnregistered(unregistered)
    }
}

impl Display for YubiKeyUnregistered {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"YubiKey {} was removed from the local TVC configuration.
The device was not modified and no organization operator was revoked."#,
            self.serial
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_serializes_expected_json() {
        let unregistered = YubiKeyUnregistered {
            serial: YubiKeySerial::from(0x01c9_5c1f),
        };

        assert_eq!(
            serde_json::to_value(Outcome::from(unregistered)).unwrap(),
            serde_json::json!({
                "reason": "yubikey_unregistered",
                "serial": "01c95c1f",
            })
        );
    }

    #[test]
    fn rendering_states_the_local_only_effect() {
        let rendered = YubiKeyUnregistered {
            serial: YubiKeySerial::from(0x01c9_5c1f),
        }
        .to_string();

        assert!(rendered.contains("removed from the local TVC configuration"));
        assert!(rendered.contains("device was not modified"));
        assert!(rendered.contains("no organization operator was revoked"));
    }
}
