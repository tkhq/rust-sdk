//! TVC operator resolution and manifest approval.
//!
//! Operator semantics are isolated here: resolution decides which backend an
//! operator uses and seals that decision behind the [`Signer`] port, so
//! everything downstream treats an operator as identity plus a signing
//! capability. Hosted-specific behavior lives in [`hosted`].

pub(crate) mod hosted;

pub use hosted::DEFAULT_HOSTED_OPERATOR_BASE_PATH;
pub(crate) use hosted::{ResolvedHostedOperator, hosted_activity_error};

use crate::{
    approvals::ValidatedManifest,
    client::build_client,
    config::turnkey::{
        Config, OperatorKind, QosOperatorPublicKey, StoredQosOperatorKey, YubiKeySerial,
    },
    local_operator_key::{
        LocalOperatorSeedSource, resolve_local_operator, resolve_registered_local_operator,
    },
    pair::{LocalPair, Pair, Signer},
    yubikey::{DeviceError, DeviceOps, Pin},
};
use anyhow::{Context, Result, anyhow, bail, ensure};
use hosted::HostedSigner;
use p256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use qos_core::protocol::services::boot::Approval;
use std::{
    fmt::{self, Debug, Display, Formatter},
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use uuid::Uuid;

/// A validated, uncompressed P-256 operator public key.
///
/// String parsing accepts bare hexadecimal input with surrounding whitespace.
/// Display always emits canonical lowercase hexadecimal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorPublicKey(PublicKey);

/// Error returned when parsing an [`OperatorPublicKey`].
#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum OperatorPublicKeyParseError {
    /// The input was empty after trimming surrounding whitespace.
    #[error("must not be empty")]
    Empty,
    /// The input was not bare hexadecimal.
    #[error("must be bare hex encoded")]
    InvalidHex,
    /// The bytes were not an uncompressed 65-byte SEC1 point.
    #[error("must be a 65-byte uncompressed P-256 public key")]
    InvalidEncoding,
    /// The bytes did not identify a valid point on the P-256 curve.
    #[error("is not a valid P-256 point")]
    InvalidPoint,
}

impl FromStr for OperatorPublicKey {
    type Err = OperatorPublicKeyParseError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let value = value.trim();
        if value.is_empty() {
            return Err(OperatorPublicKeyParseError::Empty);
        }

        let bytes = hex::decode(value).map_err(|_| OperatorPublicKeyParseError::InvalidHex)?;
        if bytes.len() != 65 || bytes.first() != Some(&0x04) {
            return Err(OperatorPublicKeyParseError::InvalidEncoding);
        }

        PublicKey::from_sec1_bytes(&bytes)
            .map(Self)
            .map_err(|_| OperatorPublicKeyParseError::InvalidPoint)
    }
}

impl Display for OperatorPublicKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(self.0.to_encoded_point(false).as_bytes()))
    }
}

/// A non-serializable operator with its credentials resolved for use: common
/// identity plus the signing capability sealed behind the [`Signer`] port.
pub(crate) struct ResolvedOperator {
    /// Absent for an ad-hoc local seed override.
    name: Option<String>,
    /// Always present for hosted operators and optional for local operators.
    operator_id: Option<Uuid>,
    signer: Box<dyn Signer>,
}

// The signer is a trait object with nothing printable, so the derive is off
// the table; the identity fields keep test failures readable.
impl Debug for ResolvedOperator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResolvedOperator")
            .field("name", &self.name)
            .field("operator_id", &self.operator_id)
            .finish_non_exhaustive()
    }
}

impl ResolvedOperator {
    pub(crate) fn id(&self) -> Option<Uuid> {
        self.operator_id
    }

