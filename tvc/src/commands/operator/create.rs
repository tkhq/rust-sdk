//! Create a TVC operator — hosted (the default) or YubiKey — and save it to
//! the active organization.

use crate::{
    client::build_client,
    commands::Run,
    config::turnkey::{
        Config, OperatorKind, OperatorRecord, OperatorRecordKind, QosOperatorPublicKey,
        Registration, YubiKeyOperatorRecord, YubiKeySerial,
    },
    operator::{
        DEFAULT_HOSTED_OPERATOR_BASE_PATH, ensure_authenticated_org,
        hosted::CreateOperatorRequestResult, hosted_activity_error, timestamp_ms,
    },
    outcome::Outcome,
    output::StdCtx,
    prompts,
    yubikey::{self, ConnectedYubiKeys, DeviceError, DeviceOps, Pin, QosSlot, YubiKeySource},
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

    /// Serial (hex) of the YubiKey to use (yubikey only). A registered
    /// serial is added from the registry cache without touching hardware; an
    /// unregistered one is provisioned interactively. Only use this if you
    /// know the serial number; otherwise just run the command with a single
    /// YubiKey plugged in.
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
            CreatePlan::Hosted { spec, make_default } => Ok(CreateOutcome::Hosted(
                create_hosted(&mut config, spec, make_default).await?,
            )),
            CreatePlan::Yubikey {
                name,
                serial,
                make_default,
            } => {
                let can_prompt = !ctx.is_non_interactive() && prompts::stdin_can_prompt();

                let source = match serial {
                    Some(serial) if config.yubikeys.contains(serial) => {
                        YubiKeySource::Registered(serial)
                    }
                    Some(serial) if can_prompt => YubiKeySource::Provision(
                        ConnectedYubiKeys::from(ctx.connected_yubikeys()?).choose(Some(serial))?,
                    ),
                    Some(serial) => bail!(
                        "YubiKey {serial} is not in the device registry, and provisioning it \
                         is interactive (the PIN is prompted and the device must be touched); \
                         run interactively, or `tvc keys provision-yubikey --serial {serial}` \
                         first"
                    ),
                    None if can_prompt => {
                        YubiKeySource::prompt(&config.yubikeys, || ctx.connected_yubikeys())?
                    }
                    None => return Err(prompts::error_required_in_non_interactive("--serial")),
                };

                let make_default = make_default
                    || (can_prompt
                        && prompts::confirm(
                            "Make this the default operator for the organization?",
                            false,
                        )?);

                let pin = if matches!(source, YubiKeySource::Provision(_)) {
                    Some(Pin::from(prompts::password(
                        "YubiKey PIV PIN (the factory default is 123456; touch the device each time it blinks)",
                    )?))
                } else {
                    None
                };

                let create = YubikeyCreate {
                    name,
                    source,
                    make_default,
                    pin,
                };
                let added = create.execute(&mut config, yubikey::open)?;

                // Nothing is persisted yet; the remediation renders from the
                // typed outcome, and re-running is safe (provisioning is
                // idempotent and a duplicate serial is refused).
                let registry_recovery = match added.registration {
                    Registration::Added | Registration::Updated => format!(
                        "[[yubikeys]]\nserial = \"{}\"\npublic_key = \"{}\"\n\n",
                        added.serial, added.operator_public_key
                    ),
                    Registration::Unchanged => String::new(),
                };
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

{registry_recovery}{record_recovery}{default_note}"#
                    )
                })?;

                Ok(CreateOutcome::Yubikey(added))
            }
        }
    }
}

