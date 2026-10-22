//! The share holder's review of a deployment manifest before a quorum-key
//! share is handed to the enclave that runs it.
//!
//! A manifest envelope's own approvals and attestation only prove the enclave
//! runs the manifest it carries; the share holder still has to establish that
//! manifest is one they trust. [`ManifestReview`] is that decision, and
//! [`Reviewed`] is the proof it was made.

use crate::commands::deploy::approve::interactive_approve;
use crate::config::app::{AppConfig, OperatorSetParams};
use crate::output::StdCtx;
use crate::prompts;
use crate::util::read_json_file;
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use std::collections::BTreeSet;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::path::PathBuf;

/// Flags choosing how the share holder reviews the manifest.
#[derive(Debug, ClapArgs)]
pub(crate) struct ManifestReviewArgs {
    /// DANGEROUS: skip interactive prompts for approving each aspect of the manifest
    /// before the share is re-encrypted to it.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_INTERACTIVE")]
    dangerous_skip_interactive: bool,

    /// Path to the app config whose `manifestSetParams` is trusted in place of the
    /// interactive manifest review: the manifest set must match it exactly and its
    /// approvals must meet the threshold.
    #[arg(
        long,
        value_name = "PATH",
        env = "TVC_TRUSTED_APP_CONFIG",
        conflicts_with = "dangerous_skip_interactive"
    )]
    trusted_app_config: Option<PathBuf>,
}

/// How the share holder establishes trust in a manifest.
pub(crate) enum ManifestReview {
    Interactive,
    TrustedAppConfig(TrustedManifestSet),
    DangerousSkip,
}

impl ManifestReview {
    /// Resolve the review the flags choose, reading the trusted app config if
    /// one is named. Without either flag the review is interactive, which
    /// requires `can_prompt`.
    pub(crate) async fn from_args(args: ManifestReviewArgs, can_prompt: bool) -> Result<Self> {
        match (args.trusted_app_config, args.dangerous_skip_interactive) {
            (Some(path), _) => {
                let app_config: AppConfig = read_json_file(&path, "trusted app config").await?;
                Ok(Self::TrustedAppConfig(
                    TrustedManifestSet::try_from(app_config)
                        .with_context(|| format!("trusted app config {}", path.display()))?,
                ))
            }
            (None, true) => Ok(Self::DangerousSkip),
            (None, false) if can_prompt => Ok(Self::Interactive),
            (None, false) => Err(prompts::error_required_in_non_interactive(
                "--trusted-app-config or --dangerous-skip-interactive",
            )),
        }
    }

    pub(crate) fn review<T: CarriesManifest>(
        self,
        ctx: &mut StdCtx,
        value: T,
    ) -> Result<Reviewed<T>> {
        let manifest_envelope = value.manifest_envelope();
        match self {
            Self::Interactive => interactive_approve(ctx, &manifest_envelope.clone().manifest())?,
            Self::TrustedAppConfig(trusted) => trusted.ensure_matches(manifest_envelope)?,
            Self::DangerousSkip => {}
        }
        Ok(Reviewed(value))
    }
}

/// A value carrying the manifest envelope a share is handed over against.
pub(crate) trait CarriesManifest {
    fn manifest_envelope(&self) -> &VersionedManifestEnvelope;
}

/// A value whose manifest passed the share holder's [`ManifestReview`]; only
/// that review constructs it.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct Reviewed<T>(T);

impl<T> Reviewed<T> {
    pub(crate) fn into_inner(self) -> T {
        self.0
    }

    #[cfg(test)]
    pub(crate) fn assume_reviewed(value: T) -> Self {
        Self(value)
    }
}

impl<T> Deref for Reviewed<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

/// The manifest set an app config declares, as the keys and threshold a
/// manifest must carry to be trusted.
#[cfg_attr(test, derive(Debug))]
pub(crate) struct TrustedManifestSet {
    threshold: NonZeroU32,
    members: BTreeSet<Vec<u8>>,
}

impl TryFrom<AppConfig> for TrustedManifestSet {
    type Error = anyhow::Error;

