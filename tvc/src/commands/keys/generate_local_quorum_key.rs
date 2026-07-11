//! Generate local quorum key command - generates and encrypts a quorum key from a given config.

use crate::config::quorum_key::QuorumKeyConfig;
use crate::outcome::Outcome;
use crate::output::{Message, StdCtx};
use crate::quorum_key_metadata::{
    EncryptedShareMetadata, QuorumKeyMetadata, decode_p256_public_key_hex,
};
use crate::util::read_json_file;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use qos_p256::{P256Pair, P256Public};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use zeroize::Zeroizing;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the quorum key config file.
    #[arg(short = 'c', long, value_name = "PATH", env = "TVC_QUORUM_KEY_CONFIG")]
    pub config_file: PathBuf,

    /// Output file path for the generated quorum key metadata.
    #[arg(
        long,
        value_name = "PATH",
        default_value = "quorum_key_metadata.json",
        env = "TVC_QUORUM_KEY_METADATA_OUT"
    )]
    pub quorum_key_metadata_out: PathBuf,
}

struct OperatorPublicKey {
    normalized: String,
    public: P256Public,
}

// `Zeroizing` zeroes each share's memory on drop.
struct PlaintextShares(Vec<Zeroizing<Vec<u8>>>);

/// Run the quorum key generation command.
pub async fn run(_ctx: &mut StdCtx, args: Args) -> Result<Outcome> {
    let config: QuorumKeyConfig =
        read_json_file(&args.config_file, "quorum key config file").await?;
    config.validate()?;

    if args.quorum_key_metadata_out.exists() {
        anyhow::bail!(
            "quorum key metadata file already exists: {}",
            args.quorum_key_metadata_out.display()
        );
    }

    let operator_publics = parse_operator_public_keys(&config.operator_public_keys)?;
    let metadata = generate_and_encrypt_shares(&operator_publics, config.shares, config.threshold)?;
    let quorum_key_public = metadata.quorum_key_public.clone();

    let metadata_json =
        serde_json::to_vec_pretty(&metadata).context("failed to serialize quorum key metadata")?;
    fs::write(&args.quorum_key_metadata_out, &metadata_json).with_context(|| {
        format!(
            "failed to write file: {}",
            args.quorum_key_metadata_out.display()
        )
    })?;

    Ok(Outcome::KeysGenerateQuorumKey(QuorumKeyGenerated {
        quorum_key_public,
        threshold: config.threshold,
        metadata_path: args.quorum_key_metadata_out.display().to_string(),
    }))
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumKeyGenerated {
    quorum_key_public: String,
    threshold: u32,
    metadata_path: String,
}

impl Message for QuorumKeyGenerated {
    fn reason(&self) -> &'static str {
        "quorum-key-generated"
    }

    fn human_message(&self) -> String {
        format!(
            r#"Quorum key metadata written to: {}
Quorum Public Key: {}
Threshold: {}"#,
            self.metadata_path, self.quorum_key_public, self.threshold
        )
    }
}

fn parse_operator_public_keys(operator_public_keys: &[String]) -> Result<Vec<OperatorPublicKey>> {
    operator_public_keys
        .iter()
        .enumerate()
        .map(|(index, key)| {
            let public = decode_p256_public_key_hex(key)
                .with_context(|| format!("invalid operator public key at index {index}"))?;
            Ok(OperatorPublicKey {
                normalized: hex::encode(public.to_bytes()),
                public,
            })
        })
        .collect()
}

fn generate_and_encrypt_shares(
    operator_publics: &[OperatorPublicKey],
    shares: u32,
    threshold: u32,
) -> Result<QuorumKeyMetadata> {
    if operator_publics.len() != shares as usize {
        anyhow::bail!(
            "operator public key count ({}) must equal shares ({shares})",
            operator_publics.len()
        );
    }

    let quorum_pair = P256Pair::generate()
        .map_err(|e| anyhow::anyhow!("failed to generate quorum key: {e:?}"))?;
    let quorum_key_public = hex::encode(quorum_pair.public_key().to_bytes());

    let plaintext_shares = qos_crypto::shamir::shares_generate(
        quorum_pair.to_master_seed().as_slice(),
        shares as usize,
        threshold as usize,
    )
    .map_err(|e| anyhow::anyhow!("failed to generate quorum key shares: {e:?}"))?;
    let plaintext_shares = PlaintextShares(plaintext_shares);

    let shares = operator_publics
        .iter()
        .zip(plaintext_shares.0.iter())
        .map(|(operator_public, share)| {
            let encrypted_share = operator_public
                .public
                .encrypt(share.as_slice())
                .map_err(|e| anyhow::anyhow!("failed to encrypt quorum key share: {e:?}"))?;

            Ok(EncryptedShareMetadata {
                operator_public_key: operator_public.normalized.clone(),
                share: hex::encode(encrypted_share),
            })
        })
        .collect::<Result<Vec<EncryptedShareMetadata>>>()?;

    Ok(QuorumKeyMetadata {
        quorum_key_public,
        threshold,
        shares,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use qos_p256::MASTER_SEED_LEN;

    fn operator_pair() -> P256Pair {
        P256Pair::generate().unwrap()
    }

    fn operator_key() -> String {
        hex::encode(operator_pair().public_key().to_bytes())
    }

    #[test]
    fn parse_operator_public_keys_rejects_malformed_key() {
        assert!(parse_operator_public_keys(&["not-hex".to_string()]).is_err());
    }

    #[test]
    fn parse_operator_public_keys_canonicalizes_bare_uppercase_hex() {
        let key = operator_key();
        let parsed = parse_operator_public_keys(&[key.to_uppercase()]).unwrap();

        assert_eq!(parsed[0].normalized, key);
    }

    #[test]
    fn generate_and_encrypt_shares_rejects_key_count_mismatch() {
        let operator_publics = parse_operator_public_keys(&[operator_key()]).unwrap();
        let err = generate_and_encrypt_shares(&operator_publics, 2, 2)
            .unwrap_err()
            .to_string();

        assert!(err.contains("operator public key count (1) must equal shares (2)"));
    }

    #[test]
    fn generate_and_encrypt_shares_roundtrips_threshold_shares() {
        let operator_pairs = (0..3).map(|_| operator_pair()).collect::<Vec<_>>();
        let operator_public_keys = operator_pairs
            .iter()
            .map(|pair| hex::encode(pair.public_key().to_bytes()))
            .collect::<Vec<_>>();
        let operator_publics = parse_operator_public_keys(&operator_public_keys).unwrap();

        let metadata = generate_and_encrypt_shares(&operator_publics, 3, 2).unwrap();
        let decrypted_shares = metadata
            .shares
            .iter()
            .zip(operator_pairs.iter())
            .map(|(share, pair)| {
                let encrypted_share = hex::decode(&share.share).unwrap();
                pair.decrypt(&encrypted_share).unwrap()
            })
            .collect::<Vec<_>>();

        let reconstructed_seed =
            qos_crypto::shamir::shares_reconstruct(&decrypted_shares[..2]).unwrap();
        let seed: [u8; MASTER_SEED_LEN] = reconstructed_seed.as_slice().try_into().unwrap();
        let reconstructed = P256Pair::from_master_seed(&Zeroizing::new(seed)).unwrap();

        assert_eq!(
            hex::encode(reconstructed.public_key().to_bytes()),
            metadata.quorum_key_public
        );
    }
}
