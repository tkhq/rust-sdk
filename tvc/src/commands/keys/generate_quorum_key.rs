//! Generate quorum key command - generates and encrypts a quorum key from a given config.

use crate::config::quorum_key::QuorumKeyConfig;
use crate::output::{Emitter, Report};
use crate::quorum_key_metadata::{
    decode_p256_public_key_hex, EncryptedShareMetadata, QuorumKeyMetadata,
};
use crate::util::read_json_file;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use qos_p256::{P256Pair, P256Public};
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use zeroize::Zeroize;

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

/// Summary of a generated quorum key, emitted to stdout. The encrypted shares
/// themselves live in the metadata file at `quorum_key_metadata_path`; this
/// report mirrors the human summary and does not duplicate them.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerateQuorumKeyReport {
    quorum_key_metadata_path: String,
    // Named to match the metadata file's `quorumKeyPublic` field for
    // cross-surface consistency, though the text label reads "Quorum Public Key".
    quorum_key_public: String,
    threshold: u32,
}

impl Report for GenerateQuorumKeyReport {
    fn render_text(&self, w: &mut dyn Write) -> io::Result<()> {
        writeln!(
            w,
            "Quorum key metadata written to: {}",
            self.quorum_key_metadata_path
        )?;
        writeln!(w, "Quorum Public Key: {}", self.quorum_key_public)?;
        writeln!(w, "Threshold: {}", self.threshold)
    }
}

struct PlaintextShares(Vec<Vec<u8>>);

impl Drop for PlaintextShares {
    fn drop(&mut self) {
        for share in &mut self.0 {
            share.as_mut_slice().zeroize();
        }
    }
}

/// Run the quorum key generation command.
pub async fn run(args: Args, out: &Emitter) -> Result<()> {
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

    let report = GenerateQuorumKeyReport {
        quorum_key_metadata_path: args.quorum_key_metadata_out.display().to_string(),
        quorum_key_public,
        threshold: config.threshold,
    };
    out.emit(&report)?;

    Ok(())
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
        quorum_pair.to_master_seed(),
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
                .encrypt(share)
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
    fn report_renders_text_and_camel_case_json() {
        let report = GenerateQuorumKeyReport {
            quorum_key_metadata_path: "out/metadata.json".to_string(),
            quorum_key_public: "04abcd".to_string(),
            threshold: 2,
        };

        let mut buf = Vec::new();
        report.render_text(&mut buf).unwrap();
        let text = String::from_utf8(buf).unwrap();
        assert_eq!(
            text,
            "Quorum key metadata written to: out/metadata.json\n\
             Quorum Public Key: 04abcd\n\
             Threshold: 2\n"
        );

        let value = serde_json::to_value(&report).unwrap();
        assert_eq!(value["quorumKeyMetadataPath"], "out/metadata.json");
        assert_eq!(value["quorumKeyPublic"], "04abcd");
        assert_eq!(value["threshold"], 2);
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

        let reconstructed = qos_crypto::shamir::shares_reconstruct(&decrypted_shares[..2])
            .unwrap()
            .try_into()
            .map(|seed: [u8; MASTER_SEED_LEN]| P256Pair::from_master_seed(&seed).unwrap())
            .unwrap();

        assert_eq!(
            hex::encode(reconstructed.public_key().to_bytes()),
            metadata.quorum_key_public
        );
    }
}
