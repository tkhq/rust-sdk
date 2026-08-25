//! Secret storage commands.

pub mod export;
pub mod import;

use crate::config::turnkey::{API_BASE_URL_PREPROD, API_BASE_URL_PROD};
use anyhow::{Context, Result, bail};
use turnkey_enclave_encrypt::QuorumPublicKey;

/// Resolve the signer quorum public key that authenticates every secret
/// import/export exchange with Turnkey's signer enclave.
///
/// An explicit override wins; otherwise the key is inferred from the active
/// org's API base URL. Only production and preprod signer keys ship with the
/// CLI, so other environments must pass `--signer-public-key`.
fn signer_quorum_public_key(
    api_base_url: &str,
    override_hex: Option<&str>,
) -> Result<QuorumPublicKey> {
    if let Some(hex) = override_hex {
        return QuorumPublicKey::from_string(hex).context(
            "invalid --signer-public-key: expected 130 hex-encoded bytes \
             (two concatenated SEC1 uncompressed P-256 public keys)",
        );
    }
    match api_base_url {
        API_BASE_URL_PROD => Ok(QuorumPublicKey::production_signer()),
        API_BASE_URL_PREPROD => Ok(QuorumPublicKey::preprod_signer()),
        other => bail!(
            "no built-in signer quorum public key for API base URL '{other}'; \
             pass --signer-public-key"
        ),
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use crate::config::turnkey::{
        Config, KeyCurve, OperatorKind, OperatorRecord, OrgConfig, StoredApiKey,
    };
    use crate::output::{ColorChoice, Ctx, MessageFormat, Shell, StdCtx};
    use p256::ecdsa::SigningKey;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use turnkey_api_key_stamper::TurnkeyP256ApiKey;

    /// A fixed signing key standing in for the signer enclave's quorum key.
    pub fn test_signing_key() -> SigningKey {
        SigningKey::from_slice(&[7u8; 32]).unwrap()
    }

    /// The hex quorum public key whose signing half matches `signing`. The
    /// encryption half is unused by the secrets flow.
    pub fn quorum_public_key_hex(signing: &SigningKey) -> String {
        let verifying = signing
            .verifying_key()
            .to_encoded_point(false)
            .as_bytes()
            .to_vec();
        hex::encode([verifying.clone(), verifying].concat())
    }

    /// A config whose single active org points at `api_base_url` with a
    /// freshly generated API key on disk.
    pub fn test_config(dir: &TempDir, api_base_url: &str) -> Config {
        let api_key_path = dir.path().join("api_key.json");
        let stamper = TurnkeyP256ApiKey::generate();
        std::fs::write(
            &api_key_path,
            serde_json::to_string(&StoredApiKey {
                public_key: hex::encode(stamper.compressed_public_key()),
                private_key: hex::encode(stamper.private_key()),
                curve: KeyCurve::P256,
            })
            .unwrap(),
        )
        .unwrap();

        Config {
            active_org: Some("test".to_string()),
            orgs: HashMap::from([(
                "test".to_string(),
                OrgConfig {
                    id: "org-1".to_string(),
                    api_key_path,
                    api_base_url: api_base_url.to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators: vec![OperatorRecord::local(dir.path().join("operator.json"))],
                    extra: toml::Table::new(),
                },
            )]),
            last_created_app_id: HashMap::new(),
            last_operator_ids: HashMap::new(),
            extra: toml::Table::new(),
        }
    }

    pub fn test_ctx() -> StdCtx {
        Ctx::new(
            Shell::standard(MessageFormat::Human, ColorChoice::Never),
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_url_resolves_to_production_signer() {
        let key = signer_quorum_public_key(API_BASE_URL_PROD, None).unwrap();
        assert_eq!(key, QuorumPublicKey::production_signer());
    }

    #[test]
    fn preprod_url_resolves_to_preprod_signer() {
        let key = signer_quorum_public_key(API_BASE_URL_PREPROD, None).unwrap();
        assert_eq!(key, QuorumPublicKey::preprod_signer());
    }

    #[test]
    fn override_beats_url_inference() {
        let hex = "ab".repeat(130);
        let key = signer_quorum_public_key(API_BASE_URL_PROD, Some(&hex)).unwrap();
        assert_eq!(key, QuorumPublicKey::from_bytes([0xab; 130]).unwrap());
    }

    #[test]
    fn invalid_override_is_rejected() {
        let error = signer_quorum_public_key(API_BASE_URL_PROD, Some("zz")).unwrap_err();
        assert!(error.to_string().contains("--signer-public-key"));
    }

    #[test]
    fn unknown_url_without_override_is_rejected() {
        let error =
            signer_quorum_public_key("https://api.dev.turnkey.engineering", None).unwrap_err();
        assert!(error.to_string().contains("pass --signer-public-key"));
    }
}
