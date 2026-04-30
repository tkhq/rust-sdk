//! Generate quorum key command - generates and encrypts a quorum key from a given config.

use crate::config::quorum_key::QuorumKeyConfig;
use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use qos_p256::{P256Pair, P256Public};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::{self, File};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Debug, ClapArgs)]
#[command(about, long_about = None)]
pub struct Args {
    /// Path to the quorum key config file.
    pub config_file: PathBuf,

    /// Output file path for the generated quorum key metadata.
    #[arg(
        long,
        default_value = "quorum_key_metadata.json",
        env = "QUORUM_KEY_OUT"
    )]
    pub quorum_key_out: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QuorumKeyMetadata {
    quorum_key_public: String,
    threshold: u32,
    shares: Vec<EncryptedShareMetadata>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedShareMetadata {
    operator_public_key: String,
    share: String,
}

struct OperatorPublicKey {
    normalized: String,
    public: P256Public,
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
pub async fn run(args: Args) -> Result<()> {
    let config_file = File::open(&args.config_file).with_context(|| {
        format!(
            "failed to read quorum key config file: {}",
            args.config_file.display()
        )
    })?;

    let config: QuorumKeyConfig = serde_json::from_reader(config_file).with_context(|| {
        format!(
            "failed to parse quorum key config file: {}",
            args.config_file.display()
        )
    })?;
    config.validate()?;

    if args.quorum_key_out.exists() {
        anyhow::bail!(
            "quorum key metadata file already exists: {}",
            args.quorum_key_out.display()
        );
    }

    let operator_publics = parse_operator_public_keys(&config.operator_public_keys)?;
    let metadata = generate_and_encrypt_shares(&operator_publics, config.shares, config.threshold)?;
    let quorum_key_public = metadata.quorum_key_public.clone();

    let metadata_json =
        serde_json::to_vec_pretty(&metadata).context("failed to serialize quorum key metadata")?;
    fs::write(&args.quorum_key_out, &metadata_json)
        .with_context(|| format!("failed to write file: {}", args.quorum_key_out.display()))?;

    println!(
        "Quorum key metadata written to: {}",
        args.quorum_key_out.display()
    );

    println!("Quorum Public Key: {quorum_key_public}");
    println!("Threshold: {}", config.threshold);

    Ok(())
}

fn parse_operator_public_keys(operator_public_keys: &[String]) -> Result<Vec<OperatorPublicKey>> {
    let mut parsed = Vec::with_capacity(operator_public_keys.len());
    let mut seen = HashSet::new();

    for (index, key) in operator_public_keys.iter().enumerate() {
        let (normalized, public) = parse_operator_public_key(key)
            .with_context(|| format!("invalid operator public key at index {index}"))?;

        if !seen.insert(normalized.clone()) {
            anyhow::bail!("duplicate operator public key at index {index}");
        }

        parsed.push(OperatorPublicKey { normalized, public });
    }

    Ok(parsed)
}

fn parse_operator_public_key(operator_public_key: &str) -> Result<(String, P256Public)> {
    let bytes = hex::decode(operator_public_key).context("public key must be bare hex encoded")?;
    let public = P256Public::from_bytes(&bytes)
        .map_err(|e| anyhow::anyhow!("invalid QOS P-256 key: {e:?}"))?;

    Ok((hex::encode(public.to_bytes()), public))
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

    let encrypted_shares = operator_publics
        .iter()
        .zip(plaintext_shares.0.iter())
        .map(|(operator_public, share)| {
            operator_public
                .public
                .encrypt(share)
                .map_err(|e| anyhow::anyhow!("failed to encrypt quorum key share: {e:?}"))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(QuorumKeyMetadata {
        quorum_key_public,
        threshold,
        shares: operator_publics
            .iter()
            .zip(encrypted_shares.iter())
            .map(
                |(operator_public, encrypted_share)| EncryptedShareMetadata {
                    operator_public_key: operator_public.normalized.clone(),
                    share: hex::encode(encrypted_share),
                },
            )
            .collect(),
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

    fn err_string<T>(result: Result<T>) -> String {
        match result {
            Ok(_) => panic!("expected error"),
            Err(error) => error.to_string(),
        }
    }

    #[test]
    fn parse_operator_public_keys_rejects_malformed_key() {
        let err = err_string(parse_operator_public_keys(&["not-hex".to_string()]));

        assert!(err.contains("invalid operator public key at index 0"));
    }

    #[test]
    fn parse_operator_public_keys_canonicalizes_bare_uppercase_hex() {
        let key = operator_key();
        let parsed = parse_operator_public_keys(&[key.to_uppercase()]).unwrap();

        assert_eq!(parsed[0].normalized, key);
    }

    #[test]
    fn parse_operator_public_keys_rejects_duplicate_after_canonicalization() {
        let key = operator_key();
        let err = err_string(parse_operator_public_keys(&[
            key.clone(),
            key.to_uppercase(),
        ]));

        assert!(err.contains("duplicate operator public key at index 1"));
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