    pub(crate) async fn approve_manifest(
        &self,
        manifest: &ValidatedManifest<'_>,
    ) -> Result<Approval> {
        let public_key = self.signer.public_key();
        let member = {
            manifest
                .manifest_set()
                .members
                .iter()
                .find(|member| member.pub_key == public_key)
                .cloned()
                .ok_or_else(|| match &self.name {
                    Some(name) => anyhow!(
                        "operator '{name}' ({}) not part of manifest set",
                        hex::encode(public_key)
                    ),
                    None => anyhow!(
                        "operator ({}) not part of manifest set",
                        hex::encode(public_key)
                    ),
                })
        }?;
        let signature = self.signer.sign(&manifest.manifest_hash()).await?;
        let approval = Approval { signature, member };

        // Membership is already proven — `member` came out of the manifest
        // set — so verifying the fresh signature is the only remaining check.
        manifest.verify_approval(&approval)?;

        Ok(approval)
    }
}

pub(crate) fn ensure_authenticated_org(
    authenticated_org_id: Uuid,
    configured_org_id: Uuid,
) -> Result<()> {
    ensure!(
        authenticated_org_id == configured_org_id,
        "authenticated organization ({authenticated_org_id}) does not match configured organization ({configured_org_id})"
    );
    Ok(())
}

/// What the caller requires of the operator being resolved.
///
/// Checked during resolution, so an unsatisfiable requirement is rejected
/// before any backend dependencies (credentials, API clients) are acquired —
/// a machine with no credentials at all still gets the real refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignerRequirement {
    Any,
    /// The approval must be producible without the network (`--skip-post`).
    OfflineApproval,
}

/// One approval signer selected by its manifest-set public key. The command
/// resolves selection before this boundary; operator defaults never
/// participate in approval signing.
pub(crate) enum OperatorSelection {
    /// One-shot local credentials supplied on the command line.
    ExplicitLocal {
        pair: LocalPair,
        post_operator_id: Option<Uuid>,
    },
    /// A registered local key file, retaining its existing optional identity.
    Local {
        name: String,
        key_path: PathBuf,
        configured_operator_id: Option<String>,
        post_operator_id: Option<Uuid>,
    },
    /// A Turnkey-hosted signer with its validated registry metadata.
    Hosted(ResolvedHostedOperator),
    /// A registered hardware signer and the server identity used for posting.
    Yubikey {
        name: String,
        serial: YubiKeySerial,
        pin: Pin,
        post_operator_id: Option<Uuid>,
    },
}

/// A YubiKey operator and its already-acquired PIN, resolved at a command
/// endpoint before shared operator dispatch begins.
pub(crate) struct SelectedYubiKey {
    serial: YubiKeySerial,
    pin: Pin,
}

impl SelectedYubiKey {
    pub(crate) fn new(serial: YubiKeySerial, pin: Pin) -> Self {
        Self { serial, pin }
    }
}

/// One operator identity known for the active org, labeled with its registry
/// name when it has one. The ID stays a string: callers that need a UUID
/// parse at their own boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OperatorCandidate {
    pub id: String,
    pub name: Option<String>,
    /// The key this identity is proven to use. IDs remembered from an app
    /// response predate key-aware persistence, so they deliberately carry
    /// `None` and cannot override an app config by inference.
    pub public_key: Option<QosOperatorPublicKey>,
}

impl Display for OperatorCandidate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{name} ({})", self.id),
            None => f.write_str(&self.id),
        }
    }
}

/// A share-decryption backend selected by
/// [`Config::select_operator_pair_source`] from config and endpoint-resolved
/// inputs. [`Self::resolve`] performs file reads and device verification.
pub(crate) enum OperatorPairSource {
    /// An explicit seed from the command line.
    Seed(LocalOperatorSeedSource),
    /// The sole registered local operator's key file.
    RegisteredKeyFile(PathBuf),
    /// The sole registered yubikey operator's device.
    Yubikey { serial: YubiKeySerial, pin: Pin },
}

