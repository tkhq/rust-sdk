//! CLI commands.
//!
//! Each command module contains an `Args` struct deriving `clap::Args` and
//! either a `run(ctx, args) -> anyhow::Result<Outcome>` function (legacy) or
//! an implementation of [`Run`] (the target shape; `deploy approve` is the
//! pilot).

use crate::{config::turnkey::Config, outcome, output::StdCtx};

pub mod app;
pub mod app_status;
pub mod confirmation;
pub mod deploy;
pub mod display;
pub mod keys;
pub mod login;
pub mod operator;
pub mod version;
pub mod yubikey;

/// A command: consumes its parsed `Args`, returns its typed terminal outcome.
///
/// The associated `Outcome` is a command-local type; `Into<outcome::Outcome>`
/// is the whole contract, since serialization (including the `reason` tag) and
/// human rendering both live on the top-level [`outcome::Outcome`].
pub trait Run {
    type Outcome: Into<outcome::Outcome>;

    fn run(
        self,
        ctx: &mut StdCtx,
        config: Config,
    ) -> impl Future<Output = anyhow::Result<Self::Outcome>> + Send;
}
