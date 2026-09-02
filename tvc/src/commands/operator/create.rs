//! Create a TVC operator — hosted (the default) or YubiKey — and save it to
//! the active organization.

use crate::{
    client::build_client,
    commands::Run,
    config::turnkey::{
        Config, OperatorKind, OperatorRecord, OperatorRecordKind, QosOperatorPublicKey,
        YubiKeyOperatorRecord, YubiKeySerial,
    },
    operator::{
        DEFAULT_HOSTED_OPERATOR_BASE_PATH, ensure_authenticated_org,
        hosted::CreateOperatorRequestResult, hosted_activity_error, timestamp_ms,
    },
    outcome::Outcome,
    output::StdCtx,
    prompts,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::{ArgGroup, Args as ClapArgs, ValueEnum, builder::NonEmptyStringValueParser};
use serde::Serialize;
use std::fmt::{self, Display, Formatter};
use tracing::instrument;
use turnkey_client::generated::CreateTvcOperatorIntent;
use uuid::Uuid;

const DEFAULT_OPERATOR_NAME: &str = "tvc-operator";
const DEFAULT_WALLET_NAME: &str = "tvc-wallet";

/// The kind of operator to create.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum CreateKind {
    /// Keys held by Turnkey, signing through the API.
    Hosted,
    /// Keys on a YubiKey, referenced by serial.
    Yubikey,
}

/// Create one TVC operator — hosted (the default) or YubiKey — and save it
/// to the active organization.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
#[command(group(
    ArgGroup::new("wallet")
        .args(["wallet_name", "wallet_id"])
        .multiple(false)
))]
pub struct Args {
    /// Operator kind to create.
    #[arg(long, value_enum, default_value = "hosted", env = "TVC_OPERATOR_KIND")]
    kind: CreateKind,

    /// Human-readable operator name [default: tvc-operator (hosted),
    /// yubikey-<serial> (yubikey)].
    #[arg(
        long,
        env = "TVC_OPERATOR_NAME",
        value_parser = NonEmptyStringValueParser::new()
    )]
    name: Option<String>,

    /// Name for a newly created wallet (hosted only) [default: tvc-wallet].
    #[arg(
        long,
        env = "TVC_OPERATOR_WALLET_NAME",
        value_parser = NonEmptyStringValueParser::new()
    )]
    wallet_name: Option<String>,

    /// Existing wallet UUID in which to create the operator accounts (hosted
    /// only).
    #[arg(long, env = "TVC_OPERATOR_WALLET_ID")]
    wallet_id: Option<Uuid>,

    /// Base derivation path (hosted only); the server appends the
    /// encrypt/sign role paths [default: m/5527107'/0'/0'].
    #[arg(
        long,
        env = "TVC_OPERATOR_ACCOUNT_PATH",
        value_parser = NonEmptyStringValueParser::new()
    )]
    account_path: Option<String>,

    /// Serial (hex) of a registered YubiKey to use (yubikey only). If omitted
    /// interactively, selects from the local registry. Never modifies a device.
    #[arg(
        long,
        value_name = "SERIAL",
        conflicts_with_all = ["wallet_name", "wallet_id", "account_path"]
    )]
    serial: Option<YubiKeySerial>,

    /// Make the new operator the organization's default operator kind.
    #[arg(long)]
    default: bool,
}

/// A parsed `operator create` invocation: kind-specific inputs separated,
/// kind-incompatible flags rejected, and hosted defaults applied — all
/// before configuration or credentials are touched.
enum CreatePlan {
    Hosted {
        spec: HostedOperatorSpec,
        make_default: bool,
    },
    Yubikey {
        name: Option<String>,
        serial: Option<YubiKeySerial>,
        make_default: bool,
    },
}

impl TryFrom<Args> for CreatePlan {
    type Error = anyhow::Error;

