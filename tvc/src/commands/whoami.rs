//! Whoami command for printing the active Turnkey identity.

use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use turnkey_client::generated::GetWhoamiRequest;

/// Print the current authenticated Turnkey identity.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {}

pub async fn run(_args: Args) -> Result<()> {
    let auth = crate::client::build_client().await?;
    let response = auth
        .client
        .get_whoami(GetWhoamiRequest {
            organization_id: auth.org_id.clone(),
        })
        .await
        .context("whoami request failed")?;

    println!(
        "{}",
        format_identity(
            &response.organization_name,
            &response.organization_id,
            &response.username,
            &response.user_id,
        )
    );

    Ok(())
}

fn format_identity(
    organization_name: &str,
    organization_id: &str,
    username: &str,
    user_id: &str,
) -> String {
    format!(
        "Organization: {organization_name} ({organization_id})\nUser: {username} ({user_id})"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_minimal_identity_output() {
        assert_eq!(
            format_identity("Example Org", "org_123", "alice", "user_456"),
            "Organization: Example Org (org_123)\nUser: alice (user_456)"
        );
    }
}
