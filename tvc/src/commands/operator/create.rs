//! Create a Turnkey-hosted TVC operator.

use crate::{
    client::build_client,
    config::turnkey::{Config, OperatorRecord, OperatorRecordKind},
    operator::{
        DEFAULT_HOSTED_OPERATOR_BASE_PATH, ensure_authenticated_org,
        hosted::CreateOperatorRequestResult, hosted_activity_error, timestamp_ms,
    },
    outcome::Outcome,
    output::StdCtx,
};
use anyhow::{Context, Result, anyhow};
use clap::{ArgGroup, Args as ClapArgs, builder::NonEmptyStringValueParser};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use tracing::instrument;
use turnkey_client::generated::CreateTvcOperatorIntent;
use uuid::Uuid;

const DEFAULT_OPERATOR_NAME: &str = "tvc-operator";
const DEFAULT_WALLET_NAME: &str = "tvc-wallet";

/// Create one hosted TVC operator and save it to the active organization.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
#[command(group(
    ArgGroup::new("wallet")
        .args(["wallet_name", "wallet_id"])
        .multiple(false)
))]
pub struct Args {
    /// Human-readable operator name.
    #[arg(
        long,
        env = "TVC_OPERATOR_NAME",
        default_value = DEFAULT_OPERATOR_NAME,
        value_parser = NonEmptyStringValueParser::new()
    )]
    name: String,

    /// Name for a newly created wallet. Defaults to tvc-wallet.
    #[arg(
        long,
        env = "TVC_OPERATOR_WALLET_NAME",
        default_value = DEFAULT_WALLET_NAME,
        value_parser = NonEmptyStringValueParser::new()
    )]
    wallet_name: String,

    /// Existing wallet UUID in which to create the operator accounts.
    #[arg(long, env = "TVC_OPERATOR_WALLET_ID")]
    wallet_id: Option<Uuid>,

    /// Base derivation path. Defaults to m/5527107'/0'/0'. The server appends
    /// the encrypt/sign role paths.
    #[arg(
        long,
        env = "TVC_OPERATOR_ACCOUNT_PATH",
        default_value = DEFAULT_HOSTED_OPERATOR_BASE_PATH,
        value_parser = NonEmptyStringValueParser::new()
    )]
    account_path: String,
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default))]
#[serde(rename_all = "camelCase")]
pub struct OperatorCreated {
    name: String,
    operator_id: Uuid,
    wallet_id: Uuid,
    encrypt_public_key: String,
    sign_public_key: String,
    composite_public_key: String,
    saved: bool,
}

impl Display for OperatorCreated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            r#"Hosted operator created!
Operator name: {}
Operator ID: {}
Wallet ID: {}
Encryption public key: {}
Signing public key: {}
Composite public key: {}
Saved: true"#,
            self.name,
            self.operator_id,
            self.wallet_id,
            self.encrypt_public_key,
            self.sign_public_key,
            self.composite_public_key
        )
    }
}

