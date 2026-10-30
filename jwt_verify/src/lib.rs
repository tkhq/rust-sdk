#![doc = include_str!("../README.md")]

pub mod b64;
mod error;
pub mod jwks;
mod jws;
mod time;

pub use error::Error;
pub use jwks::{JwksError, KeySource, PinnedKeySet, VerificationKey, thumbprint_p256};
pub use jws::{ES256, JwsRules, ProtectedHeader, VerifiedJws, object, verify};
pub use time::{TimeRules, check_times, unix_now};
