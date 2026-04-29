//! Re-encrypt share command.

use crate::operator_key::load_operator_pair;
use crate::pair::Pair;
use crate::provisioning::ProvisionBundle;
use crate::util::write_file;
use anyhow::{anyhow, Context};
use clap::Args as ClapArgs;
use qos_core::protocol::services::boot::{Approval, ManifestEnvelope, QuorumMember};
use qos_core::protocol::QosHash;
use serde::Serialize;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// Re-encrypt a share with an ephemeral key.
#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the share file to re-encrypt.
    /// Assumed to be encrypted by the operator key.
    #[arg(long, env = "SHARE_PATH")]
    pub share_path: PathBuf,

    /// Provision bundle from a deployment.
    /// This should match the output format of `tvc deploy provisioning-details`.
    #[arg(long, env = "PROVISION_BUNDLE")]
    pub provision_bundle: PathBuf,

    /// Path to the file containing the master seed for the operator key.
    /// If not provided, uses the operator key from the logged-in org config.
    #[arg(
        long,
        help_heading = "Operator encryption key",
        value_name = "OPERATOR_PATH"
    )]
    pub operator_seed: Option<PathBuf>,

    /// Never use for sensitive applications! Skip attestation and manifest approval verification.
    #[arg(long)]
    pub dangerous_skip_verification: bool,

    /// Output path for the re-encrypted share.
    #[arg(long, env = "OUTPUT_PATH")]
    pub re_encrypted_out: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ReEncryptedShareOutput {
    re_encrypted_share: String,
    share_approval: Approval,
}

/// Run the re-encrypt-share command.
pub async fn run(args: Args) -> anyhow::Result<()> {
    if args.dangerous_skip_verification {
        eprintln!(
            "WARNING: Skipping verification! This is dangerous and should not be used for sensitive applications."
        );
    }

    let provision_bundle = read_provision_bundle(&args.provision_bundle).await?;
    let encrypted_share = tokio::fs::read(&args.share_path)
        .await
        .with_context(|| format!("failed to read share file: {}", args.share_path.display()))?;
    let operator_pair = load_operator_pair(args.operator_seed.as_deref()).await?;

    let output = build_re_encrypted_share_output(
        &encrypted_share,
        &provision_bundle,
        &operator_pair,
        args.dangerous_skip_verification,
    )
    .await?;

    write_output(args.re_encrypted_out.as_deref(), &output).await
}

async fn read_provision_bundle(path: &Path) -> anyhow::Result<ProvisionBundle> {
    let contents = tokio::fs::read(path)
        .await
        .with_context(|| format!("failed to read provision bundle: {}", path.display()))?;
    serde_json::from_slice(&contents)
        .with_context(|| format!("failed to parse provision bundle: {}", path.display()))
}

async fn build_re_encrypted_share_output(
    encrypted_share: &[u8],
    provision_bundle: &ProvisionBundle,
    operator_pair: &dyn Pair,
    dangerous_skip_verification: bool,
) -> anyhow::Result<ReEncryptedShareOutput> {
    let ephemeral_public_key =
        provision_bundle.ephemeral_public_key(dangerous_skip_verification)?;
    let member = find_share_set_member(
        provision_bundle.manifest_envelope(),
        &operator_pair.public_key(),
    )?;

    let re_encrypted_share = {
        let plaintext_share = Zeroizing::new(
            operator_pair
                .decrypt(encrypted_share.to_vec())
                .await
                .context("failed to decrypt share with operator key")?,
        );

        ephemeral_public_key
            .encrypt(plaintext_share.as_slice())
            .map_err(|err| anyhow!("failed to encrypt share to ephemeral key: {err:?}"))?
    };

    let signature = operator_pair
        .sign(
            provision_bundle
                .manifest_envelope()
                .manifest
                .qos_hash()
                .to_vec(),
        )
        .await
        .context("failed to sign share approval with operator key")?;
    let share_approval = Approval { signature, member };

    Ok(ReEncryptedShareOutput {
        re_encrypted_share: hex::encode(re_encrypted_share),
        share_approval,
    })
}

fn find_share_set_member(
    manifest_envelope: &ManifestEnvelope,
    operator_public_key: &[u8],
) -> anyhow::Result<QuorumMember> {
    manifest_envelope
        .manifest
        .share_set
        .members
        .iter()
        .find(|member| member.pub_key == operator_public_key)
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "operator ({}) not part of share set",
                hex::encode(operator_public_key)
            )
        })
}