#[instrument(skip_all)]
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let mut config = Config::load().await?;
    let (alias, configured_org_id) = config
        .active_org_config()
        .map(|(alias, org)| (alias.clone(), org.id.as_str()))
        .context("No active organization. Run `tvc login` first.")?;

    let auth = build_client().await?;
    ensure_authenticated_org(&auth.org_id, configured_org_id)?;

    let HostedOperatorSpec { name, wallet, path } = args.into();

    let (wallet_name, wallet_id) = match wallet {
        HostedOperatorWallet::New(name) => (Some(name), None),
        HostedOperatorWallet::Existing(id) => (None, Some(id.to_string())),
    };

    let intent = CreateTvcOperatorIntent {
        wallet_name,
        wallet_id,
        path: path.clone(),
        operator_name: name.clone(),
    };

    let result = auth
        .client
        .create_tvc_operator(auth.org_id.clone(), timestamp_ms()?, intent)
        .await
        .map_err(|error| hosted_activity_error("create hosted TVC operator", error))?;

    let result = CreateOperatorRequestResult {
        name,
        path,
        result: result.result,
    };

    let record = OperatorRecord::try_from(result)?;

    config
        .orgs
        .get_mut(&alias)
        .with_context(|| format!("active organization '{alias}' disappeared from config"))?
        .operators
        .push(record.clone());

    let OperatorRecord { name, kind } = record;
    let OperatorRecordKind::Hosted(hosted) = kind else {
        return Err(anyhow!("hosted operator creation returned a local record"));
    };

    if let Err(save_error) = config.save().await {
        let record = config
            .orgs
            .get(&alias)
            .and_then(|org| org.operators.last())
            .with_context(|| {
                format!("hosted operator disappeared from active organization '{alias}'")
            })?;
        let recovery = recovery_toml(&alias, record)?;
        return Err(anyhow!(
            r#"hosted operator {} was created remotely, but saving the local config failed: {save_error}
Do not retry creation blindly; doing so would create another remote operator. Restore this record under the active organization in tvc.config.toml:

{recovery}"#,
            hosted.operator_id
        ));
    }

    let output = OperatorCreated {
        name,
        composite_public_key: format!("{}{}", hosted.encrypt_public_key, hosted.sign_public_key),
        operator_id: hosted.operator_id,
        wallet_id: hosted.wallet_id,
        encrypt_public_key: hosted.encrypt_public_key,
        sign_public_key: hosted.sign_public_key,
        saved: true,
    };

    Ok(Outcome::OperatorCreated(output))
}

/// Inputs for creating one hosted TVC operator.
#[derive(Debug, PartialEq, Eq)]
struct HostedOperatorSpec {
    name: String,
    wallet: HostedOperatorWallet,
    path: String,
}

/// Valid wallet selections for hosted operator creation.
#[derive(Debug, PartialEq, Eq)]
enum HostedOperatorWallet {
    /// Create a new wallet with this name to hold the operator accounts.
    New(String),
    /// Add the operator accounts to the existing wallet with this ID.
    Existing(Uuid),
}

impl From<Args> for HostedOperatorSpec {
    fn from(args: Args) -> Self {
        let wallet = match args.wallet_id {
            Some(id) => HostedOperatorWallet::Existing(id),
            None => HostedOperatorWallet::New(args.wallet_name),
        };

        HostedOperatorSpec {
            name: args.name,
            wallet,
            path: args.account_path,
        }
    }
}

fn recovery_toml(alias: &str, record: &OperatorRecord) -> Result<String> {
    let quoted_alias = serde_json::to_string(alias)
        .context("failed to quote organization alias for recovery record")?;
    let record = toml::to_string_pretty(record)
        .context("failed to serialize hosted operator recovery record")?;
    Ok(format!(
        r#"[[orgs.{quoted_alias}.operators]]
{}"#,
        record.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::HostedOperatorRecord;

    fn hosted_record() -> OperatorRecord {
        OperatorRecord {
            name: "tvc-operator".to_string(),
            kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                operator_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
                wallet_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
                path: "m/5527107'/0'/0'".to_string(),
                encrypt_public_key: format!("04{}", "11".repeat(64)),
                sign_public_key: format!("04{}", "22".repeat(64)),
                extra: toml::Table::new(),
            }),
        }
    }

    #[test]
    fn recovery_toml_contains_complete_hosted_record() {
        #[derive(serde::Deserialize)]
        struct Recovery {
            orgs: std::collections::HashMap<String, RecoveryOrg>,
        }
        #[derive(serde::Deserialize)]
        struct RecoveryOrg {
            operators: Vec<OperatorRecord>,
        }

        let expected = hosted_record();
        let recovery: Recovery =
            toml::from_str(&recovery_toml("default", &expected).unwrap()).unwrap();

        assert_eq!(recovery.orgs["default"].operators, vec![expected]);
    }
}