    fn try_from(args: Args) -> Result<Self> {
        let Args {
            kind,
            name,
            wallet_name,
            wallet_id,
            account_path,
            serial,
            default,
        } = args;

        match kind {
            CreateKind::Hosted => {
                ensure!(
                    serial.is_none(),
                    "--serial is only valid with --kind yubikey"
                );

                let wallet = match wallet_id {
                    Some(id) => HostedOperatorWallet::Existing(id),
                    None => HostedOperatorWallet::New(
                        wallet_name.unwrap_or_else(|| DEFAULT_WALLET_NAME.to_string()),
                    ),
                };

                Ok(Self::Hosted {
                    spec: HostedOperatorSpec {
                        name: name.unwrap_or_else(|| DEFAULT_OPERATOR_NAME.to_string()),
                        wallet,
                        path: account_path
                            .unwrap_or_else(|| DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string()),
                    },
                    make_default: default,
                })
            }
            CreateKind::Yubikey => {
                ensure!(
                    wallet_name.is_none() && wallet_id.is_none() && account_path.is_none(),
                    "--wallet-name, --wallet-id, and --account-path are only valid with --kind hosted"
                );

                Ok(Self::Yubikey {
                    name,
                    serial,
                    make_default: default,
                })
            }
        }
    }
}

/// Terminal shapes of `operator create`, mapped onto the outcome vocabulary.
pub enum CreateOutcome {
    Hosted(OperatorCreated),
    Yubikey(YubikeyOperatorAdded),
}

impl From<CreateOutcome> for Outcome {
    fn from(outcome: CreateOutcome) -> Self {
        match outcome {
            CreateOutcome::Hosted(created) => Outcome::OperatorCreated(created),
            CreateOutcome::Yubikey(added) => Outcome::YubikeyOperatorAdded(added),
        }
    }
}

impl Run for Args {
    type Outcome = CreateOutcome;

    #[instrument(skip_all)]
    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<CreateOutcome> {
        match CreatePlan::try_from(self)? {
            CreatePlan::Hosted { spec, make_default } => {
                let (alias, configured_org_id) = config
                    .active_org_config()
                    .map(|(alias, org)| (alias.clone(), org.id))
                    .context("No active organization. Run `tvc login` first.")?;

                let auth = build_client(&config).await?;
                ensure_authenticated_org(&auth.org_id, &configured_org_id.to_string())?;

                let HostedOperatorSpec { name, wallet, path } = spec;

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

                let org = config.orgs.get_mut(&alias).with_context(|| {
                    format!("active organization '{alias}' disappeared from config")
                })?;
                org.operators.push(record.clone());

                if make_default {
                    org.default_operator_kind = OperatorKind::Hosted;
                }

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
                            format!(
                                "hosted operator disappeared from active organization '{alias}'"
                            )
                        })?;
                    let recovery = recovery_toml(&alias, record)?;
                    let default_note = if make_default {
                        "\n\nAlso set default_operator_kind = \"hosted\" in the organization's table."
                    } else {
                        ""
                    };

                    return Err(anyhow!(
                        r#"hosted operator {} was created remotely, but saving the local config failed: {save_error}
Do not retry creation blindly; doing so would create another remote operator. Restore this record under the active organization in tvc.config.toml:

{recovery}{default_note}"#,
                        hosted.operator_id
                    ));
                }

