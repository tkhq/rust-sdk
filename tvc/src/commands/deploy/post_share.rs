//! Deploy post-share command.

use crate::commands::keys::re_encrypt_share::ReEncryptedShareOutput;
use crate::output::{Ctx, Message};
use crate::util::read_json_file;
use anyhow::Context;
use clap::Args as ClapArgs;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use turnkey_client::generated::{PostTvcQuorumKeyShareIntent, QuorumKeyShareApprovalBundle};

/// Post a re-encrypted quorum key share for a deployment.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the re-encrypted share output from `tvc keys re-encrypt-share`.
    #[arg(long, value_name = "PATH", env = "TVC_RE_ENCRYPTED_SHARE")]
    pub re_encrypted_share: PathBuf,

    /// Turnkey share set operator ID (UUID) for the operator posting this share.
    #[arg(long, env = "TVC_SHARE_OPERATOR_ID")]
    pub share_operator_id: String,
}

/// Run the deploy post-share command.
pub async fn run<W: Write>(ctx: &mut Ctx<W>, args: Args) -> anyhow::Result<()> {
    let re_encrypted_share: ReEncryptedShareOutput =
        read_json_file(&args.re_encrypted_share, "re-encrypted share output").await?;
    let intent =
        build_post_tvc_quorum_key_share_intent(&re_encrypted_share, &args.share_operator_id);

    let auth = crate::client::build_client().await?;
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time before unix epoch")?
        .as_millis();

    let result = auth
        .client
        .post_tvc_quorum_key_share(auth.org_id, timestamp_ms, intent)
        .await
        .context("failed to post quorum key share")?;

    ctx.shell().emit(&QuorumKeySharePosted {
        provisioning_share_id: result.result.provisioning_share_id,
    })?;

    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QuorumKeySharePosted {
    provisioning_share_id: String,
}

impl Message for QuorumKeySharePosted {
    fn reason(&self) -> &'static str {
        "quorum-key-share-posted"
    }

    fn human_message(&self) -> String {
        format!("Provisioning Share ID: {}", self.provisioning_share_id)
    }
}

fn build_post_tvc_quorum_key_share_intent(
    re_encrypted_share: &ReEncryptedShareOutput,
    share_operator_id: &str,
) -> PostTvcQuorumKeyShareIntent {
    PostTvcQuorumKeyShareIntent {
        deployment_id: re_encrypted_share.deployment_id.clone(),
        ephemeral_public_key_hex: re_encrypted_share.ephemeral_public_key_hex.clone(),
        share_approval_bundle: Some(QuorumKeyShareApprovalBundle {
            operator_id: share_operator_id.to_string(),
            re_encrypted_share_hex: re_encrypted_share.re_encrypted_share.clone(),
            signature: hex::encode(&re_encrypted_share.share_approval.signature),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::build_post_tvc_quorum_key_share_intent;
    use crate::commands::keys::re_encrypt_share::ReEncryptedShareOutput;
    use qos_core::protocol::services::boot::{Approval, QuorumMember};
    use serde_json::json;
    use turnkey_client::generated::QuorumKeyShareApprovalBundle;

    fn sample_re_encrypted_share(deployment_id: &str) -> ReEncryptedShareOutput {
        ReEncryptedShareOutput {
            deployment_id: deployment_id.to_string(),
            ephemeral_public_key_hex: "04abcd".to_string(),
            re_encrypted_share: "010203".to_string(),
            share_approval: Approval {
                signature: vec![0xde, 0xad, 0xbe, 0xef],
                member: QuorumMember {
                    alias: "operator-1".to_string(),
                    pub_key: vec![0xaa, 0xbb, 0xcc],
                },
            },
        }
    }

    #[test]
    fn builds_expected_intent_shape() {
        let output = sample_re_encrypted_share("deploy-from-file");
        let intent = build_post_tvc_quorum_key_share_intent(&output, "share-operator-id");

        assert_eq!(intent.deployment_id, "deploy-from-file");
        assert_eq!(intent.ephemeral_public_key_hex, "04abcd");
        assert_eq!(
            intent.share_approval_bundle,
            Some(QuorumKeyShareApprovalBundle {
                operator_id: "share-operator-id".to_string(),
                re_encrypted_share_hex: "010203".to_string(),
                signature: "deadbeef".to_string(),
            })
        );
    }

    #[test]
    fn rejects_re_encrypt_share_json_without_deployment_id() {
        let err = serde_json::from_value::<ReEncryptedShareOutput>(json!({
            "ephemeralPublicKeyHex": "04abcd",
            "reEncryptedShare": "010203",
            "shareApproval": {
                "signature": "deadbeef",
                "member": {
                    "alias": "operator-1",
                    "pubKey": "aabbcc",
                },
            },
        }))
        .unwrap_err();

        assert!(err.to_string().contains("deploymentId"));
    }
}
