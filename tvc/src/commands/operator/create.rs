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

pub async fn run(ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let mut config = Config::load().await?;
    let (alias, org) = config
        .active_org_config()
        .context("No active organization. Run `tvc login` first.")?;
    let alias = alias.clone();

    // Only an existing wallet can collide: --wallet-name mints a fresh wallet
    // (fresh seed), so its derived keys cannot match any saved record.
    if let Some(wallet_id) = args.wallet_id {
        let collisions = find_hosted_key_collisions(&org.operators, wallet_id, &args.account_path);

        if !collisions.is_empty() {
            let existing = collisions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            if ctx.is_non_interactive() {
                bail!(
                    r#"wallet {wallet_id} at path {path} already backs hosted operator(s) {existing}.
Hosted operator keys are derived from the wallet and account path, so the new operator's keys would be identical under a different operator ID.
Use a different --account-path or --wallet-id, or rerun interactively to confirm creating an operator with duplicate keys."#,
                    path = args.account_path
                );
            }

            prompts::confirm_or_bail(
                &format!(
                    "Operator(s) {existing} are already backed by wallet {wallet_id} at path {}; the new operator's keys will be identical. Create anyway?",
                    args.account_path
                ),
                "hosted operator creation",
            )?;
        }
    }

    let auth = build_client().await?;
    ensure_authenticated_org(&auth.org_id, &org.id)?;

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

    Ok(Outcome::OperatorCreated(output))
}

fn hosted_operator_spec(args: Args) -> HostedOperatorSpec {
    let wallet = match args.wallet_id {
        Some(id) => HostedOperatorWallet::Existing(id),
        None => HostedOperatorWallet::New(args.wallet_name),
    };

    HostedOperatorSpec::new(args.name, wallet, args.account_path)
}

/// Identity of an existing hosted operator already backed by a wallet + path.
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct CollidingHostedOperator {
    name: String,
    operator_id: Uuid,
}

impl Display for CollidingHostedOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "'{}' ({})", self.name, self.operator_id)
    }
}

/// Find hosted operators whose keys a new operator would duplicate.
///
/// Hosted operator keys are derived server-side from the wallet and base
/// derivation path, so an existing record with the same wallet ID and account
/// path implies identical keys under a different operator ID. Pure and
/// mode-agnostic; the caller prompts (interactive) or errors (non-interactive)
/// on matches. Path comparison is textual (trimmed), so equivalent derivation
/// paths spelled differently are not caught.
fn find_hosted_key_collisions(
    operators: &[OperatorRecord],
    wallet_id: Uuid,
    account_path: &str,
) -> Vec<CollidingHostedOperator> {
    operators
        .iter()
        .filter_map(|record| match &record.kind {
            OperatorRecordKind::Hosted(hosted)
                if hosted.wallet_id == wallet_id && hosted.path.trim() == account_path.trim() =>
            {
                Some(CollidingHostedOperator {
                    name: record.name.clone(),
                    operator_id: hosted.operator_id,
                })
            }
            OperatorRecordKind::Hosted(_) | OperatorRecordKind::Local(_) => None,
        })
        .collect()
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
    use crate::config::turnkey::{HostedOperatorRecord, LocalOperatorRecord};
    use std::path::PathBuf;

    const FIXTURE_WALLET: &str = "22222222-2222-4222-8222-222222222222";
    const FIXTURE_PATH: &str = "m/5527107'/0'/0'";

    fn fixture_wallet_id() -> Uuid {
        Uuid::parse_str(FIXTURE_WALLET).unwrap()
    }

    fn hosted_record() -> OperatorRecord {
        OperatorRecord {
            name: "tvc-operator".to_string(),
            kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                operator_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
                wallet_id: fixture_wallet_id(),
                path: FIXTURE_PATH.to_string(),
                encrypt_public_key: format!("04{}", "11".repeat(64)),
                sign_public_key: format!("04{}", "22".repeat(64)),
                extra: toml::Table::new(),
            }),
        }
    }

    fn local_record() -> OperatorRecord {
        OperatorRecord {
            name: "default".to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path: PathBuf::from("operator.json"),
                operator_id: None,
                extra: toml::Table::new(),
            }),
        }
    }

    /// Nothing to collide with: no records, or only a local operator (local
    /// keys are not wallet-derived).
    #[test]
    fn no_collisions_without_matching_hosted_records() {
        assert_eq!(
            find_hosted_key_collisions(&[], fixture_wallet_id(), FIXTURE_PATH),
            vec![]
        );
        assert_eq!(
            find_hosted_key_collisions(&[local_record()], fixture_wallet_id(), FIXTURE_PATH),
            vec![]
        );
    }

    /// Same wallet at another path, or another wallet at the same path,
    /// derives different keys -> no collision.
    #[test]
    fn no_collision_when_wallet_or_path_differs() {
        let records = [hosted_record()];

        assert_eq!(
            find_hosted_key_collisions(&records, fixture_wallet_id(), "m/5527107'/0'/1'"),
            vec![]
        );

        let other_wallet = Uuid::parse_str("33333333-3333-4333-8333-333333333333").unwrap();

        assert_eq!(
            find_hosted_key_collisions(&records, other_wallet, FIXTURE_PATH),
            vec![]
        );
    }

    /// Same wallet + path repeats the server-side key derivation -> the
    /// existing operator's identity is surfaced for the prompt/error.
    #[test]
    fn collision_returns_existing_operator_identity() {
        let expected = CollidingHostedOperator {
            name: "tvc-operator".to_string(),
            operator_id: Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap(),
        };

        assert_eq!(
            find_hosted_key_collisions(&[hosted_record()], fixture_wallet_id(), FIXTURE_PATH),
            vec![expected]
        );
    }

    /// Every matching record is reported, in registry order.
    #[test]
    fn collision_returns_every_matching_record() {
        let mut second = hosted_record();
        second.name = "tvc-operator-2".to_string();

        let collisions = find_hosted_key_collisions(
            &[hosted_record(), local_record(), second],
            fixture_wallet_id(),
            FIXTURE_PATH,
        );

        assert_eq!(
            collisions
                .iter()
                .map(|collision| collision.name.as_str())
                .collect::<Vec<_>>(),
            vec!["tvc-operator", "tvc-operator-2"]
        );
    }

    /// Path comparison trims surrounding whitespace (and is otherwise textual).
    #[test]
    fn collision_path_comparison_trims_whitespace() {
        let padded_path = format!("  {FIXTURE_PATH} ");

        assert_eq!(
            find_hosted_key_collisions(&[hosted_record()], fixture_wallet_id(), &padded_path).len(),
            1
        );
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