async fn write_output(path: Option<&Path>, output: &ReEncryptedShareOutput) -> anyhow::Result<()> {
    let contents =
        serde_json::to_string_pretty(output).context("failed to serialize re-encrypted share")?;

    if let Some(path) = path {
        write_file(path, &contents).await?;
        eprintln!("Re-encrypted share written to: {}", path.display());
    } else {
        println!("{contents}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{build_re_encrypted_share_output, find_share_set_member, ReEncryptedShareOutput};
    use crate::pair::LocalPair;
    use crate::provisioning::ProvisionBundle;
    use qos_core::protocol::services::boot::{
        Approval, Manifest, ManifestEnvelope, ManifestSet, Namespace, NitroConfig, PatchSet,
        PivotConfig, QuorumMember, RestartPolicy, ShareSet,
    };
    use qos_core::protocol::QosHash;
    use qos_p256::{P256Pair, P256Public};
    use serde_json::json;

    fn sample_manifest_envelope(share_set_members: Vec<QuorumMember>) -> ManifestEnvelope {
        ManifestEnvelope {
            manifest: Manifest {
                namespace: Namespace {
                    name: "test-namespace".to_string(),
                    nonce: 7,
                    quorum_key: P256Pair::generate().unwrap().public_key().to_bytes(),
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
        }
    }

    fn bundle_with_ephemeral_key(
        ephemeral_key: &P256Public,
        share_set_members: Vec<QuorumMember>,
    ) -> ProvisionBundle {
        ProvisionBundle::new(
            "deploy-123".to_string(),
            b"not parsed when verification is skipped",
            sample_manifest_envelope(share_set_members),
            1_712_345_678_901,
            &ephemeral_key.to_bytes(),
        )
    }

    fn local_pair_from_pair(pair: &P256Pair) -> LocalPair {
        let seed_hex = String::from_utf8(pair.to_master_seed_hex()).unwrap();
        LocalPair::from_hex_seed(&seed_hex).unwrap()
    }

    #[test]
    fn output_serializes_expected_json_shape_with_hex() {
        let output = ReEncryptedShareOutput {
            re_encrypted_share: "010203".to_string(),
            share_approval: Approval {
                signature: vec![0xde, 0xad, 0xbe, 0xef],
                member: QuorumMember {
                    alias: "operator-1".to_string(),
                    pub_key: vec![0xaa, 0xbb, 0xcc],
                },
            },
        };

        let value = serde_json::to_value(&output).unwrap();

        assert_eq!(
            value,
            json!({
                "re_encrypted_share": "010203",
                "share_approval": {
                    "signature": "deadbeef",
                    "member": {
                        "alias": "operator-1",
                        "pubKey": "aabbcc",
                    },
                },
            })
        );
    }

    #[test]
    fn finds_operator_share_set_member_by_public_key() {
        let operator_pair = P256Pair::generate().unwrap();
        let member = QuorumMember {
            alias: "operator-1".to_string(),
            pub_key: operator_pair.public_key().to_bytes(),
        };
        let manifest_envelope = sample_manifest_envelope(vec![member.clone()]);

        let found =
            find_share_set_member(&manifest_envelope, &operator_pair.public_key().to_bytes())
                .unwrap();

        assert_eq!(found, member);
    }

    #[test]
    fn rejects_operator_missing_from_share_set() {
        let operator_pair = P256Pair::generate().unwrap();
        let other_pair = P256Pair::generate().unwrap();
        let manifest_envelope = sample_manifest_envelope(vec![QuorumMember {
            alias: "operator-1".to_string(),
            pub_key: other_pair.public_key().to_bytes(),
        }]);

        assert!(
            find_share_set_member(&manifest_envelope, &operator_pair.public_key().to_bytes())
                .is_err()
        );
    }

    #[tokio::test]
    async fn re_encrypt_round_trip_and_generates_verifiable_approval() {
        let operator_pair = P256Pair::generate().unwrap();
        let operator_member = QuorumMember {
            alias: "operator-1".to_string(),
            pub_key: operator_pair.public_key().to_bytes(),
        };
        let operator_local_pair = local_pair_from_pair(&operator_pair);
        let ephemeral_pair = P256Pair::generate().unwrap();
        let bundle =
            bundle_with_ephemeral_key(&ephemeral_pair.public_key(), vec![operator_member.clone()]);
        let plaintext_share = b"arbitrary test share bytes";
        let encrypted_share = operator_pair.public_key().encrypt(plaintext_share).unwrap();

        let output =
            build_re_encrypted_share_output(&encrypted_share, &bundle, &operator_local_pair, true)
                .await
                .unwrap();

        let re_encrypted_share = hex::decode(&output.re_encrypted_share).unwrap();
        let decrypted_share = ephemeral_pair.decrypt(&re_encrypted_share).unwrap();
        assert_eq!(decrypted_share, plaintext_share);
        assert_eq!(output.share_approval.member, operator_member);

        let approval_public_key =
            P256Public::from_bytes(&output.share_approval.member.pub_key).unwrap();
        approval_public_key
            .verify(
                &bundle.manifest_envelope().manifest.qos_hash(),
                &output.share_approval.signature,
            )
            .unwrap();
    }
}
