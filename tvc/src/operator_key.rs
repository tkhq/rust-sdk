//! Resolve an operator's QOS key pair from CLI args or the active org config.

use crate::config::turnkey::{Config, StoredQosOperatorKey};
use crate::pair::LocalPair;
use anyhow::anyhow;
use std::path::Path;

/// Load the operator's QOS key pair, preferring an explicit seed file path
/// and falling back to the operator key stored under the active org config.
pub async fn load_operator_pair(operator_seed: Option<&Path>) -> anyhow::Result<LocalPair> {
    match operator_seed {
        Some(path) => LocalPair::from_master_seed(path).await,
        None => {
            let tvc_config = Config::load().await?;
            let (alias, org_config) = tvc_config.active_org_config().ok_or_else(|| {
                anyhow!("No active organization. Run `tvc login` first or provide --operator-seed.")
            })?;

            let operator_key = StoredQosOperatorKey::load(org_config)
                .await?
                .ok_or_else(|| {
                    anyhow!(
                        "No operator key found for org '{alias}'. \
                         Run `tvc login` first or provide --operator-seed."
                    )
                })?;

            LocalPair::from_hex_seed(&operator_key.private_key)
        }
    }
}
