use anyhow::{anyhow, Context, Result};
use base64::Engine;
use sha2::{Digest, Sha512};

/// SSH agent protocol constants, framing helpers, and reference links.
pub mod protocol;

/// Git signing invocation parsing.
pub mod git;

const SSH_ED25519_ALGORITHM: &str = "ssh-ed25519";
const SSHSIG_PREAMBLE: &[u8] = b"SSHSIG";
/// Default hash algorithm name encoded into SSHSIG payloads.
pub const DEFAULT_HASH_ALGORITHM: &str = "sha512";

/// Encodes a raw Ed25519 public key as an OpenSSH `authorized_keys` line.
pub fn encode_public_key_line(public_key: &[u8], comment: Option<&str>) -> Result<String> {
    if public_key.len() != 32 {
        return Err(anyhow!(
            "expected 32-byte ed25519 public key, got {} bytes",
            public_key.len()
        ));
    }

    let blob = encode_string(SSH_ED25519_ALGORITHM.as_bytes(), Vec::new());
    let blob = encode_string(public_key, blob);
    let encoded = base64::engine::general_purpose::STANDARD.encode(blob);

    Ok(match comment {
        Some(comment) if !comment.is_empty() => {
            format!("{SSH_ED25519_ALGORITHM} {encoded} {comment}")
        }
        _ => format!("{SSH_ED25519_ALGORITHM} {encoded}"),
    })
}

fn encode_string(bytes: &[u8], mut output: Vec<u8>) -> Vec<u8> {
    output.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    output.extend_from_slice(bytes);
    output
}

/// Parsed components of an OpenSSH Ed25519 public key line.
pub struct ParsedPublicKey {
    /// Raw 32-byte Ed25519 public key.
    pub public_key: Vec<u8>,
    /// Full OpenSSH public key blob including algorithm tag.
    pub public_key_blob: Vec<u8>,
}

/// Parses an OpenSSH Ed25519 public key line.
pub fn parse_public_key_line(line: &str) -> Result<ParsedPublicKey> {
    let trimmed = line.trim();
    let mut parts = trimmed.split_whitespace();
    let algorithm = parts
        .next()
        .ok_or_else(|| anyhow!("missing SSH key algorithm"))?;
    if algorithm != SSH_ED25519_ALGORITHM {
        return Err(anyhow!("unsupported SSH public key algorithm: {algorithm}"));
    }

    let encoded = parts
        .next()
        .ok_or_else(|| anyhow!("missing SSH public key body"))?;

    let blob = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("failed to decode SSH public key body")?;

    let (blob_algorithm, public_key) = parse_public_key_blob(&blob)?;
    if blob_algorithm != SSH_ED25519_ALGORITHM {
        return Err(anyhow!(
            "SSH public key blob algorithm mismatch: {blob_algorithm}"
        ));
    }
    if public_key.len() != 32 {
        return Err(anyhow!(
            "expected 32-byte SSH Ed25519 public key, got {} bytes",
            public_key.len()
        ));
    }

    Ok(ParsedPublicKey {
        public_key,
        public_key_blob: blob,
    })
}

/// Builds the `SSHSIG` signed payload for the given namespace and message.
pub fn build_signed_data(namespace: &str, payload: &[u8]) -> Vec<u8> {
    let digest = Sha512::digest(payload);
    let mut output = Vec::new();
    output.extend_from_slice(SSHSIG_PREAMBLE);
    output = encode_string(namespace.as_bytes(), output);
    output = encode_string(&[], output);
    output = encode_string(DEFAULT_HASH_ALGORITHM.as_bytes(), output);
    output = encode_string(&digest, output);
    output
}

/// Encodes a detached SSH signature in OpenSSH armored format.
pub fn encode_armored_signature(
    public_key_blob: &[u8],
    namespace: &str,
    hash_algorithm: &str,
    signature: &[u8],
) -> Result<String> {
    if signature.len() != 64 {
        return Err(anyhow!(
            "expected 64-byte ed25519 signature, got {} bytes",
            signature.len()
        ));
    }

    let signature_blob = encode_string(
        signature,
        encode_string(SSH_ED25519_ALGORITHM.as_bytes(), Vec::new()),
    );

    let mut blob = Vec::new();
    blob.extend_from_slice(SSHSIG_PREAMBLE);
    blob.extend_from_slice(&1u32.to_be_bytes());
    blob = encode_string(public_key_blob, blob);
    blob = encode_string(namespace.as_bytes(), blob);
    blob = encode_string(&[], blob);
    blob = encode_string(hash_algorithm.as_bytes(), blob);
    blob = encode_string(&signature_blob, blob);

    let base64 = base64::engine::general_purpose::STANDARD.encode(blob);
    let wrapped = wrap_base64(&base64, 76);

    Ok(format!(
        "-----BEGIN SSH SIGNATURE-----\n{wrapped}\n-----END SSH SIGNATURE-----\n"
    ))
}

fn wrap_base64(input: &str, width: usize) -> String {
    let mut lines = Vec::new();
    let mut start = 0;
    while start < input.len() {
        let end = usize::min(start + width, input.len());
        lines.push(input[start..end].to_string());
        start = end;
    }
    lines.join("\n")
}

fn parse_public_key_blob(blob: &[u8]) -> Result<(String, Vec<u8>)> {
    let mut cursor = blob;
    let algorithm = read_ssh_bytes(&mut cursor)?;
    let key = read_ssh_bytes(&mut cursor)?;
    if !cursor.is_empty() {
        return Err(anyhow!("unexpected trailing data in SSH public key blob"));
    }

    let algorithm = String::from_utf8(algorithm).context("SSH algorithm was not valid utf-8")?;
    Ok((algorithm, key))
}

fn read_ssh_bytes(cursor: &mut &[u8]) -> Result<Vec<u8>> {
    if cursor.len() < 4 {
        return Err(anyhow!("truncated SSH string length"));
    }

    let length = u32::from_be_bytes(cursor[..4].try_into().expect("length slice should be 4"));
    *cursor = &cursor[4..];

    let length = length as usize;
    if cursor.len() < length {
        return Err(anyhow!("truncated SSH string body"));
    }

    let value = cursor[..length].to_vec();
    *cursor = &cursor[length..];
    Ok(value)
}
