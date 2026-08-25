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

/// The registered YubiKey devices, at most one entry per serial.
///
/// Construction is the proof of uniqueness: deserialization funnels through
/// `TryFrom`, and the mutating methods preserve the invariant.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(try_from = "Vec<YubiKeyRegistryEntry>")]
pub struct YubiKeyRegistry(Vec<YubiKeyRegistryEntry>);

/// Error returned when parsing a [`YubiKeyRegistry`].
#[derive(Debug, displaydoc::Display, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum YubiKeyRegistryParseError {
    /// duplicate YubiKey registry entry for serial {0}
    DuplicateSerial(YubiKeySerial),
}

impl TryFrom<Vec<YubiKeyRegistryEntry>> for YubiKeyRegistry {
    type Error = YubiKeyRegistryParseError;

    fn try_from(entries: Vec<YubiKeyRegistryEntry>) -> Result<Self, Self::Error> {
        let duplicate = entries.iter().enumerate().find(|(index, entry)| {
            entries[..*index]
                .iter()
                .any(|earlier| earlier.serial == entry.serial)
        });

        match duplicate {
            Some((_, entry)) => Err(YubiKeyRegistryParseError::DuplicateSerial(entry.serial)),
            None => Ok(Self(entries)),
        }
    }
}

/// How [`YubiKeyRegistry::register`] changed the registry.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Registration {
    /// The serial was newly registered.
    Added,
    /// The serial was registered with a stale public key; the cache now
    /// matches the device.
    Updated,
    /// The registry already matched the device.
    #[default]
    Unchanged,
}

impl YubiKeyRegistry {
    /// Record a device and its operator public key.
    ///
    /// The device is authoritative: when the serial is already registered
    /// with a different key (reprovisioned outside tvc, or a hand-edited
    /// config), the cached key is replaced.
    pub fn register(
        &mut self,
        serial: YubiKeySerial,
        public_key: QosOperatorPublicKey,
    ) -> Registration {
        let Some(entry) = self.0.iter_mut().find(|entry| entry.serial == serial) else {
            self.0.push(YubiKeyRegistryEntry {
                serial,
                public_key,
                extra: toml::Table::new(),
            });
            return Registration::Added;
        };

        if entry.public_key == public_key {
            Registration::Unchanged
        } else {
            entry.public_key = public_key;
            Registration::Updated
        }
    }

    /// Remove a serial. Returns `false` when it was not registered.
    pub fn deregister(&mut self, serial: YubiKeySerial) -> bool {
        let count_before = self.0.len();
        self.0.retain(|entry| entry.serial != serial);
        self.0.len() < count_before
    }

    pub fn get(&self, serial: YubiKeySerial) -> Option<&YubiKeyRegistryEntry> {
        self.0.iter().find(|entry| entry.serial == serial)
    }

    pub fn contains(&self, serial: YubiKeySerial) -> bool {
        self.get(serial).is_some()
    }

    /// Registered serials, in config order.
    pub fn serials(&self) -> impl Iterator<Item = YubiKeySerial> + '_ {
        self.0.iter().map(|entry| entry.serial)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
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

    fn serial() -> YubiKeySerial {
        YubiKeySerial::from(0x01c9_5c1f)
    }

    fn pair_key() -> QosOperatorPublicKey {
        QosOperatorPublicKey::try_from([7u8; 130].as_slice()).unwrap()
    }

    #[test]
    fn registration_is_idempotent() {
        let mut registry = YubiKeyRegistry::default();

        assert_eq!(registry.register(serial(), pair_key()), Registration::Added);
        assert_eq!(
            registry.register(serial(), pair_key()),
            Registration::Unchanged
        );
        assert_eq!(registry.serials().collect::<Vec<_>>(), vec![serial()]);
    }

    #[test]
    fn registration_refreshes_a_stale_cached_key() {
        let mut registry = YubiKeyRegistry::default();
        let stale = QosOperatorPublicKey::try_from([9u8; 130].as_slice()).unwrap();
        registry.register(serial(), stale);

        assert_eq!(
            registry.register(serial(), pair_key()),
            Registration::Updated
        );
        assert_eq!(registry.0[0].public_key, pair_key());
    }

    #[test]
    fn deregistration_removes_the_entry() {
        let mut registry = YubiKeyRegistry::default();
        registry.register(serial(), pair_key());

        assert!(registry.deregister(serial()));
        assert!(!registry.deregister(serial()));
        assert!(registry.is_empty());
        assert!(!registry.contains(serial()));
    }

    #[test]
    fn deserialization_rejects_duplicate_serials() {
        let entry = YubiKeyRegistryEntry {
            serial: serial(),
            public_key: pair_key(),
            extra: toml::Table::new(),
        };

        let error = YubiKeyRegistry::try_from(vec![entry.clone(), entry]).unwrap_err();

        assert_eq!(error, YubiKeyRegistryParseError::DuplicateSerial(serial()));
    }
}
