//! Resume a creation using its original caller-owned receipt.

use crate::{
    client::build_client,
    config::turnkey::Config,
    outcome::Outcome,
    output::StdCtx,
    pending_creation::{CreationKind, Operation},
};
use anyhow::Result;
use clap::{Args as ClapArgs, builder::NonEmptyStringValueParser};
use uuid::Uuid;

#[derive(Debug, ClapArgs)]
pub struct ResumeArgs {
    /// Local saved creation UUID printed by the original command.
    operation_id: Uuid,
    /// Activity ID to look up, as printed by creation or shown in Turnkey.
    #[arg(long, value_parser = NonEmptyStringValueParser::new())]
    activity_id: String,
    /// Expand human output with the checks and public proof evidence.
    #[arg(long)]
    verbose_proofs: bool,
}

impl ResumeArgs {
    pub(crate) async fn run(
        self,
        ctx: &mut StdCtx,
        config: Config,
        kind: CreationKind,
    ) -> Result<Outcome> {
        let operation = Operation::open(self.operation_id)?;
        let saved = operation.load().await?;
        saved.check_kind(kind)?;
        let auth = build_client(&config).await?;
        saved
            .resume(ctx, &operation, auth, self.activity_id, self.verbose_proofs)
            .await
    }
}
