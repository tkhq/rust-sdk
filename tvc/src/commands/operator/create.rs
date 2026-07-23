//! Create a Turnkey-hosted TVC operator.

use crate::{
    client::build_client,
    config::turnkey::{Config, OperatorRecord, OperatorRecordKind},
    operator::{
        DEFAULT_HOSTED_OPERATOR_BASE_PATH, HostedOperatorSpec, HostedOperatorWallet,
        create_hosted_operator, ensure_authenticated_org,
    },
    outcome::Outcome,
    output::StdCtx,
};
use anyhow::{Context, Result, anyhow};
use clap::{ArgGroup, Args as ClapArgs, builder::NonEmptyStringValueParser};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
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
Encryption public key: {}
Signing public key: {}
Composite public key: {}
Saved: true"#,
            self.name,
            self.operator_id,
            self.encrypt_public_key,
            self.sign_public_key,
            self.composite_public_key
        )
    }
}

pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let mut config = Config::load().await?;
    let (alias, configured_org_id) = config
        .active_org_config()
        .map(|(alias, org)| (alias.clone(), org.id.as_str()))
        .context("No active organization. Run `tvc login` first.")?;

    let auth = build_client().await?;
    ensure_authenticated_org(&auth.org_id, configured_org_id)?;

    let record = create_hosted_operator(&auth, hosted_operator_spec(args)).await?;
    let output = output_from_record(record.clone())?;

    config
        .orgs
        .get_mut(&alias)
        .with_context(|| format!("active organization '{alias}' disappeared from config"))?
        .operators
        .push(record);

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
            output.operator_id
        ));
    }

    Ok(Outcome::OperatorCreate(output))
}

fn hosted_operator_spec(args: Args) -> HostedOperatorSpec {
    let wallet = match args.wallet_id {
        Some(id) => HostedOperatorWallet::Existing(id),
        None => HostedOperatorWallet::New(args.wallet_name),
    };

    HostedOperatorSpec::new(args.name, wallet, args.account_path)
}

fn output_from_record(record: OperatorRecord) -> Result<OperatorCreated> {
    let OperatorRecord { name, kind } = record;
    let OperatorRecordKind::Hosted(hosted) = kind else {
        return Err(anyhow!("hosted operator creation returned a local record"));
    };
    let composite_public_key = format!("{}{}", hosted.encrypt_public_key, hosted.sign_public_key);

    Ok(OperatorCreated {
        name,
        operator_id: hosted.operator_id,
        encrypt_public_key: hosted.encrypt_public_key,
        sign_public_key: hosted.sign_public_key,
        composite_public_key,
        saved: true,
    })
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
    use crate::output::Message;

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

    #[test]
    fn operator_created_serializes_expected_json() {
        let output = output_from_record(hosted_record()).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&Outcome::OperatorCreate(output).to_json_string()).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "operator-created",
                "name": "tvc-operator",
                "operatorId": "11111111-1111-4111-8111-111111111111",
                "encryptPublicKey": format!("04{}", "11".repeat(64)),
                "signPublicKey": format!("04{}", "22".repeat(64)),
                "compositePublicKey": format!("04{}04{}", "11".repeat(64), "22".repeat(64)),
                "saved": true,
            })
        );
    }
}