impl OperatorPairSource {
    /// Resolve the selected backend into a usable pair.
    pub(crate) async fn resolve<D, O>(
        self,
        config: &Config,
        open_device: O,
    ) -> Result<Box<dyn Pair>>
    where
        D: DeviceOps + Send + 'static,
        O: FnOnce(YubiKeySerial) -> Result<D, DeviceError>,
    {
        match self {
            Self::Seed(source) => Ok(Box::new(
                resolve_local_operator(config, Some(source)).await?,
            )),
            Self::RegisteredKeyFile(path) => {
                Ok(Box::new(resolve_registered_local_operator(path).await?))
            }
            Self::Yubikey { serial, pin } => {
                let device = open_device(serial)?;
                Ok(Box::new(config.resolve_yubikey(serial, device, pin).await?))
            }
        }
    }
}

/// Operator semantics of the registry [`Config`] holds; the methods live
/// here, with the rest of operator resolution, rather than in the config
/// module.
impl Config {
    /// Best-effort read of the active org's operator public key under its
    /// default backend, as qos composite hex, for prompt and template
    /// defaults. The local backend reads the sole registered key file; the
    /// hosted backend joins the sole record's stored points; the yubikey
    /// backend reads the registry's cached key, so the device need not be
    /// connected. `None` on any miss.
    pub(crate) async fn default_operator_public_key(&self) -> Option<String> {
        let (_, org) = self.active_org_config()?;

        match org.default_operator_kind {
            OperatorKind::Local => {
                let (_, local) = org.select_local_operator().ok()?;
                let operator_key = StoredQosOperatorKey::load(&local.key_path).await.ok()??;
                Some(operator_key.public_key.to_string())
            }
            OperatorKind::Hosted => {
                let (_, hosted) = org.select_hosted_operator().ok()?;
                Some(format!(
                    "{}{}",
                    hosted.encrypt_public_key, hosted.sign_public_key
                ))
            }
            OperatorKind::Yubikey => {
                let (_, yubikey) = org.select_yubikey_operator(None).ok()?;
                let entry = self.yubikeys.get(yubikey.serial)?;
                Some(entry.public_key.to_string())
            }
        }
    }

    /// Acquire the backend for an approval signer already selected by public
    /// key. Config defaults are intentionally absent from this boundary.
    pub(crate) async fn resolve_operator<D, O>(
        &self,
        open_device: O,
        selection: OperatorSelection,
        requirement: SignerRequirement,
    ) -> Result<ResolvedOperator>
    where
        D: DeviceOps + Send + 'static,
        O: FnOnce(YubiKeySerial) -> Result<D, DeviceError>,
    {
        match selection {
            OperatorSelection::ExplicitLocal {
                pair,
                post_operator_id,
            } => Ok(ResolvedOperator {
                name: None,
                operator_id: post_operator_id,
                signer: Box::new(pair),
            }),
            OperatorSelection::Local {
                name,
                key_path,
                configured_operator_id,
                post_operator_id,
            } => {
                let configured_operator_id = configured_operator_id
                    .as_deref()
                    .map(|id| {
                        Uuid::parse_str(id)
                            .map_err(|_| anyhow!("configured local operator ID must be a UUID"))
                    })
                    .transpose()?;

                let resolved_operator_id = match (configured_operator_id, post_operator_id) {
                    (Some(configured), Some(requested)) => {
                        ensure!(
                            configured == requested,
                            "requested operator ID ({requested}) does not match configured local operator ID ({configured})"
                        );
                        Some(configured)
                    }
                    (configured, requested) => configured.or(requested),
                };

                Ok(ResolvedOperator {
                    name: Some(name),
                    operator_id: resolved_operator_id,
                    signer: Box::new(resolve_registered_local_operator(key_path).await?),
                })
            }
            OperatorSelection::Hosted(hosted) => {
                if requirement == SignerRequirement::OfflineApproval {
                    bail!("--skip-post is not supported for hosted operators");
                }

                let auth = build_client(self).await?;
                ensure_authenticated_org(auth.org_id, hosted.organization_id())?;

                Ok(ResolvedOperator {
                    name: Some(hosted.name().to_string()),
                    operator_id: Some(hosted.operator_id()),
                    signer: Box::new(HostedSigner::new(hosted, auth)),
                })
            }
            OperatorSelection::Yubikey {
                name,
                serial,
                pin,
                post_operator_id,
            } => {
                let device = open_device(serial)?;
                let pair = self.resolve_yubikey(serial, device, pin).await?;

                Ok(ResolvedOperator {
                    name: Some(name),
                    operator_id: post_operator_id,
                    signer: Box::new(pair),
                })
            }
        }
    }

