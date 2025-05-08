//! Basic validation for fields of the Nitro Secure Module Attestation Document.

use std::collections::BTreeMap;

use aws_nitro_enclaves_nsm_api::api::Digest;

use super::{AttestError, ByteBuf};

const MIN_PCR_COUNT: usize = 1;
const MAX_PRC_COUNT: usize = 32;
const MAX_PCR_INDEX: usize = 32;
const VALID_PCR_LENS: [usize; 3] = [32, 48, 64];

const MIN_PUB_KEY_LEN: usize = 1;
const MIN_CERT_CHAIN_LEN: usize = 1;
const MAX_PUB_KEY_LEN: usize = 1024;

const MIN_CERT_LEN: usize = 1;
const MAX_CERT_LEN: usize = 1024;

/// Mandatory field
pub(super) fn module_id(id: &str) -> Result<(), AttestError> {
    if id.is_empty() {
        Err(AttestError::InvalidModuleId)
    } else {
        Ok(())
    }
}
/// Mandatory field
pub(super) fn pcrs(pcrs: &BTreeMap<usize, ByteBuf>) -> Result<(), AttestError> {
    let is_valid_pcr_count = pcrs.len() >= MIN_PCR_COUNT && pcrs.len() <= MAX_PRC_COUNT;

    let is_valid_index_and_len = pcrs.iter().all(|(idx, pcr)| {
        let is_valid_idx = *idx <= MAX_PCR_INDEX;
        let is_valid_pcr_len = VALID_PCR_LENS.contains(&pcr.len());
        is_valid_idx && is_valid_pcr_len
    });

    if !is_valid_index_and_len || !is_valid_pcr_count {
        Err(AttestError::InvalidPcr)
    } else {
        Ok(())
    }
}
/// Mandatory field
pub(super) fn cabundle(cabundle: &[ByteBuf]) -> Result<(), AttestError> {
    let is_valid_len = cabundle.len() >= MIN_CERT_CHAIN_LEN;
    let is_valid_entries = cabundle
        .iter()
        .all(|cert| cert.len() >= MIN_CERT_LEN && cert.len() <= MAX_CERT_LEN);

    if !is_valid_len || !is_valid_entries {
        Err(AttestError::InvalidCABundle)
    } else {
        Ok(())
    }
}
/// Mandatory field
pub(super) fn digest(d: Digest) -> Result<(), AttestError> {
    if d == Digest::SHA384 {
        Ok(())
    } else {
        Err(AttestError::InvalidDigest)
    }
}
/// Mandatory field
pub(super) fn timestamp(t: u64) -> Result<(), AttestError> {
    if t == 0 {
        Err(AttestError::InvalidTimeStamp)
    } else {
        Ok(())
    }
}
/// Optional field
pub(super) fn public_key(pub_key: &Option<ByteBuf>) -> Result<(), AttestError> {
    if let Some(key) = pub_key {
        (key.len() >= MIN_PUB_KEY_LEN && key.len() <= MAX_PUB_KEY_LEN)
            .then_some(())
            .ok_or(AttestError::InvalidPubKey)?;
    }

    Ok(())
}
/// Optional field
pub(super) fn user_data(data: &Option<ByteBuf>) -> Result<(), AttestError> {
    bytes_512(data)
}
/// Optional field
pub(super) fn nonce(n: &Option<ByteBuf>) -> Result<(), AttestError> {
    bytes_512(n)
}

fn bytes_512(val: &Option<ByteBuf>) -> Result<(), AttestError> {
    if let Some(val) = val {
        if val.len() > 512 {
            return Err(AttestError::InvalidBytes);
        }
    }

    Ok(())
}
