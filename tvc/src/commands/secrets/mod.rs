//! Secret storage commands.

pub mod delete;
pub mod export;
pub mod import;
pub mod list;

use crate::client::AuthenticatedClient;
use anyhow::{Context, Result};
use turnkey_client::generated::external::options::v1::Pagination;
use turnkey_client::generated::{ListSecretsRequest, ListSecretsResponse, SecretMetadata};

/// Page size for metadata listings.
const LIST_PAGE_LIMIT: usize = 100;

/// Fetch all secret metadata for the active organization, following
/// pagination until a short page.
async fn list_all_secrets(auth: &AuthenticatedClient) -> Result<Vec<SecretMetadata>> {
    let mut all = Vec::new();
    let mut after = String::new();
    loop {
        let ListSecretsResponse { secrets } = auth
            .client
            .list_secrets(ListSecretsRequest {
                organization_id: auth.org_id.clone(),
                pagination_options: Some(Pagination {
                    limit: LIST_PAGE_LIMIT.to_string(),
                    before: String::new(),
                    after,
                }),
            })
            .await
            .context("failed to list secrets")?;

        let page_len = secrets.len();
        after = secrets
            .last()
            .map(|secret| secret.secret_id.clone())
            .unwrap_or_default();
        all.extend(secrets);
        if page_len < LIST_PAGE_LIMIT {
            return Ok(all);
        }
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

    pub fn test_json_ctx() -> StdCtx {
        Ctx::new(
            Shell::standard(MessageFormat::Json, ColorChoice::Never),
            true,
        )
    }
}
