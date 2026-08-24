//! YubiKey registry refresh command - re-reads a provisioned device's
//! operator key and updates the cached registry entry.

use crate::{
    commands::Run,
    config::turnkey::{
        Config, QosOperatorPublicKey, Registration, YubiKeySerial, config_file_path,
    },
    outcome::Outcome,
    output::StdCtx,
    prompts,
    yubikey::{self, DeviceOps},
};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::fmt::{self, Display, Formatter};

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
        let connected = ctx.connected_yubikeys()?;

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
                _ if ctx.is_non_interactive() || !prompts::stdin_can_prompt() => {
                    return Err(prompts::error_required_in_non_interactive("--serial"));
                }
                _ => prompts::select("YubiKey to refresh", connected)?,
            },
        };

        // Reading the slot certificates needs neither the PIN nor a touch,
        // so the refresh itself also runs non-interactively.
        let mut yubikey = yubikey::open(serial)?;
        let operator_public_key = yubikey.verified_pair_public_key()?;
        let registration = config.yubikeys.register(serial, operator_public_key);

        if registration != Registration::Unchanged {
            let config_path = config_file_path()?;
            config.save().await.with_context(|| {
                format!(
                    r#"the key was read but saving the config failed; register it manually by adding this to {}:

[[yubikeys]]
serial = "{serial}"
public_key = "{operator_public_key}""#,
                    config_path.display(),
                )
            })?;
        }

        Ok(YubiKeyRefreshed {
            serial,
            operator_public_key,
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
