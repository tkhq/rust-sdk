//! Resume a creation using its original caller-owned receipt.

use crate::{
    client::build_client, config::turnkey::Config, outcome::Outcome, output::StdCtx,
    pending_creation::Operation,
};
use anyhow::Result;
use clap::Args as ClapArgs;
use uuid::Uuid;

#[derive(Debug, ClapArgs)]
pub struct ResumeArgs {
    /// Local saved creation UUID printed by the original command.
    operation_id: Uuid,
    /// Candidate activity label when the original response was lost.
    #[arg(long)]
    activity_id: Option<String>,
    /// Expand human output with the checks and public proof evidence.
    #[arg(long)]
    verbose_proofs: bool,
}

pub async fn resume(ctx: &mut StdCtx, args: ResumeArgs, config: Config) -> Result<Outcome> {
    let operation = Operation::open(args.operation_id)?;
    let saved = operation.load()?;
    let auth = build_client(&config).await?;
    let saved = saved.for_client(&auth)?;
    saved
        .resume(
            ctx,
            &auth,
            &operation,
            args.activity_id,
            args.verbose_proofs,
        )
        .await
}
