//! The share holder's review of a deployment manifest before a quorum-key
//! share is handed to the enclave that runs it.
//!
//! A manifest envelope's own approvals and attestation only prove the enclave
//! runs the manifest it carries; the share holder still has to establish that
//! manifest is one they trust. [`ManifestReview`] is that decision, and
//! [`Reviewed`] is the proof it was made.

use crate::commands::deploy::approve::interactive_approve;
use crate::output::StdCtx;
use crate::prompts;
use anyhow::Result;
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::VersionedManifestEnvelope;
use std::ops::Deref;

/// Flags choosing how the share holder reviews the manifest.
#[derive(Debug, ClapArgs)]
pub(crate) struct ManifestReviewArgs {
    /// DANGEROUS: skip interactive prompts for approving each aspect of the manifest
    /// before the share is re-encrypted to it.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_INTERACTIVE")]
    dangerous_skip_interactive: bool,
}

/// How the share holder establishes trust in a manifest.
pub(crate) enum ManifestReview {
    Interactive,
    DangerousSkip,
}

impl ManifestReview {
    /// Resolve the review the flags choose. Without
    /// `--dangerous-skip-interactive` the review is interactive, which requires
    /// `can_prompt`.
    pub(crate) fn from_args(args: ManifestReviewArgs, can_prompt: bool) -> Result<Self> {
        if args.dangerous_skip_interactive {
            Ok(Self::DangerousSkip)
        } else if can_prompt {
            Ok(Self::Interactive)
        } else {
            Err(prompts::error_required_in_non_interactive(
                "--dangerous-skip-interactive",
            ))
        }
    }

    pub(crate) fn review<T: CarriesManifest>(
        self,
        ctx: &mut StdCtx,
        value: T,
    ) -> Result<Reviewed<T>> {
        match self {
            Self::Interactive => {
                interactive_approve(ctx, &value.manifest_envelope().clone().manifest())?
            }
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
