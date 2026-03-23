use std::io::ErrorKind;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context};
use clap::Args as ClapArgs;
use tokio::net::{UnixListener, UnixStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::task::JoinSet;

use crate::config::Config;
use crate::ssh;
use crate::ssh::protocol;
use crate::turnkey::TurnkeySigner;

#[derive(Debug, ClapArgs)]
#[command(about = "Run a foreground SSH agent over a Unix socket.", long_about = None)]
pub struct Args {
    /// Unix socket path to bind for SSH agent connections.
    #[arg(long, value_name = "path")]
    pub socket: PathBuf,
}

/// Runs the `auth ssh-agent` subcommand.
pub async fn run(args: Args) -> anyhow::Result<()> {
    remove_stale_socket(&args.socket).await?;
    let socket_path = args.socket.clone();

    let result = async {
        let signer = Arc::new(TurnkeySigner::new(Config::resolve().await?)?);
        let public_key = signer.get_public_key().await?;
        let public_key_blob = Arc::new(
            ssh::parse_public_key_line(&ssh::encode_public_key_line(&public_key, None)?)
                .context("failed to build SSH public key blob")?
                .public_key_blob,
        );

        let listener = UnixListener::bind(&args.socket).with_context(|| {
            format!(
                "failed to bind SSH agent socket at {}",
                args.socket.display()
            )
        })?;
        let mut interrupt = signal(SignalKind::interrupt()).context("failed to install SIGINT handler")?;
        let mut terminate = signal(SignalKind::terminate()).context("failed to install SIGTERM handler")?;
        let mut connections = JoinSet::new();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    let (stream, _) = accept_result.context("failed to accept SSH agent connection")?;
                    let signer = Arc::clone(&signer);
                    let public_key_blob = Arc::clone(&public_key_blob);
                    connections.spawn(async move {
                        handle_connection(stream, signer, public_key_blob).await
                    });
                }
                _ = interrupt.recv() => {
                    break;
                }
                _ = terminate.recv() => {
                    break;
                }
                Some(join_result) = connections.join_next() => {
                    match join_result {
                        Ok(Ok(())) => {}
                        Ok(Err(error)) if is_connection_error_kind(error.kind()) => {}
                        Ok(Err(error)) => {
                            return Err(error).context("failed to serve SSH agent connection");
                        }
                        Err(error) => {
                            return Err(anyhow!("ssh-agent connection task failed: {error}"));
                        }
                    }
                }
            }
        }

        connections.abort_all();
        while let Some(join_result) = connections.join_next().await {
            if let Err(error) = join_result {
                if error.is_panic() {
                    return Err(anyhow!("ssh-agent connection task panicked: {error}"));
                }
            }
        }

        Ok(())
    }
    .await;

    let _ = tokio::fs::remove_file(&socket_path).await;
    result
}

async fn handle_connection(
    mut stream: UnixStream,
    signer: Arc<TurnkeySigner>,
    configured_public_key_blob: Arc<Vec<u8>>,
) -> std::io::Result<()> {
    loop {
        let frame = match protocol::read_frame(&mut stream).await {
            Ok(Some(frame)) => frame,
            Ok(None) => return Ok(()),
            Err(error) if error.kind() == ErrorKind::InvalidData => {
                let failure = protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[]);
                let _ = protocol::write_frame(&mut stream, &failure).await;
                return Ok(());
            }
            Err(error) if is_connection_error_kind(error.kind()) => return Ok(()),
            Err(error) => return Err(error),
        };

        let response = match frame.get(4).copied() {
            Some(protocol::SSH_AGENTC_REQUEST_IDENTITIES) => {
                protocol::encode_request_identities_response(&configured_public_key_blob)
                    .unwrap_or_else(|_| {
                        protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
                    })
            }
            Some(protocol::SSH_AGENTC_SIGN_REQUEST) => {
                match protocol::parse_sign_request_frame(&frame) {
                    Ok(request) if request.public_key_blob == *configured_public_key_blob => {
                        match signer.sign_ssh_auth_payload(&request.data).await {
                            Ok(signature) => protocol::encode_sign_response(&signature)
                                .unwrap_or_else(|_| {
                                    protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
                                }),
                            Err(_) => {
                                protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
                            }
                        }
                    }
                    Ok(_) | Err(_) => {
                        protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[])
                    }
                }
            }
            Some(_) | None => protocol::encode_agent_frame(protocol::SSH_AGENT_FAILURE, &[]),
        };

        if let Err(error) = protocol::write_frame(&mut stream, &response).await {
            if is_connection_error_kind(error.kind()) {
                return Ok(());
            }
            return Err(error);
        }
    }
}

async fn remove_stale_socket(path: &Path) -> anyhow::Result<()> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_socket() => {
            tokio::fs::remove_file(path).await.with_context(|| {
                format!(
                    "failed to remove stale SSH agent socket at {}",
                    path.display()
                )
            })?;
        }
        Ok(_) => {
            return Err(anyhow!(
                "refusing to remove non-socket path at {}",
                path.display()
            ));
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    Ok(())
}
fn is_connection_error_kind(kind: ErrorKind) -> bool {
    matches!(
        kind,
        ErrorKind::WouldBlock
            | ErrorKind::TimedOut
            | ErrorKind::UnexpectedEof
            | ErrorKind::BrokenPipe
            | ErrorKind::ConnectionAborted
            | ErrorKind::ConnectionReset
            | ErrorKind::Interrupted
    )
}
