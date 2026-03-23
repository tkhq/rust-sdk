use turnkey_auth::cli::Cli;
use turnkey_auth::commands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let raw_args = std::env::args().skip(1).collect::<Vec<_>>();

    // Git invokes the configured SSH signer directly with `ssh-keygen`-style
    // flags like `-Y sign -n git ...`, not as `auth git-sign ...`.
    if raw_args.first().is_some_and(|arg| arg == "-Y") {
        return commands::git_sign::run(commands::git_sign::Args {
            ssh_keygen_args: raw_args,
        })
        .await;
    }

    Cli::run().await
}
