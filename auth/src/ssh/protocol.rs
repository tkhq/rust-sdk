//! SSH agent protocol helpers.
//!
//! Reference docs:
//! - <https://datatracker.ietf.org/doc/html/draft-ietf-sshm-ssh-agent>
//! - <https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.agent>

use anyhow::{anyhow, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

const SSH_ED25519_ALGORITHM: &str = "ssh-ed25519";
const CONNECTION_IO_TIMEOUT: Duration = Duration::from_millis(250);
const MAX_AGENT_FRAME_SIZE: usize = 1 << 20;

/// Generic SSH agent failure response message code.
pub const SSH_AGENT_FAILURE: u8 = 5;
/// SSH agent request code for listing available identities.
pub const SSH_AGENTC_REQUEST_IDENTITIES: u8 = 11;
/// SSH agent response code for returning identities.
pub const SSH_AGENT_IDENTITIES_ANSWER: u8 = 12;
/// SSH agent request code for signing data with a key.
pub const SSH_AGENTC_SIGN_REQUEST: u8 = 13;
/// SSH agent response code for returning a signature.
pub const SSH_AGENT_SIGN_RESPONSE: u8 = 14;

/// Parsed fields from an SSH agent sign request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSignRequest {
    /// Requested public key blob in OpenSSH wire format.
    pub public_key_blob: Vec<u8>,
    /// Raw payload bytes the agent should sign.
    pub data: Vec<u8>,
    /// OpenSSH signing flags from the request.
    pub flags: u32,
}

/// Encodes an SSH agent packet with the given message type and payload.
pub fn encode_agent_frame(message_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(4 + 1 + payload.len());
    frame.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
    frame.push(message_type);
    frame.extend_from_slice(payload);
    frame
}

/// Reads one SSH agent frame from a Unix stream with a bounded size and timeout.
pub async fn read_frame(stream: &mut UnixStream) -> std::io::Result<Option<Vec<u8>>> {
    let mut length_bytes = [0u8; 4];
    match read_exact_with_deadline(stream, &mut length_bytes).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(error),
    }

    let length = u32::from_be_bytes(length_bytes) as usize;
    if length > MAX_AGENT_FRAME_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "SSH agent frame exceeds maximum size",
        ));
    }

    let mut body = vec![0u8; length];
    read_exact_with_deadline(stream, &mut body).await?;

    let mut frame = length_bytes.to_vec();
    frame.extend_from_slice(&body);
    Ok(Some(frame))
}

/// Writes one SSH agent frame to a Unix stream with a timeout.
pub async fn write_frame(stream: &mut UnixStream, frame: &[u8]) -> std::io::Result<()> {
    match timeout(CONNECTION_IO_TIMEOUT, stream.write_all(frame)).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(std::io::Error::from(std::io::ErrorKind::TimedOut)),
    }
}

/// Encodes a `SSH_AGENT_IDENTITIES_ANSWER` packet for one Ed25519 key.
pub fn encode_request_identities_response(public_key_blob: &[u8]) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_be_bytes());
    payload = encode_string(public_key_blob, payload);
    payload = encode_string(&[], payload);
    Ok(encode_agent_frame(SSH_AGENT_IDENTITIES_ANSWER, &payload))
}

/// Parses the fields from a `SSH_AGENTC_SIGN_REQUEST` frame.
pub fn parse_sign_request_frame(frame: &[u8]) -> Result<AgentSignRequest> {
    let (message_type, payload) = parse_agent_frame(frame)?;
    if message_type != SSH_AGENTC_SIGN_REQUEST {
        return Err(anyhow!(
            "unsupported SSH agent message type: {message_type}"
        ));
    }

    let mut cursor = payload;
    let public_key_blob = read_ssh_bytes(&mut cursor)?;
    let data = read_ssh_bytes(&mut cursor)?;
    let flags = read_u32(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(anyhow!("unexpected trailing SSH agent sign request data"));
    }

    Ok(AgentSignRequest {
        public_key_blob,
        data,
        flags,
    })
}

/// Encodes a `SSH_AGENT_SIGN_RESPONSE` packet for a 64-byte Ed25519 signature.
pub fn encode_sign_response(signature: &[u8]) -> Result<Vec<u8>> {
    if signature.len() != 64 {
        return Err(anyhow!(
            "expected 64-byte ed25519 signature, got {} bytes",
            signature.len()
        ));
    }

    let mut signature_blob = Vec::new();
    signature_blob = encode_string(SSH_ED25519_ALGORITHM.as_bytes(), signature_blob);
    signature_blob = encode_string(signature, signature_blob);

    let mut payload = Vec::new();
    payload = encode_string(&signature_blob, payload);
    Ok(encode_agent_frame(SSH_AGENT_SIGN_RESPONSE, &payload))
}

async fn read_exact_with_deadline(stream: &mut UnixStream, buf: &mut [u8]) -> std::io::Result<()> {
    match timeout(CONNECTION_IO_TIMEOUT, stream.read_exact(buf)).await {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(error)) => Err(error),
        Err(_) => Err(std::io::Error::from(std::io::ErrorKind::TimedOut)),
    }
}

fn encode_string(bytes: &[u8], mut output: Vec<u8>) -> Vec<u8> {
    output.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    output.extend_from_slice(bytes);
    output
}

fn parse_agent_frame(frame: &[u8]) -> Result<(u8, &[u8])> {
    if frame.len() < 4 {
        return Err(anyhow!("truncated SSH agent frame length"));
    }

    let length = u32::from_be_bytes(frame[..4].try_into().expect("length slice should be 4"));
    let length = length as usize;
    if frame.len() < 4 + length {
        return Err(anyhow!("truncated SSH agent frame payload"));
    }
    if frame.len() != 4 + length {
        return Err(anyhow!("unexpected trailing bytes in SSH agent frame"));
    }
    if length == 0 {
        return Err(anyhow!("truncated SSH agent frame payload"));
    }

    Ok((frame[4], &frame[5..]))
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

fn read_u32(cursor: &mut &[u8]) -> Result<u32> {
    if cursor.len() < 4 {
        return Err(anyhow!("truncated SSH agent unsigned 32-bit integer"));
    }

    let value = u32::from_be_bytes(cursor[..4].try_into().expect("length slice should be 4"));
    *cursor = &cursor[4..];
    Ok(value)
}
