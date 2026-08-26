//! Signer quorum key selection for secret import/export flows.

use crate::config::turnkey::{API_BASE_URL_PREPROD, API_BASE_URL_PROD};
use anyhow::{Context, Result, bail};
use turnkey_enclave_encrypt::QuorumPublicKey;

/// Resolve the signer quorum public key that authenticates every secret
/// import/export exchange with Turnkey's signer enclave.
///
/// An explicit override wins; otherwise the key is inferred from the active
/// org's API base URL. Only production and preprod signer keys ship with the
/// CLI, so other environments must pass `--signer-quorum-key`.
pub fn signer_quorum_key(
    api_base_url: &str,
    override_hex: Option<&str>,
) -> Result<QuorumPublicKey> {
    if let Some(hex) = override_hex {
        return QuorumPublicKey::from_string(hex).context(
            "invalid --signer-quorum-key: expected 130 hex-encoded bytes \
             (two concatenated SEC1 uncompressed P-256 public keys)",
        );
    }
    match api_base_url {
        API_BASE_URL_PROD => Ok(QuorumPublicKey::production_signer()),
        API_BASE_URL_PREPROD => Ok(QuorumPublicKey::preprod_signer()),
        other => bail!(
            "no built-in signer quorum public key for API base URL '{other}'; \
             pass --signer-quorum-key"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_url_resolves_to_production_signer() {
        let key = signer_quorum_key(API_BASE_URL_PROD, None).unwrap();
        assert_eq!(key, QuorumPublicKey::production_signer());
    }

    #[test]
    fn preprod_url_resolves_to_preprod_signer() {
        let key = signer_quorum_key(API_BASE_URL_PREPROD, None).unwrap();
        assert_eq!(key, QuorumPublicKey::preprod_signer());
    }

    #[test]
    fn override_beats_url_inference() {
        let hex = "ab".repeat(130);
        let key = signer_quorum_key(API_BASE_URL_PROD, Some(&hex)).unwrap();
        assert_eq!(key, QuorumPublicKey::from_bytes([0xab; 130]).unwrap());
    }

    #[test]
    fn invalid_override_is_rejected() {
        let error = signer_quorum_key(API_BASE_URL_PROD, Some("zz")).unwrap_err();
        assert!(error.to_string().contains("--signer-quorum-key"));
    }

    #[test]
    fn unknown_url_without_override_is_rejected() {
        let error = signer_quorum_key("https://api.dev.turnkey.engineering", None).unwrap_err();
        assert!(error.to_string().contains("pass --signer-quorum-key"));
    }
}