                Ok(CreateOutcome::Hosted(OperatorCreated {
                    name,
                    composite_public_key: format!(
                        "{}{}",
                        hosted.encrypt_public_key, hosted.sign_public_key
                    ),
                    operator_id: hosted.operator_id,
                    wallet_id: hosted.wallet_id,
                    encrypt_public_key: hosted.encrypt_public_key,
                    sign_public_key: hosted.sign_public_key,
                    saved: true,
                }))
            }
            CreatePlan::Yubikey {
                name,
                serial,
                make_default,
            } => {
                let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

                if serial.is_none() && !can_prompt {
                    return Err(prompts::error_required_in_non_interactive("--serial"));
                }

                let (org_alias, org) = config
                    .active_org_config()
                    .context("No active organization. Run `tvc login` first.")?;
                let org_alias = org_alias.clone();

                let serial = match serial {
                    Some(serial) => {
                        ensure!(
                            config.yubikeys.contains(serial),
                            "YubiKey {serial} is not in the device registry; install its \
                             certificates and run `tvc keys refresh-yubikey --serial {serial}` \
                             first"
                        );
                        serial
                    }
                    None => {
                        let registered = config.yubikeys.serials().collect::<Vec<_>>();
                        match registered.as_slice() {
                            [] => bail!(
                                "no YubiKeys are registered; complete the external setup and run \
                                 `tvc keys refresh-yubikey` first"
                            ),
                            [sole] => *sole,
                            _ => prompts::select("YubiKey to use as the operator", registered)?,
                        }
                    }
                };
                let record = org
                    .new_yubikey_operator(serial, name)
                    .with_context(|| format!("org '{org_alias}'"))?;

                let make_default = make_default
                    || (can_prompt
                        && prompts::confirm(
                            "Make this the default operator for the organization?",
                            false,
                        )?);

                let create = YubikeyCreate {
                    record,
                    serial,
                    make_default,
                    org_alias,
                };
                let added = create.execute(&mut config)?;

                // Nothing is persisted yet; re-running is safe because a
                // duplicate serial is refused before another record is added.
                let record_recovery = recovery_toml(
                    &added.org_alias,
                    &OperatorRecord {
                        name: added.name.clone(),
                        kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                            serial: added.serial,
                            extra: toml::Table::new(),
                        }),
                    },
                )?;
                let default_note = if added.made_default {
                    "\n\nAlso set default_operator_kind = \"yubikey\" in the organization's table."
                } else {
                    ""
                };

                config.save().await.with_context(|| {
                    format!(
                        r#"the YubiKey operator could not be saved; re-running this command is safe, or restore this under the active organization in tvc.config.toml:

{record_recovery}{default_note}"#
                    )
                })?;

                Ok(CreateOutcome::Yubikey(added))
            }
        }
    }
}

/// A resolved YubiKey create using an existing local registry entry.
struct YubikeyCreate {
    record: OperatorRecord,
    serial: YubiKeySerial,
    make_default: bool,
    org_alias: String,
}

impl YubikeyCreate {
    /// Add the serial-only org record and optionally flip the default backend.
    /// Mutates only the in-memory config; [`Run::run`] persists it once.
    fn execute(self, config: &mut Config) -> Result<YubikeyOperatorAdded> {
        let entry = config.yubikeys.get(self.serial).ok_or_else(|| {
            anyhow!(
                "YubiKey {} is not in the device registry; install its certificates and run \
                 `tvc keys refresh-yubikey --serial {}` first",
                self.serial,
                self.serial,
            )
        })?;
        let operator_public_key = entry.public_key;

        let org = config.orgs.get_mut(&self.org_alias).with_context(|| {
            format!(
                "active organization '{}' disappeared from config",
                self.org_alias
            )
        })?;

        let name = self.record.name.clone();
        org.operators.push(self.record);

        if self.make_default {
            org.default_operator_kind = OperatorKind::Yubikey;
        }

        Ok(YubikeyOperatorAdded {
            name,
            serial: self.serial,
            operator_public_key,
            made_default: self.make_default,
            org_alias: self.org_alias,
        })
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Default, Debug))]
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

#[derive(Default, Serialize)]
#[cfg_attr(test, derive(Debug))]
#[serde(rename_all = "camelCase")]
pub struct YubikeyOperatorAdded {
    name: String,
    serial: YubiKeySerial,
    /// The composite `encrypt_public ‖ sign_public` operator key.
    operator_public_key: QosOperatorPublicKey,
    made_default: bool,
    #[serde(skip)]
    org_alias: String,
}

impl From<YubikeyOperatorAdded> for Outcome {
    fn from(added: YubikeyOperatorAdded) -> Self {
        Outcome::YubikeyOperatorAdded(added)
    }
}

impl Display for YubikeyOperatorAdded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let default_note = if self.made_default {
            "\nIt is now the organization's default operator."
        } else {
            ""
        };

        write!(
            f,
            r#"YubiKey operator added!

Operator name:       {}
Serial:              {}
Operator public key: {}{default_note}"#,
            self.name, self.serial, self.operator_public_key
        )
    }
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