    fn try_from(app_config: AppConfig) -> Result<Self> {
        let Some(OperatorSetParams {
            threshold,
            new_operators,
            existing_operator_ids,
            name: _,
        }) = app_config.manifest_set_params
        else {
            bail!("manifestSetParams is required to trust the manifest set offline");
        };
        ensure!(
            existing_operator_ids.is_empty(),
            "manifestSetParams.existingOperatorIds cannot be checked offline; list those operators' public keys in newOperators"
        );
        let threshold =
            NonZeroU32::new(threshold).context("manifestSetParams.threshold must be at least 1")?;
        let members: BTreeSet<Vec<u8>> = new_operators
            .into_iter()
            .map(|operator| {
                hex::decode(&operator.public_key).with_context(|| {
                    format!(
                        "invalid public key for manifest set operator '{}'",
                        operator.name
                    )
                })
            })
            .collect::<Result<_>>()?;
        ensure!(
            threshold.get() as usize <= members.len(),
            "manifestSetParams.threshold ({threshold}) exceeds its {} distinct newOperators",
            members.len()
        );

        Ok(Self { threshold, members })
    }
}

impl TrustedManifestSet {
    fn ensure_matches(&self, manifest_envelope: &VersionedManifestEnvelope) -> Result<()> {
        let manifest_set = manifest_envelope.manifest_set();
        let members: BTreeSet<&[u8]> = manifest_set
            .members
            .iter()
            .map(|member| member.pub_key.as_slice())
            .collect();
        ensure!(
            manifest_set.threshold == self.threshold.get()
                && members
                    .into_iter()
                    .eq(self.members.iter().map(Vec::as_slice)),
            "manifest set does not match the trusted app config manifest set"
        );

        // Trusting the set only means something if enough of its members
        // approved this manifest, which `--dangerous-skip-verification` would
        // otherwise leave unchecked.
        manifest_envelope
            .check_approvals()
            .context("failed to verify manifest approvals")
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use qos_core::protocol::services::boot::{
        Manifest, ManifestEnvelope, ManifestSet, Namespace, NitroConfig, PatchSet, PivotConfig,
        QuorumMember, RestartPolicy, ShareSet, VersionedManifestEnvelope,
    };

    pub(crate) fn sample_manifest_envelope(
        quorum_key: Vec<u8>,
        share_set_members: Vec<QuorumMember>,
    ) -> VersionedManifestEnvelope {
        let envelope = ManifestEnvelope {
            manifest: Manifest {
                namespace: Namespace {
                    name: "test-namespace".to_string(),
                    nonce: 7,
                    quorum_key,
                },
                pivot: PivotConfig {
                    hash: [0; 32],
                    restart: RestartPolicy::Never,
                    bridge_config: vec![],
                    debug_mode: false,
                    args: vec![],
                },
                manifest_set: ManifestSet {
                    threshold: 0,
                    members: vec![],
                },
                share_set: ShareSet {
                    threshold: share_set_members.len() as u32,
                    members: share_set_members,
                },
                enclave: NitroConfig {
                    pcr0: vec![0; 48],
                    pcr1: vec![1; 48],
                    pcr2: vec![2; 48],
                    pcr3: vec![3; 48],
                    aws_root_certificate: vec![],
                    qos_commit: "test-commit".to_string(),
                },
                patch_set: PatchSet {
                    threshold: 0,
                    members: vec![],
                },
            },
            manifest_set_approvals: vec![],
            share_set_approvals: vec![],
        };
        envelope.into()
    }
}

#[cfg(test)]
mod tests {
    use super::TrustedManifestSet;
    use super::test_support::sample_manifest_envelope;
    use crate::config::app::{AppConfig, OperatorParams};
    use qos_core::protocol::services::boot::{
        Approval, ManifestSet, QuorumMember, VersionedManifestEnvelope,
    };
    use qos_p256::P256Pair;