    /// Select the share-decryption backend from config and endpoint-resolved
    /// inputs before file or device I/O. An explicit seed is local, always,
    /// and skips the config. Hosted operators sign through the API and hold
    /// no local key material, so a hosted default is redirected to the hosted
    /// counterpart of the flow.
    pub(crate) fn select_operator_pair_source(
        &self,
        explicit: Option<LocalOperatorSeedSource>,
        selected_yubikey: Option<SelectedYubiKey>,
    ) -> Result<OperatorPairSource> {
        if let Some(explicit) = explicit {
            return Ok(OperatorPairSource::Seed(explicit));
        }

        let (alias, org) = self.active_org_config().ok_or_else(|| {
            anyhow!(
                "No active organization. Run `tvc login` first or provide \
                 --operator-seed or --operator-seed-path."
            )
        })?;

        match org.default_operator_kind {
            OperatorKind::Local => {
                let (_, local) = org
                    .select_local_operator()
                    .with_context(|| format!("org '{alias}'"))?;

                Ok(OperatorPairSource::RegisteredKeyFile(
                    local.key_path.clone(),
                ))
            }
            OperatorKind::Yubikey => {
                let selected_yubikey = selected_yubikey
                    .context("YubiKey operator input must be resolved before dispatch")?;
                let (_, yubikey) = org
                    .select_yubikey_operator(Some(selected_yubikey.serial))
                    .with_context(|| format!("org '{alias}'"))?;

                Ok(OperatorPairSource::Yubikey {
                    serial: yubikey.serial,
                    pin: selected_yubikey.pin,
                })
            }
            // TODO(TVC-202): remove hardcoded user-facing messages mentioning
            // other commands.
            OperatorKind::Hosted => bail!(
                "re-encrypting a local share needs the local operator key the share was \
                 encrypted to; hosted operators re-encrypt through Turnkey with \
                 `tvc deploy provision`"
            ),
        }
    }

    /// Operator identities known for the active org: registered hosted
    /// operators (config order) then the last `app create`'s manifest-set
    /// operator IDs, deduplicated by ID. Each source keeps one authoritative
    /// home — durable hosted identities live in the registry,
    /// app-create-minted ones only in the saved IDs.
    pub(crate) fn known_operator_candidates(&self) -> Vec<OperatorCandidate> {
        let Some((_, org)) = self.active_org_config() else {
            return Vec::new();
        };

        let mut candidates: Vec<OperatorCandidate> = org
            .hosted_operators()
            .map(|(name, hosted)| OperatorCandidate {
                id: hosted.operator_id.to_string(),
                name: name.to_owned().into(),
                public_key: format!("{}{}", hosted.encrypt_public_key, hosted.sign_public_key)
                    .parse()
                    .ok(),
            })
            .collect();

        for id in self.get_last_operator_ids().unwrap_or_default() {
            if candidates.iter().all(|candidate| candidate.id != id) {
                candidates.push(OperatorCandidate {
                    id,
                    name: None,
                    public_key: None,
                });
            }
        }

        candidates
    }
}

