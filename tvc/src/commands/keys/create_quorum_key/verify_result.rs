//! Verify the result of a saved hosted quorum key creation.

use crate::{
    commands::Run,
    config::turnkey::Config,
    outcome::Outcome,
    output::StdCtx,
    pending_creation::{self, CreationKind},
};
use anyhow::{Context, Result, ensure};
use clap::{Args as ClapArgs, builder::NonEmptyStringValueParser};
use uuid::Uuid;

#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Local saved creation UUID printed by the original command.
    #[arg(
        required_unless_present = "dangerous_skip_verification",
        required_if_eq("dangerous_skip_verification", "false")
    )]
    operation_id: Option<Uuid>,

    /// Activity ID to look up, as printed by creation or shown in Turnkey.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    activity_id: String,

    /// Expand human output with the checks and public proof evidence.
    #[arg(long)]
    verbose_proofs: bool,

    /// Display the API result without verifying proofs or using a saved creation.
    #[arg(long, env = "TVC_DANGEROUS_SKIP_VERIFICATION")]
    dangerous_skip_verification: bool,
}

impl Run for Args {
    type Outcome = Outcome;

    async fn run(self, ctx: &mut StdCtx, config: Config) -> Result<Outcome> {
        if self.dangerous_skip_verification {
            ensure!(
                self.operation_id.is_none(),
                "an operation ID cannot be used with --dangerous-skip-verification"
            );
            return pending_creation::lookup_unverified(
                ctx,
                &config,
                self.activity_id,
                CreationKind::QuorumKey,
                self.verbose_proofs,
            )
            .await;
        }
        let operation_id = self
            .operation_id
            .context("missing local creation operation ID")?;
        pending_creation::verify_result(
            ctx,
            config,
            operation_id,
            self.activity_id,
            CreationKind::QuorumKey,
            self.verbose_proofs,
        )
        .await
    }
}
