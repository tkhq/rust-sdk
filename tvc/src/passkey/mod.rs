//! Passkey and WebAuthn support for TVC authentication.

use anyhow::{Result, bail};
use base64::Engine as _;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use clap::ValueEnum;
use qos_crypto::sha_256;
use serde::{Deserialize, Serialize};
use turnkey_api_key_stamper::{Stamp, StampHeader, StamperError};
use turnkey_client::generated::external::webauthn::v1::WebAuthnStamp;

/// Header accepted by Turnkey public API requests for WebAuthn assertions.
pub const WEBAUTHN_STAMP_HEADER_NAME: &str = "X-Stamp-WebAuthn";

/// WebAuthn transport requested by the user.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum PasskeyTransport {
    /// Prefer a native/browser-capable ceremony when available; fall back to USB.
    Auto,
    /// Roaming USB security keys such as YubiKeys.
    Usb,
    /// Browser or hybrid QR flow for platform passkeys and password managers.
    Browser,
    /// Cross-device WebAuthn flow, usually shown as a QR code by a browser.
    CrossDevice,
}

impl std::fmt::Display for PasskeyTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Usb => write!(f, "usb"),
            Self::Browser => write!(f, "browser"),
            Self::CrossDevice => write!(f, "cross-device"),
        }
    }
}

/// WebAuthn assertion fields sent to Turnkey as a stamp.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnAssertion {
    pub credential_id: String,
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
}

impl From<WebAuthnAssertion> for WebAuthnStamp {
    fn from(assertion: WebAuthnAssertion) -> Self {
        Self {
            credential_id: assertion.credential_id,
            client_data_json: assertion.client_data_json,
            authenticator_data: assertion.authenticator_data,
            signature: assertion.signature,
        }
    }
}

/// WebAuthn attestation fields used when registering an authenticator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnAttestation {
    pub credential_id: String,
    pub client_data_json: String,
    pub attestation_object: String,
    pub transports: Vec<String>,
}

/// A synchronous WebAuthn ceremony provider.
pub trait PasskeyCeremony: Send + Sync {
    fn assert(&self, challenge: &str) -> Result<WebAuthnAssertion>;

    fn attest(&self, _challenge: &str, _name: &str) -> Result<WebAuthnAttestation> {
        bail!("passkey registration is not implemented for this transport")
    }
}

/// Derive the Turnkey WebAuthn challenge from the exact serialized request body.
///
/// Backend verification expects `base64url(hex(sha256(raw_post_body)))`.
pub fn derive_challenge(body: &[u8]) -> String {
    let digest = sha_256(body);
    BASE64_URL_SAFE_NO_PAD.encode(hex::encode(digest))
}

/// Stamper that turns a WebAuthn assertion ceremony into `X-Stamp-WebAuthn`.
pub struct WebAuthnStamper<C> {
    ceremony: C,
}

impl<C> WebAuthnStamper<C> {
    pub fn new(ceremony: C) -> Self {
        Self { ceremony }
    }
}

impl WebAuthnStamper<FixtureCeremony> {
    /// Construct a deterministic stamper for CI tests without hardware.
    pub fn new_for_tests(assertion: WebAuthnAssertion) -> Self {
        Self::new(FixtureCeremony { assertion })
    }
}

impl<C: PasskeyCeremony> Stamp for WebAuthnStamper<C> {
    fn stamp(&self, body: &[u8]) -> std::result::Result<StampHeader, StamperError> {
        let challenge = derive_challenge(body);
        let assertion = self
            .ceremony
            .assert(&challenge)
            .map_err(|e| StamperError::WebAuthn(e.to_string()))?;
        let stamp: WebAuthnStamp = assertion.into();
        let value =
            serde_json::to_string(&stamp).map_err(|e| StamperError::WebAuthn(e.to_string()))?;

        Ok(StampHeader {
            name: WEBAUTHN_STAMP_HEADER_NAME.to_string(),
            value,
        })
    }
}

/// CI-only deterministic ceremony used by tests and local mock flows.
pub struct FixtureCeremony {
    assertion: WebAuthnAssertion,
}

impl PasskeyCeremony for FixtureCeremony {
    fn assert(&self, _challenge: &str) -> Result<WebAuthnAssertion> {
        Ok(self.assertion.clone())
    }
}

/// Placeholder for real user-presence transports until native/browser ceremony code lands.
pub struct UnsupportedCeremony {
    transport: PasskeyTransport,
}

impl UnsupportedCeremony {
    pub fn new(transport: PasskeyTransport) -> Self {
        Self { transport }
    }
}

impl PasskeyCeremony for UnsupportedCeremony {
    fn assert(&self, _challenge: &str) -> Result<WebAuthnAssertion> {
        match self.transport {
            PasskeyTransport::Usb => bail!(
                "USB passkey support requires a native CTAP2 provider; no YubiKey provider is enabled in this build"
            ),
            PasskeyTransport::Browser | PasskeyTransport::CrossDevice => bail!(
                "browser passkey handoff is not enabled in this build; use a browser or hybrid WebAuthn flow once available"
            ),
            PasskeyTransport::Auto => bail!(
                "no passkey transport is available in this build; try --passkey-transport=usb or --passkey-transport=browser after enabling native support"
            ),
        }
    }
}
