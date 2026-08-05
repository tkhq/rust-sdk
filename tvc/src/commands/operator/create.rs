//! Create a Turnkey-hosted TVC operator.

use crate::{
    client::build_client,
    commands::Run,
    config::turnkey::{Config, OperatorRecord, OperatorRecordKind},
    operator::{
        DEFAULT_HOSTED_OPERATOR_BASE_PATH, HostedOperatorSpec, HostedOperatorWallet,
        create_hosted_operator, ensure_authenticated_org,
    },
    outcome::Outcome,
    output::StdCtx,
    prompts,
};
use anyhow::{Context, Result, anyhow, bail};
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
    wallet_id: Uuid,
    encrypt_public_key: String,
    sign_public_key: String,
    composite_public_key: String,
    saved: bool,
}

impl From<OperatorCreated> for Outcome {
    fn from(outcome: OperatorCreated) -> Self {
        Outcome::OperatorCreated(outcome)
    }
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

impl Run for Args {
    type Outcome = OperatorCreated;

    async fn run(self, ctx: &mut StdCtx) -> Result<OperatorCreated> {
        let mut config = Config::load().await?;
        let (alias, org) = config
            .active_org_config()
            .context("No active organization. Run `tvc login` first.")?;
        // Review answer (delete me): the clone is needed because `alias` names
        // the org-map slot we re-borrow mutably at `config.orgs.get_mut(&alias)`
        // after the network call; keeping the `&String` borrow of `config` alive
        // that long conflicts with the `&mut`. It's inherent to the data flow
        // (we keep the key), not induced by a helper boundary.
        let alias = alias.clone();

        // Only an existing wallet can collide: --wallet-name mints a fresh
        // wallet (fresh seed), so its derived keys cannot match any saved
        // record. Hosted operator keys are derived server-side from the wallet
        // and base derivation path, so an existing record with the same wallet
        // + path (textual comparison) implies identical keys under a different
        // operator ID.
        if let Some(wallet_id) = self.wallet_id {
            let existing: Vec<String> = org
                .operators
                .iter()
                .filter_map(|record| match &record.kind {
                    OperatorRecordKind::Hosted(hosted)
                        if hosted.wallet_id == wallet_id
                            && hosted.path.trim() == self.account_path.trim() =>
                    {
                        Some(format!("'{}' ({})", record.name, hosted.operator_id))
                    }
                    OperatorRecordKind::Hosted(_) | OperatorRecordKind::Local(_) => None,
                })
                .collect();

            if !existing.is_empty() {
                let existing = existing.join(", ");

                if ctx.is_non_interactive() {
                    bail!(
                        r#"wallet {wallet_id} at path {path} already backs hosted operator(s) {existing}.
Hosted operator keys are derived from the wallet and account path, so the new operator's keys would be identical under a different operator ID.
Use a different --account-path or --wallet-id, or rerun interactively to confirm creating an operator with duplicate keys."#,
                        path = self.account_path
                    );
                }

                prompts::confirm_or_bail(
                    &format!(
                        "Operator(s) {existing} are already backed by wallet {wallet_id} at path {}; the new operator's keys will be identical. Create anyway?",
                        self.account_path
                    ),
                    "hosted operator creation",
                )?;
            }
        }

        let auth = build_client().await?;
        ensure_authenticated_org(&auth.org_id, &org.id)?;

        let record = create_hosted_operator(&auth, hosted_operator_spec(self)).await?;
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

        Ok(output)
    }
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
        wallet_id: hosted.wallet_id,
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
        let value = serde_json::to_value(Outcome::OperatorCreated(output)).unwrap();

        assert_eq!(
            value,
            serde_json::json!({
                "reason": "operator_created",
                "name": "tvc-operator",
                "operatorId": "11111111-1111-4111-8111-111111111111",
                "walletId": "22222222-2222-4222-8222-222222222222",
                "encryptPublicKey": format!("04{}", "11".repeat(64)),
                "signPublicKey": format!("04{}", "22".repeat(64)),
                "compositePublicKey": format!("04{}04{}", "11".repeat(64), "22".repeat(64)),
                "saved": true,
            })
        );
    }
}
