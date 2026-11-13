//! Create a TVC operator — hosted (the default) or YubiKey — and save it to
//! the active organization.

use crate::{
    client::build_client,
    commands::Run,
    config::turnkey::{
        Config, HostedOperatorRecord, OperatorKind, OperatorRecord, OperatorRecordKind,
        QosOperatorPublicKey, YubiKeyOperatorRecord, YubiKeySerial,
    },
    operator::{DEFAULT_HOSTED_OPERATOR_BASE_PATH, ensure_authenticated_org, timestamp_ms},
    outcome::Outcome,
    output::StdCtx,
    prompts,
    terminal::TerminalText,
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::{ArgGroup, Args as ClapArgs, ValueEnum, builder::NonEmptyStringValueParser};
use serde::Serialize;
use std::{
    fmt::{self, Display, Formatter},
    path::Path,
};
use tracing::instrument;
use turnkey_client::generated::CreateTvcOperatorIntent;
use turnkey_proofs::tvc::{OperatorExpectation, RolePublicKey, TvcProvenance};
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

    /// Expand human output with the checks and public proof evidence.
    #[arg(long)]
    verbose_proofs: bool,
}

/// A parsed `operator create` invocation: kind-specific inputs separated,
/// kind-incompatible flags rejected, and hosted defaults applied — all
/// before configuration or credentials are touched.
enum CreatePlan {
    Hosted {
        spec: HostedOperatorSpec,
        make_default: bool,
        verbose_proofs: bool,
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
            verbose_proofs,
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
                    verbose_proofs,
                })
            }
            CreateKind::Yubikey => {
                ensure!(
                    !verbose_proofs,
                    "--verbose-proofs is only valid with --kind hosted"
                );
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

impl Run for Args {
    type Outcome = Outcome;

    #[instrument(skip_all)]
    async fn run(self, ctx: &mut StdCtx, mut config: Config) -> Result<Outcome> {
        match CreatePlan::try_from(self)? {
            CreatePlan::Hosted {
                spec,
                make_default,
                verbose_proofs,
            } => {
                let mut auth = build_client(&config).await?;
                let _ = TvcProvenance::for_api_url(&auth.api_base_url)?;
                let (alias, configured_org_id, configured_api_base_url) = config
                    .active_org_config()
                    .map(|(alias, org)| (alias.clone(), org.id.as_str(), org.api_base_url.as_str()))
                    .context("No active organization. Run `tvc login` first.")?;

                ensure_authenticated_org(&auth.org_id, configured_org_id)?;
                ensure!(
                    auth.api_base_url == configured_api_base_url,
                    "authenticated API environment differs from the operator's configured organization"
                );

                let HostedOperatorSpec { name, wallet, path } = spec;

                let (wallet_name, wallet_id) = match wallet {
                    HostedOperatorWallet::New(name) => (Some(name), None),
                    HostedOperatorWallet::Existing(id) => (None, Some(id.to_string())),
                };

                let intent = CreateTvcOperatorIntent {
                    wallet_name,
                    wallet_id,
                    path,
                    operator_name: name,
                };
                // Reject unsupported environments and malformed expectations before submission.
                let _ = OperatorExpectation::parse(
                    auth.org_id.clone(),
                    intent.wallet_id.clone(),
                    intent.path.clone(),
                )?;
                auth.client = auth.client.with_app_proofs();
                crate::pending_creation::submit(
                    ctx,
                    auth,
                    crate::pending_creation::Creation::Operator {
                        intent,
                        org_alias: alias,
                        make_default,
                    },
                    timestamp_ms()?,
                    verbose_proofs,
                )
                .await
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

                Ok(Outcome::YubikeyOperatorAdded(added))
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{field} must be a UUID")]
pub(crate) struct MalformedHostedOperatorResponse {
    field: &'static str,
    #[source]
    source: uuid::Error,
}

pub(crate) async fn save_hosted_operator(
    config_path: &Path,
    alias: &str,
    organization_id: &str,
    api_base_url: &str,
    name: String,
    make_default: bool,
    (wallet_id, path, encrypt, sign, operator_id): (
        String,
        String,
        RolePublicKey,
        RolePublicKey,
        String,
    ),
) -> Result<OperatorCreated> {
    let hosted = HostedOperatorRecord {
        wallet_id: Uuid::parse_str(&wallet_id).map_err(|source| {
            MalformedHostedOperatorResponse {
                field: "accepted wallet ID",
                source,
            }
        })?,
        operator_id: Uuid::parse_str(&operator_id).map_err(|source| {
            MalformedHostedOperatorResponse {
                field: "unsigned operator routing label",
                source,
            }
        })?,
        path,
        encrypt_public_key: encrypt.to_string(),
        sign_public_key: sign.to_string(),
        extra: toml::Table::new(),
    };
    let _lock = crate::util::lock_owner_only_file(&config_path.with_extension("toml.lock"))
        .context("cannot lock config for hosted operator registration")?;
    let content = std::fs::read_to_string(config_path)
        .context("failed to reload config for hosted operator registration")?;
    let mut config = Config::from_toml(&content)?;
    let org = config.orgs.get_mut(alias).with_context(|| {
        format!(
            "original organization '{}' disappeared from config",
            TerminalText(alias).single_line()
        )
    })?;
    ensure!(
        org.id == organization_id,
        "original organization alias now refers to a different organization"
    );
    ensure!(
        org.api_base_url == api_base_url,
        "original organization alias now refers to a different API environment"
    );
    let record = OperatorRecord {
        name: name.clone(),
        kind: OperatorRecordKind::Hosted(hosted.clone()),
    };
    if register_hosted_operator(org, record, make_default)? {
        config.save_to_path(config_path).await.context("accepted hosted operator could not be saved; resume the saved creation to finish registration")?;
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

fn register_hosted_operator(
    org: &mut crate::config::turnkey::OrgConfig,
    record: OperatorRecord,
    make_default: bool,
) -> Result<bool> {
    let OperatorRecordKind::Hosted(hosted) = &record.kind else {
        bail!("expected hosted operator record")
    };
    let mut matched = false;
    for existing in &org.operators {
        if let OperatorRecordKind::Hosted(previous) = &existing.kind
            && (previous.operator_id == hosted.operator_id
                || (previous.encrypt_public_key == hosted.encrypt_public_key
                    && previous.sign_public_key == hosted.sign_public_key))
        {
            ensure!(
                existing.name == record.name
                    && previous.operator_id == hosted.operator_id
                    && previous.wallet_id == hosted.wallet_id
                    && previous.path == hosted.path
                    && previous.encrypt_public_key == hosted.encrypt_public_key
                    && previous.sign_public_key == hosted.sign_public_key,
                "saved operator conflicts with an existing local operator; no config was changed"
            );
            matched = true;
        }
    }
    if matched {
        return Ok(false);
    }
    org.operators.push(record);
    if make_default {
        org.default_operator_kind = OperatorKind::Hosted;
    }
    Ok(true)
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
            TerminalText(&self.name).single_line(),
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
            TerminalText(&self.name).single_line(),
            self.serial,
            self.operator_public_key
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
    use std::collections::BTreeMap;
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

    fn finalization_config() -> (Config, HostedOperatorRecord) {
        let public = qos_p256::P256Pair::generate()
            .unwrap()
            .public_key()
            .to_bytes();
        let hosted = HostedOperatorRecord {
            encrypt_public_key: hex::encode(&public[..65]),
            sign_public_key: hex::encode(&public[65..]),
            ..match hosted_record().kind {
                OperatorRecordKind::Hosted(hosted) => hosted,
                _ => panic!("expected hosted fixture"),
            }
        };
        (
            Config {
                orgs: BTreeMap::from([(
                    "original".into(),
                    OrgConfig {
                        id: "org-id".into(),
                        api_key_path: PathBuf::from("unused"),
                        api_base_url: "https://api.turnkey.com".into(),
                        default_operator_kind: OperatorKind::Local,
                        operators: Vec::new(),
                        extra: toml::Table::new(),
                    },
                )]),
                ..Config::default()
            },
            hosted,
        )
    }

    async fn finalize_at(
        path: &Path,
        hosted: &HostedOperatorRecord,
        make_default: bool,
    ) -> Result<OperatorCreated> {
        save_hosted_operator(
            path,
            "original",
            "org-id",
            "https://api.turnkey.com",
            "operator".into(),
            make_default,
            (
                hosted.wallet_id.to_string(),
                hosted.path.clone(),
                RolePublicKey::parse(&hosted.encrypt_public_key).unwrap(),
                RolePublicKey::parse(&hosted.sign_public_key).unwrap(),
                hosted.operator_id.to_string(),
            ),
        )
        .await
    }

    #[tokio::test]
    async fn repeated_finalization_preserves_existing_config_and_later_default_choice() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.toml");
        let (config, hosted) = finalization_config();
        config.save_to_path(&path).await.unwrap();
        finalize_at(&path, &hosted, true).await.unwrap();
        let mut config = Config::from_toml(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let org = config.orgs.get_mut("original").unwrap();
        assert_eq!(org.operators.len(), 1);
        assert_eq!(org.default_operator_kind, OperatorKind::Hosted);
        org.default_operator_kind = OperatorKind::Local;
        let OperatorRecordKind::Hosted(existing) = &mut org.operators[0].kind else {
            panic!("expected hosted record")
        };
        existing.extra.insert(
            "future_field".into(),
            toml::Value::String("preserve".into()),
        );
        config.save_to_path(&path).await.unwrap();
        let before = format!(
            "{}\n# preserve this comment too\n",
            std::fs::read_to_string(&path).unwrap()
        );
        std::fs::write(&path, &before).unwrap();
        finalize_at(&path, &hosted, true).await.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), before);
    }

    #[tokio::test]
    async fn finalization_refuses_conflicts_and_changed_original_context_without_writing() {
        for case in [
            "same-id",
            "same-keys",
            "renamed",
            "organization",
            "environment",
        ] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("config.toml");
            let (mut config, hosted) = finalization_config();
            let org = config.orgs.get_mut("original").unwrap();
            let mut existing = hosted.clone();
            match case {
                "same-id" => existing.sign_public_key = "different-key".into(),
                "same-keys" => {
                    existing.operator_id = "33333333-3333-4333-8333-333333333333".parse().unwrap()
                }
                "organization" => org.id = "different-org".into(),
                "environment" => {
                    org.api_base_url = "https://api.preprod.turnkey.engineering".into()
                }
                "renamed" => (),
                _ => panic!("unknown conflict fixture"),
            }
            org.operators.push(OperatorRecord {
                name: if case == "renamed" {
                    "different-name"
                } else {
                    "operator"
                }
                .into(),
                kind: OperatorRecordKind::Hosted(existing),
            });
            config.save_to_path(&path).await.unwrap();
            let before = std::fs::read(&path).unwrap();
            assert!(finalize_at(&path, &hosted, true).await.is_err(), "{case}");
            assert_eq!(std::fs::read(&path).unwrap(), before, "{case}");
        }
    }

    #[test]
    fn malformed_routing_labels_preserve_classification() {
        let error: anyhow::Error = MalformedHostedOperatorResponse {
            field: "unsigned operator routing label",
            source: Uuid::parse_str("not-a-uuid").unwrap_err(),
        }
        .into();
        assert_eq!(
            serde_json::to_value(crate::errors::classify(&error).code).unwrap(),
            serde_json::json!("malformed_response")
        );
        assert!(error.chain().any(|cause| cause.is::<uuid::Error>()));
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
            verbose_proofs: false,
        }
    }

    #[test]
    fn a_hosted_plan_applies_the_hosted_defaults() {
        let plan = CreatePlan::try_from(args(CreateKind::Hosted)).unwrap();

        let CreatePlan::Hosted {
            spec,
            make_default,
            verbose_proofs,
        } = plan
        else {
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
        assert!(!verbose_proofs);
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
            orgs: BTreeMap::from([(
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
