//! YubiKey device registry configuration.

use super::QosOperatorPublicKey;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

/// A YubiKey device serial number.
///
/// String parsing accepts bare hexadecimal input with surrounding whitespace.
/// Display always emits the canonical form: lowercase hexadecimal, zero-padded
/// to eight digits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub struct YubiKeySerial(u32);

/// Error returned when parsing a [`YubiKeySerial`].
#[derive(Debug, displaydoc::Display, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum YubiKeySerialParseError {
    /// must be bare hex encoded
    InvalidHex,
    /// must fit in 32 bits
    Overflow,
}

impl YubiKeySerial {
    /// The raw serial number, for handing to device APIs.
    pub fn number(self) -> u32 {
        self.0
    }
}

impl From<u32> for YubiKeySerial {
    fn from(number: u32) -> Self {
        Self(number)
    }
}

impl FromStr for YubiKeySerial {
    type Err = YubiKeySerialParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.trim();

        // `from_str_radix` also accepts a sign prefix, which a serial must
        // not have, so the charset is checked first.
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(YubiKeySerialParseError::InvalidHex);
        }

        u32::from_str_radix(value, 16)
            .map(Self)
            .map_err(|_| YubiKeySerialParseError::Overflow)
    }
}

impl Display for YubiKeySerial {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:08x}", self.0)
    }
}

/// One registered YubiKey device.
///
/// The stored public key is a convenience cache so key-consuming flows work
/// without the device connected; the device itself stays authoritative, and
/// provisioning refreshes the cache whenever the two disagree. Private
/// material never leaves the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct YubiKeyRegistryEntry {
    pub serial: YubiKeySerial,
    /// The composite `encrypt_public ‖ sign_public` operator key.
    pub public_key: QosOperatorPublicKey,
    /// Unrecognized fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

/// Reference to a registered YubiKey used as an organization's operator.
///
/// Holds only the serial of a top-level registry entry; public keys, slots,
/// paths, and secrets stay on the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
pub struct YubiKeyOperatorRecord {
    pub serial: YubiKeySerial,
    /// Unrecognized fields retained across supported config rewrites.
    #[serde(default, flatten)]
    pub extra: toml::Table,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_canonicalizes_hex() {
        let serial: YubiKeySerial = " 1C95C1F \n".parse().unwrap();

        assert_eq!(serial, YubiKeySerial::from(0x01c9_5c1f));
        assert_eq!(serial.to_string(), "01c95c1f");
    }

    #[test]
    fn rejects_invalid_input() {
        assert_eq!(
            "".parse::<YubiKeySerial>(),
            Err(YubiKeySerialParseError::InvalidHex)
        );
        assert_eq!(
            "0x1f".parse::<YubiKeySerial>(),
            Err(YubiKeySerialParseError::InvalidHex)
        );
        assert_eq!(
            "+1f".parse::<YubiKeySerial>(),
            Err(YubiKeySerialParseError::InvalidHex)
        );
        assert_eq!(
            "fffffffff".parse::<YubiKeySerial>(),
            Err(YubiKeySerialParseError::Overflow)
        );
    }

    #[test]
    fn registry_entry_round_trips_with_unknown_fields() {
        let public_key = "07".repeat(130);
        let entry: YubiKeyRegistryEntry = toml::from_str(&format!(
            "serial = \"01c95c1f\"\npublic_key = \"{public_key}\"\nfuture_field = 42"
        ))
        .unwrap();

        assert_eq!(entry.serial, YubiKeySerial::from(0x01c9_5c1f));
        assert_eq!(entry.public_key.to_string(), public_key);

        let reserialized = toml::to_string(&entry).unwrap();
        let round_tripped: YubiKeyRegistryEntry = toml::from_str(&reserialized).unwrap();

        assert_eq!(round_tripped, entry);
    }
}