pub(crate) fn timestamp_ms() -> Result<u128> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::turnkey::{
        HostedOperatorRecord, LocalOperatorRecord, OperatorRecord, OperatorRecordKind, OrgConfig,
        QosOperatorPublicKey, YubiKeyOperatorRecord, YubiKeySerial,
    };
    use crate::pair::LocalPair;
    use crate::yubikey::test_support::{self, FakeDevice};
    use crate::yubikey::{Pin, SlotStatus};
    use indexmap::IndexMap;
    use qos_p256::P256Pair;
    use std::path::PathBuf;

    fn public_keys() -> (String, String) {
        let first = P256Pair::generate().unwrap().public_key().to_bytes();
        let second = P256Pair::generate().unwrap().public_key().to_bytes();
        (hex::encode(&first[..65]), hex::encode(&second[65..]))
    }

    const HOSTED_ID: &str = "11111111-1111-4111-8111-111111111111";

    fn config_with_operators(operators: Vec<OperatorRecord>) -> Config {
        Config {
            active_org: Some("active".to_string()),
            orgs: IndexMap::from([(
                "active".to_string(),
                OrgConfig {
                    id: Uuid::from_u128(0xA1),
                    api_key_path: PathBuf::from("api-key.json"),
                    api_base_url: "https://api.turnkey.com".to_string(),
                    default_operator_kind: OperatorKind::Local,
                    operators,
                    extra: toml::Table::new(),
                },
            )]),
            ..Config::default()
        }
    }

    fn hosted_operator(name: &str) -> OperatorRecord {
        let (encrypt_public_key, sign_public_key) = public_keys();
        OperatorRecord {
            name: name.to_string(),
            kind: OperatorRecordKind::Hosted(HostedOperatorRecord {
                operator_id: HOSTED_ID.parse().unwrap(),
                wallet_id: Uuid::from_u128(0xB1),
                path: "m/5527107'/0'/0'".to_string(),
                encrypt_public_key,
                sign_public_key,
                extra: toml::Table::new(),
            }),
        }
    }

    fn local_operator() -> OperatorRecord {
        OperatorRecord {
            name: "local".to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path: PathBuf::from("operator.json"),
                operator_id: None,
                extra: toml::Table::new(),
            }),
        }
    }

    fn yubikey_operator(serial: YubiKeySerial) -> OperatorRecord {
        OperatorRecord {
            name: "yubikey".to_string(),
            kind: OperatorRecordKind::Yubikey(YubiKeyOperatorRecord {
                serial,
                extra: toml::Table::new(),
            }),
        }
    }

    fn yubikey_default_config(serial: YubiKeySerial) -> Config {
        let mut config = config_with_operators(vec![yubikey_operator(serial)]);
        config.orgs.get_mut("active").unwrap().default_operator_kind = OperatorKind::Yubikey;
        config
    }

    #[tokio::test]
    async fn default_operator_public_key_reads_the_yubikey_registry_cache() {
        let serial = YubiKeySerial::from(0x01c9_5c1f);
        let composite = QosOperatorPublicKey::try_from([7u8; 130].as_slice()).unwrap();
        let mut config = yubikey_default_config(serial);
        config.yubikeys.register(serial, composite);

        assert_eq!(
            config.default_operator_public_key().await,
            Some(composite.to_string())
        );
    }

    #[tokio::test]
    async fn default_operator_public_key_is_none_for_an_unregistered_yubikey() {
        let config = yubikey_default_config(YubiKeySerial::from(0x01c9_5c1f));

        assert_eq!(config.default_operator_public_key().await, None);
    }

    fn provisioned_fake() -> FakeDevice {
        FakeDevice::new(SlotStatus::QosProvisioned, SlotStatus::QosProvisioned)
    }

    fn open_fake(
        device: FakeDevice,
    ) -> impl FnOnce(YubiKeySerial) -> Result<FakeDevice, DeviceError> {
        move |serial| {
            if serial == test_support::serial() {
                Ok(device)
            } else {
                Err(DeviceError::NotFound { serial })
            }
        }
    }

    fn fixed_pin() -> Pin {
        Pin::from(String::from_utf8(test_support::PIN.to_vec()).unwrap())
    }

    fn selected_yubikey() -> SelectedYubiKey {
        SelectedYubiKey::new(test_support::serial(), fixed_pin())
    }

    fn yubikey_selection(operator_id: Option<Uuid>) -> OperatorSelection {
        OperatorSelection::Yubikey {
            name: "yubikey".to_string(),
            serial: test_support::serial(),
            pin: fixed_pin(),
            post_operator_id: operator_id,
        }
    }

    fn registered_yubikey_config(device: &FakeDevice) -> Config {
        let mut config = yubikey_default_config(test_support::serial());
        config
            .yubikeys
            .register(test_support::serial(), device.operator_public_key());
        config
    }

    #[tokio::test]
    async fn yubikey_default_resolves_a_device_backed_signer() {
        let device = provisioned_fake();
        let composite = device.operator_public_key();
        let config = registered_yubikey_config(&device);

        let operator = config
            .resolve_operator(
                open_fake(device),
                yubikey_selection(None),
                SignerRequirement::Any,
            )
            .await
            .unwrap();

        assert_eq!(operator.signer.public_key(), composite.as_bytes());
        assert_eq!(operator.name, Some("yubikey".to_string()));
        assert_eq!(operator.id(), None);
    }

    #[tokio::test]
    async fn yubikey_default_satisfies_an_offline_approval() {
        let device = provisioned_fake();
        let config = registered_yubikey_config(&device);

        let operator = config
            .resolve_operator(
                open_fake(device),
                yubikey_selection(None),
                SignerRequirement::OfflineApproval,
            )
            .await
            .unwrap();

        assert_eq!(operator.name, Some("yubikey".to_string()));
    }

    #[tokio::test]
    async fn yubikey_default_passes_a_given_operator_id_through() {
        let device = provisioned_fake();
        let config = registered_yubikey_config(&device);
        let requested = Uuid::from_u128(0x22);

        let operator = config
            .resolve_operator(
                open_fake(device),
                yubikey_selection(Some(requested)),
                SignerRequirement::Any,
            )
            .await
            .unwrap();

        assert_eq!(operator.id(), Some(requested));
    }

    #[tokio::test]
    async fn an_explicit_seed_beats_a_yubikey_default() {
        let device = provisioned_fake();
        let config = registered_yubikey_config(&device);
        let seed_hex = "ab".repeat(32);
        let expected = LocalPair::from_hex_seed(&seed_hex).unwrap().public_key();
        let pair = LocalPair::from_hex_seed(&seed_hex).unwrap();

        let operator = config
            .resolve_operator(
                open_fake(device),
                OperatorSelection::ExplicitLocal {
                    pair,
                    post_operator_id: None,
                },
                SignerRequirement::Any,
            )
            .await
            .unwrap();

        assert_eq!(operator.signer.public_key(), expected);
        assert_eq!(operator.name, None);
    }

    #[tokio::test]
    async fn a_hosted_operator_id_beats_a_yubikey_default() {
        let device = provisioned_fake();
        let mut config = registered_yubikey_config(&device);
        config
            .orgs
            .get_mut("active")
            .unwrap()
            .operators
            .push(hosted_operator("hosted"));

        let error = config
            .resolve_operator(
                open_fake(device),
                OperatorSelection::Hosted(
                    config
                        .find_hosted_operator(&HOSTED_ID.parse().unwrap())
                        .unwrap()
                        .unwrap(),
                ),
                SignerRequirement::OfflineApproval,
            )
            .await
            .unwrap_err();

        // The hosted-by-id branch fired (pinning its rewording), so the
        // yubikey default never resolved.
        assert_eq!(
            error.to_string(),
            "--skip-post is not supported for hosted operators"
        );
    }

    #[tokio::test]
    async fn operator_pair_explicit_seed_is_local() {
        let seed_hex = "ab".repeat(32);
        let expected = LocalPair::from_hex_seed(&seed_hex).unwrap().public_key();

        // The pair never needs a PIN on the local path, even when none could
        // be prompted.
        let config = Config::default();
        let source = config
            .select_operator_pair_source(
                Some(LocalOperatorSeedSource::Value(seed_hex.parse().unwrap())),
                None,
            )
            .unwrap();
        let pair = source
            .resolve(&config, open_fake(provisioned_fake()))
            .await
            .unwrap();

        assert_eq!(pair.public_key(), expected);
    }

    #[tokio::test]
    async fn operator_pair_local_default_reads_the_registered_key_file() {
        let temp = tempfile::TempDir::new().unwrap();
        let key_path = temp.path().join("operator.json");
        let private_key = "ab".repeat(32);
        std::fs::write(
            &key_path,
            serde_json::to_string(&StoredQosOperatorKey {
                public_key: QosOperatorPublicKey::default(),
                private_key: private_key.clone(),
            })
            .unwrap(),
        )
        .unwrap();
        let config = config_with_operators(vec![OperatorRecord {
            name: "local".to_string(),
            kind: OperatorRecordKind::Local(LocalOperatorRecord {
                key_path,
                operator_id: None,
                extra: toml::Table::new(),
            }),
        }]);
        let expected = LocalPair::from_hex_seed(&private_key).unwrap().public_key();

        let source = config.select_operator_pair_source(None, None).unwrap();
        let pair = source
            .resolve(&config, open_fake(provisioned_fake()))
            .await
            .unwrap();

        assert_eq!(pair.public_key(), expected);
    }

    #[tokio::test]
    async fn operator_pair_yubikey_default_resolves_the_device_pair() {
        let device = provisioned_fake();
        let composite = device.operator_public_key();
        let config = registered_yubikey_config(&device);

        let source = config
            .select_operator_pair_source(None, Some(selected_yubikey()))
            .unwrap();
        let pair = source.resolve(&config, open_fake(device)).await.unwrap();

        assert_eq!(pair.public_key(), composite.as_bytes());
    }

    #[test]
    fn operator_pair_yubikey_default_without_resolved_endpoint_input_is_refused() {
        let device = provisioned_fake();
        let config = registered_yubikey_config(&device);

        let error = config
            .select_operator_pair_source(None, None)
            .err()
            .expect("selection must require resolved YubiKey input");

        assert!(format!("{error:#}").contains("must be resolved before dispatch"));
    }

    #[test]
    fn operator_pair_hosted_default_is_redirected() {
        let mut config = config_with_operators(vec![hosted_operator("hosted")]);
        config.orgs.get_mut("active").unwrap().default_operator_kind = OperatorKind::Hosted;

        let error = config
            .select_operator_pair_source(None, None)
            .err()
            .expect("a hosted default cannot decrypt locally");

        assert!(error.to_string().contains("tvc deploy provision"));
    }

    fn multi_yubikey_config(device: &FakeDevice) -> Config {
        let mut config = config_with_operators(vec![
            yubikey_operator(test_support::serial()),
            yubikey_operator(YubiKeySerial::from(0xdead_beef)),
        ]);
        config.orgs.get_mut("active").unwrap().default_operator_kind = OperatorKind::Yubikey;
        config
            .yubikeys
            .register(test_support::serial(), device.operator_public_key());
        config
    }

    #[tokio::test]
    async fn yubikey_default_with_multiple_records_resolves_the_explicit_serial() {
        let device = provisioned_fake();
        let composite = device.operator_public_key();
        let config = multi_yubikey_config(&device);

        let operator = config
            .resolve_operator(
                open_fake(device),
                yubikey_selection(None),
                SignerRequirement::Any,
            )
            .await
            .unwrap();

        assert_eq!(operator.signer.public_key(), composite.as_bytes());
    }

    #[test]
    fn choosing_an_unknown_serial_is_refused() {
        let config = config_with_operators(vec![yubikey_operator(test_support::serial())]);
        let (_, org) = config.active_org_config().unwrap();

        let error = org
            .select_yubikey_operator(Some(YubiKeySerial::from(0xdead_beef)))
            .unwrap_err();

        assert_eq!(error.to_string(), "no YubiKey operator has serial deadbeef");
    }

    #[test]
    fn candidates_are_registered_hosted_operators_then_saved_ids() {
        let mut config = config_with_operators(vec![local_operator(), hosted_operator("hosted")]);
        config
            .set_last_operator_ids(&["44444444-4444-4444-8444-444444444444".to_string()])
            .unwrap();

        let candidates = config.known_operator_candidates();

        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].id, HOSTED_ID);
        assert_eq!(candidates[0].name.as_deref(), Some("hosted"));
        assert!(candidates[0].public_key.is_some());
        assert_eq!(
            candidates[1],
            OperatorCandidate {
                id: "44444444-4444-4444-8444-444444444444".parse().unwrap(),
                name: None,
                public_key: None,
            }
        );
    }

    #[test]
    fn candidates_dedupe_saved_ids_against_the_registry() {
        let mut config = config_with_operators(vec![hosted_operator("hosted")]);
        config
            .set_last_operator_ids(&[HOSTED_ID.to_string()])
            .unwrap();

        let candidates = config.known_operator_candidates();

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].id, HOSTED_ID);
        assert_eq!(candidates[0].name.as_deref(), Some("hosted"));
        assert!(candidates[0].public_key.is_some());
    }

    #[test]
    fn candidates_are_empty_without_an_active_org() {
        assert_eq!(Config::default().known_operator_candidates(), Vec::new());
    }

    #[test]
    fn candidate_display_labels_named_operators() {
        let named = OperatorCandidate {
            id: HOSTED_ID.to_string(),
            name: Some("hosted".to_string()),
            public_key: None,
        };
        let unnamed = OperatorCandidate {
            id: HOSTED_ID.to_string(),
            name: None,
            public_key: None,
        };

        assert_eq!(named.to_string(), format!("hosted ({HOSTED_ID})"));
        assert_eq!(unnamed.to_string(), HOSTED_ID);
    }

    #[test]
    fn operator_public_key_parses_and_canonicalizes() {
        let (key, _) = public_keys();
        let parsed: OperatorPublicKey = format!("  {}  ", key.to_uppercase()).parse().unwrap();

        assert_eq!(parsed.to_string(), key);
    }

    #[test]
    fn operator_public_key_rejects_invalid_inputs() {
        assert_eq!(
            " ".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::Empty
        );
        assert_eq!(
            "not-hex".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidHex
        );
        assert_eq!(
            "04abcd".parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidEncoding
        );

        let (uncompressed, _) = public_keys();
        let public_key = PublicKey::from_sec1_bytes(&hex::decode(uncompressed).unwrap()).unwrap();
        let compressed = hex::encode(public_key.to_encoded_point(true).as_bytes());
        assert_eq!(
            compressed.parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidEncoding
        );

        let invalid_point = format!("04{}", "00".repeat(64));
        assert_eq!(
            invalid_point.parse::<OperatorPublicKey>().unwrap_err(),
            OperatorPublicKeyParseError::InvalidPoint
        );
    }

    #[test]
    fn authenticated_org_must_match_configured_org() {
        let authenticated = Uuid::from_u128(0xAA);
        let configured = Uuid::from_u128(0xCC);

        assert_eq!(
            ensure_authenticated_org(authenticated, configured)
                .unwrap_err()
                .to_string(),
            format!(
                "authenticated organization ({authenticated}) does not match \
                 configured organization ({configured})"
            )
        );
    }
}