    fn envelope_with_manifest_set(
        members: &[&P256Pair],
        threshold: u32,
        approvers: &[&P256Pair],
    ) -> VersionedManifestEnvelope {
        let member = |pair: &P256Pair| QuorumMember {
            alias: hex::encode(&pair.public_key().to_bytes()[..4]),
            pub_key: pair.public_key().to_bytes(),
        };
        let VersionedManifestEnvelope::V1(mut envelope) = sample_manifest_envelope(vec![], vec![])
        else {
            panic!("sample manifest envelope should be V1");
        };
        envelope.manifest.manifest_set = ManifestSet {
            threshold,
            members: members.iter().copied().map(member).collect(),
        };
        let manifest_hash = VersionedManifestEnvelope::from(envelope.clone()).manifest_hash();
        envelope.manifest_set_approvals = approvers
            .iter()
            .map(|pair| Approval {
                signature: pair.sign(&manifest_hash).unwrap(),
                member: member(pair),
            })
            .collect();
        envelope.into()
    }

    fn trusted_app_config(members: &[&P256Pair], threshold: u32) -> AppConfig {
        let mut app_config = AppConfig::template(None);
        let params = app_config.manifest_set_params.as_mut().unwrap();
        params.threshold = threshold;
        params.new_operators = members
            .iter()
            .map(|pair| OperatorParams {
                name: "operator".to_string(),
                public_key: hex::encode(pair.public_key().to_bytes()),
            })
            .collect();
        app_config
    }

    #[test]
    fn trusted_manifest_set_rejects_a_different_threshold() {
        let first = P256Pair::generate().unwrap();
        let second = P256Pair::generate().unwrap();
        let trusted =
            TrustedManifestSet::try_from(trusted_app_config(&[&first, &second], 2)).unwrap();

        let err = trusted
            .ensure_matches(&envelope_with_manifest_set(
                &[&first, &second],
                1,
                &[&first],
            ))
            .unwrap_err();

        assert_eq!(
            format!("{err:#}"),
            "manifest set does not match the trusted app config manifest set"
        );
    }

    #[test]
    fn trusted_manifest_set_requires_threshold_approvals() {
        let operator = P256Pair::generate().unwrap();
        let trusted = TrustedManifestSet::try_from(trusted_app_config(&[&operator], 1)).unwrap();

        let err = trusted
            .ensure_matches(&envelope_with_manifest_set(&[&operator], 1, &[]))
            .unwrap_err();

        assert_eq!(
            format!("{err:#}"),
            "failed to verify manifest approvals: not enough approvals"
        );
    }

    #[test]
    fn trusted_manifest_set_requires_inline_manifest_set_params() {
        let mut by_id = AppConfig::template(None);
        by_id.manifest_set_params = None;
        by_id.manifest_set_id = Some("manifest-set-id".to_string());
        let mut with_existing = trusted_app_config(&[], 1);
        with_existing
            .manifest_set_params
            .as_mut()
            .unwrap()
            .existing_operator_ids = vec!["operator-id".to_string()];

        assert_eq!(
            format!("{:#}", TrustedManifestSet::try_from(by_id).unwrap_err()),
            "manifestSetParams is required to trust the manifest set offline"
        );
        assert_eq!(
            format!(
                "{:#}",
                TrustedManifestSet::try_from(with_existing).unwrap_err()
            ),
            "manifestSetParams.existingOperatorIds cannot be checked offline; list those operators' public keys in newOperators"
        );
    }

    #[test]
    fn trusted_manifest_set_rejects_an_unreachable_threshold() {
        let operator = P256Pair::generate().unwrap();

        assert_eq!(
            format!(
                "{:#}",
                TrustedManifestSet::try_from(trusted_app_config(&[], 0)).unwrap_err()
            ),
            "manifestSetParams.threshold must be at least 1"
        );
        assert_eq!(
            format!(
                "{:#}",
                TrustedManifestSet::try_from(trusted_app_config(&[&operator, &operator], 2))
                    .unwrap_err()
            ),
            "manifestSetParams.threshold (2) exceeds its 1 distinct newOperators"
        );
    }
}
