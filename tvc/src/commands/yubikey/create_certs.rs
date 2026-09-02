//! Create certificates for existing QuorumOS YubiKey keys without modifying
//! the device.

use crate::{
    config::turnkey::YubiKeySerial,
    outcome::Outcome,
    output::StdCtx,
    prompts,
    yubikey::{self, CertificateDeviceOps, ConnectedYubiKeys, Pin, QosSlot, YubiKeySelectionError},
};
use anyhow::{Context, Result, bail};
use clap::Args as ClapArgs;
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};
use x509_cert::der::{EncodePem, pem::LineEnding};

/// Create importable certificates for keys already generated in slots 9c and
/// 9d.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Serial (hex) of the YubiKey whose certificates to create.
    /// Defaults to the sole connected device.
    #[arg(long, value_name = "SERIAL")]
    serial: Option<YubiKeySerial>,
}

impl Args {
    pub(crate) async fn run(self, ctx: &mut StdCtx) -> Result<YubiKeyCertificatesCreated> {
        if ctx.is_non_interactive() || !prompts::stdin_can_prompt() {
            bail!(
                "creating YubiKey certificates is interactive: the PIN is prompted and the \
                 device must be touched once for each certificate"
            );
        }

        let serial = match ConnectedYubiKeys::from(ctx.connected_yubikeys()?).choose(self.serial) {
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

        let mut yubikey = yubikey::open(serial)?;
        let signing = yubikey.certificate_slot(QosSlot::Signing)?;
        let key_agreement = yubikey.certificate_slot(QosSlot::KeyAgreement)?;
        let pin = Pin::from(prompts::password(
            "YubiKey PIV PIN (touch the device once for each certificate)",
        )?);

        let signing_certificate = signing.create_certificate(&mut yubikey, &pin)?;
        let key_agreement_certificate = key_agreement.create_certificate(&mut yubikey, &pin)?;
        let signing_pem = signing_certificate
            .to_pem(LineEnding::LF)
            .context("failed to encode the signing certificate as PEM")?;
        let key_agreement_pem = key_agreement_certificate
            .to_pem(LineEnding::LF)
            .context("failed to encode the key-agreement certificate as PEM")?;

        let signing_certificate_path = PathBuf::from(format!("tvc-yubikey-{serial}-signing.pem"));
        let key_agreement_certificate_path =
            PathBuf::from(format!("tvc-yubikey-{serial}-key-agreement.pem"));
        tokio::fs::write(&signing_certificate_path, signing_pem)
            .await
            .with_context(|| {
                format!(
                    "failed to write the signing certificate to {}",
                    signing_certificate_path.display()
                )
            })?;
        tokio::fs::write(&key_agreement_certificate_path, key_agreement_pem)
            .await
            .with_context(|| {
                format!(
                    "failed to write the key-agreement certificate to {}",
                    key_agreement_certificate_path.display()
                )
            })?;

        Ok(YubiKeyCertificatesCreated {
            serial,
            signing_certificate_path,
            key_agreement_certificate_path,
        })
    }
}

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubiKeyCertificatesCreated {
    serial: YubiKeySerial,
    signing_certificate_path: PathBuf,
    key_agreement_certificate_path: PathBuf,
}

impl From<YubiKeyCertificatesCreated> for Outcome {
    fn from(created: YubiKeyCertificatesCreated) -> Self {
        Outcome::YubikeyCertificatesCreated(created)
    }
}

impl Display for YubiKeyCertificatesCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"YubiKey certificates created without modifying the device.

Serial:                    {}
Signing certificate:       {}
Key-agreement certificate: {}"#,
            self.serial,
            self.signing_certificate_path.display(),
            self.key_agreement_certificate_path.display()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn created() -> YubiKeyCertificatesCreated {
        YubiKeyCertificatesCreated {
            serial: YubiKeySerial::from(0x01c9_5c1f),
            signing_certificate_path: PathBuf::from("tvc-yubikey-01c95c1f-signing.pem"),
            key_agreement_certificate_path: PathBuf::from("tvc-yubikey-01c95c1f-key-agreement.pem"),
        }
    }

    #[test]
    fn outcome_serializes_both_certificate_paths() {
        assert_eq!(
            serde_json::to_value(Outcome::from(created())).unwrap(),
            serde_json::json!({
                "reason": "yubikey_certificates_created",
                "serial": "01c95c1f",
                "signingCertificatePath": "tvc-yubikey-01c95c1f-signing.pem",
                "keyAgreementCertificatePath": "tvc-yubikey-01c95c1f-key-agreement.pem",
            })
        );
    }

    #[test]
    fn rendering_names_both_certificate_files() {
        assert_eq!(
            created().to_string(),
            r#"YubiKey certificates created without modifying the device.

Serial:                    01c95c1f
Signing certificate:       tvc-yubikey-01c95c1f-signing.pem
Key-agreement certificate: tvc-yubikey-01c95c1f-key-agreement.pem"#
        );
    }
}