async fn create_hosted(
    config: &mut Config,
    spec: HostedOperatorSpec,
    make_default: bool,
) -> Result<OperatorCreated> {
    let (alias, configured_org_id) = config
        .active_org_config()
        .map(|(alias, org)| (alias.clone(), org.id.as_str()))
        .context("No active organization. Run `tvc login` first.")?;

    let auth = build_client(config).await?;
    ensure_authenticated_org(&auth.org_id, configured_org_id)?;

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

    let org = config
        .orgs
        .get_mut(&alias)
        .with_context(|| format!("active organization '{alias}' disappeared from config"))?;
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

    Ok(OperatorCreated {
        name,
        composite_public_key: format!("{}{}", hosted.encrypt_public_key, hosted.sign_public_key),
        operator_id: hosted.operator_id,
        wallet_id: hosted.wallet_id,
        encrypt_public_key: hosted.encrypt_public_key,
        sign_public_key: hosted.sign_public_key,
        saved: true,
    })
}

/// A resolved yubikey create: the device source is settled and the prompt
/// policies are endpoint decisions already made.
struct YubikeyCreate {
    name: Option<String>,
    source: YubiKeySource,
    make_default: bool,
    pin: Option<Pin>,
}

impl YubikeyCreate {
    /// The flow over any device boundary: enroll or reuse the source, add
    /// the serial-only org record, and optionally flip the default backend.
    /// Mutates only the in-memory config; [`Run::run`] supplies PC/SC and
    /// persists everything as one save.
    fn execute<D, O>(self, config: &mut Config, open_device: O) -> Result<YubikeyOperatorAdded>
    where
        D: DeviceOps,
        O: FnOnce(YubiKeySerial) -> Result<D, DeviceError>,
    {
        let alias = config
            .active_org_config()
            .map(|(alias, _)| alias.clone())
            .context("No active organization. Run `tvc login` first.")?;

        let (serial, enrolled) = match self.source {
            YubiKeySource::Registered(serial) => (serial, None),
            YubiKeySource::Provision(serial) => {
                let pin = self
                    .pin
                    .as_ref()
                    .context("a PIN must be resolved before provisioning a YubiKey")?;
                let mut device = open_device(serial)?;
                (
                    serial,
                    Some(config.enroll_yubikey(serial, &mut device, pin)?),
                )
            }
        };

        let (operator_public_key, registration) = match &enrolled {
            Some(enrolled) => (enrolled.public_key, enrolled.registration),
            None => {
                let entry = config.yubikeys.get(serial).ok_or_else(|| {
                    anyhow!(
                        "YubiKey {serial} is not in the device registry; run \
                         `tvc keys provision-yubikey --serial {serial}` to provision and \
                         register it"
                    )
                })?;

                (entry.public_key, Registration::Unchanged)
            }
        };

        let org = config
            .orgs
            .get_mut(&alias)
            .with_context(|| format!("active organization '{alias}' disappeared from config"))?;

        org.add_yubikey_operator(serial, self.name)
            .with_context(|| format!("org '{alias}'"))?;

        if self.make_default {
            org.default_operator_kind = OperatorKind::Yubikey;
        }

        let (record, _) = org
            .select_yubikey_operator(Some(serial))
            .with_context(|| format!("org '{alias}'"))?;

        Ok(YubikeyOperatorAdded {
            name: record.name.clone(),
            serial,
            operator_public_key,
            provisioned_slots: enrolled
                .map(|enrolled| enrolled.provisioned_slots)
                .unwrap_or_default(),
            registration,
            made_default: self.make_default,
            org_alias: alias,
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
    /// Slots newly provisioned by this run; empty when an already-registered
    /// serial or an already-provisioned device was used.
    provisioned_slots: Vec<QosSlot>,
    /// How the device registry changed; unchanged when a registered serial
    /// was reused.
    registration: Registration,
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
        let summary = if self.provisioned_slots.is_empty() {
            "YubiKey operator added!"
        } else {
            "YubiKey provisioned and added as an operator!"
        };
        let default_note = if self.made_default {
            "\nIt is now the organization's default operator."
        } else {
            ""
        };

        write!(
            f,
            r#"{summary}

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
    use crate::yubikey::test_support::{self, FakeDevice};
    use crate::yubikey::{Pin, SlotStatus};
    use std::collections::HashMap;
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
            orgs: HashMap::from([(
                "default".to_string(),
                OrgConfig {
                    id: "org-123".to_string(),
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

    fn fixed_pin() -> Pin {
        Pin::from(String::from_utf8(test_support::PIN.to_vec()).unwrap())
    }

    fn yubikey_create(source: YubiKeySource) -> YubikeyCreate {
        YubikeyCreate {
            name: None,
            source,
            make_default: false,
            pin: None,
        }
    }

    #[test]
    fn a_registered_serial_is_added_without_any_device_operation() {
        let mut config = config_with_active_org();
        // Both slots empty: any device operation would fail loudly.
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        let composite = device.operator_public_key();
        config.yubikeys.register(test_support::serial(), composite);

        let added = yubikey_create(YubiKeySource::Registered(test_support::serial()))
            .execute(&mut config, |_| -> Result<FakeDevice, DeviceError> {
                panic!("a registered source must not open a device")
            })
            .unwrap();

        assert_eq!(added.name, "yubikey-01c95c1f");
        assert_eq!(added.serial, test_support::serial());
        assert_eq!(added.operator_public_key, composite);
        assert_eq!(added.provisioned_slots, Vec::new());
        assert_eq!(added.registration, Registration::Unchanged);
        assert_eq!(device.provision_calls, Vec::new());
        assert_eq!(device.delete_calls, Vec::new());

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Local);
        assert_eq!(
            org.operators,
            vec![OperatorRecord::yubikey(test_support::serial())]
        );
    }

    #[test]
    fn a_provisioned_source_enrolls_and_adds_the_record() {
        let mut config = config_with_active_org();
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let added = YubikeyCreate {
            name: Some("signer".to_string()),
            source: YubiKeySource::Provision(test_support::serial()),
            make_default: true,
            pin: Some(fixed_pin()),
        }
        .execute(&mut config, |_| Ok(device))
        .unwrap();

        assert_eq!(added.name, "signer");
        assert_eq!(
            added.provisioned_slots,
            vec![QosSlot::Signing, QosSlot::KeyAgreement]
        );
        assert_eq!(added.registration, Registration::Added);
        assert!(added.made_default);
        assert_eq!(
            config
                .yubikeys
                .get(test_support::serial())
                .unwrap()
                .public_key,
            added.operator_public_key
        );

        let org = &config.orgs["default"];
        assert_eq!(org.default_operator_kind, OperatorKind::Yubikey);
        assert_eq!(org.operators[0].name, "signer");
    }

    #[test]
    fn a_duplicate_serial_is_refused_with_the_org_named() {
        let mut config = config_with_active_org();
        let device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);
        config
            .yubikeys
            .register(test_support::serial(), device.operator_public_key());

        yubikey_create(YubiKeySource::Registered(test_support::serial()))
            .execute(&mut config, |_| -> Result<FakeDevice, DeviceError> {
                panic!("a registered source must not open a device")
            })
            .unwrap();
        let error = yubikey_create(YubiKeySource::Registered(test_support::serial()))
            .execute(&mut config, |_| -> Result<FakeDevice, DeviceError> {
                panic!("a registered source must not open a device")
            })
            .unwrap_err();

        let rendered = format!("{error:#}");
        assert!(rendered.contains("org 'default'"), "{rendered}");
        assert!(
            rendered.contains("YubiKey 01c95c1f is already an operator of this organization"),
            "{rendered}"
        );
    }

    #[test]
    fn an_unregistered_registered_source_is_refused() {
        let mut config = config_with_active_org();
        let _device = FakeDevice::new(SlotStatus::Empty, SlotStatus::Empty);

        let error = yubikey_create(YubiKeySource::Registered(test_support::serial()))
            .execute(&mut config, |_| -> Result<FakeDevice, DeviceError> {
                panic!("a registered source must not open a device")
            })
            .unwrap_err();

        assert!(
            error.to_string().contains("is not in the device registry"),
            "{error}"
        );
    }
}