fn recovery_toml(alias: &str, record: &OperatorRecord) -> Result<String> {
    let quoted_alias = serde_json::to_string(alias)
        .context("failed to quote organization alias for recovery record")?;
    let record =
        toml::to_string_pretty(record).context("failed to serialize operator recovery record")?;
    Ok(format!(
        r#"[[orgs.{quoted_alias}.operators]]
{}"#,
        record.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{HostedOperatorRecord, OrgConfig};
    use crate::yubikey::SlotStatus;
    use crate::yubikey::test_support::{self, FakeDevice};
    use indexmap::IndexMap;
    use std::path::PathBuf;

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

    fn args(kind: CreateKind) -> Args {
        Args {
            kind,
            name: None,
            wallet_name: None,
            wallet_id: None,
            account_path: None,
            serial: None,
            default: false,
        }
    }

    #[test]
    fn a_hosted_plan_applies_the_hosted_defaults() {
        let plan = CreatePlan::try_from(args(CreateKind::Hosted)).unwrap();

        let CreatePlan::Hosted { spec, make_default } = plan else {
            panic!("a hosted invocation must produce a hosted plan");
        };
        assert_eq!(
            spec,
            HostedOperatorSpec {
                name: DEFAULT_OPERATOR_NAME.to_string(),
                wallet: HostedOperatorWallet::New(DEFAULT_WALLET_NAME.to_string()),
                path: DEFAULT_HOSTED_OPERATOR_BASE_PATH.to_string(),
            }
        );
        assert!(!make_default);
    }

    #[test]
    fn a_serial_is_rejected_for_the_hosted_kind() {
        let error = CreatePlan::try_from(Args {
            serial: Some(YubiKeySerial::from(0x01c9_5c1f)),
            ..args(CreateKind::Hosted)
        })
        .err()
        .expect("--serial must be rejected for --kind hosted");

        assert_eq!(
            error.to_string(),
            "--serial is only valid with --kind yubikey"
        );
    }

    #[test]
    fn wallet_flags_are_rejected_for_the_yubikey_kind() {
        let error = CreatePlan::try_from(Args {
            wallet_name: Some("wallet".to_string()),
            ..args(CreateKind::Yubikey)
        })
        .err()
        .expect("--wallet-name must be rejected for --kind yubikey");

        assert_eq!(
            error.to_string(),
            "--wallet-name, --wallet-id, and --account-path are only valid with --kind hosted"
        );
    }

    fn config_with_active_org() -> Config {
        Config {
            active_org: Some("default".to_string()),
            orgs: IndexMap::from([(
                "default".to_string(),
                OrgConfig {
                    id: Uuid::from_u128(0x123),
                    api_key_path: PathBuf::from("api-key.json"),
                    api_base_url: "https://api.turnkey.com".to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators: Vec::new(),
                    extra: toml::Table::new(),
                },
            )]),
            ..Config::default()
        }
    }

    fn yubikey_create(
        config: &Config,
        serial: YubiKeySerial,
        name: Option<String>,
        make_default: bool,
    ) -> YubikeyCreate {
        let record = config.orgs["default"]
            .new_yubikey_operator(serial, name)
            .unwrap();

        YubikeyCreate {
            record,
            serial,
            make_default,
            org_alias: "default".to_string(),
        }
    }

    #[test]
    fn a_registered_serial_is_added_without_any_device_operation() {
        let mut config = config_with_active_org();
        // Both slots empty: any device operation would fail loudly.
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        let composite = device.operator_public_key();
        config.yubikeys.register(test_support::serial(), composite);

        let create = yubikey_create(&config, test_support::serial(), None, false);
        let added = create.execute(&mut config).unwrap();

        assert_eq!(added.name, "yubikey-01c95c1f");
        assert_eq!(added.serial, test_support::serial());
        assert_eq!(added.operator_public_key, composite);

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Local);
        assert_eq!(
            org.operators,
            vec![OperatorRecord::yubikey(test_support::serial())]
        );
    }

    #[test]
    fn a_duplicate_org_reference_is_refused() {
        let mut config = config_with_active_org();
        config
            .orgs
            .get_mut("default")
            .unwrap()
            .operators
            .push(OperatorRecord::yubikey(test_support::serial()));

        let error = config.orgs["default"]
            .new_yubikey_operator(test_support::serial(), None)
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "YubiKey 01c95c1f is already an operator of this organization"
        );
        assert!(config.yubikeys.get(test_support::serial()).is_none());
    }

    #[test]
    fn an_unregistered_serial_is_refused() {
        let mut config = config_with_active_org();
        let create = yubikey_create(&config, test_support::serial(), None, false);
        let error = create.execute(&mut config).unwrap_err();

        assert!(
            error.to_string().contains("is not in the device registry"),
            "{error}"
        );
    }
}
