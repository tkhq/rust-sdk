use turnkey_auth::ssh::protocol;

fn encode_string(bytes: &[u8], output: &mut Vec<u8>) {
    output.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    output.extend_from_slice(bytes);
}

fn encode_frame(message_type: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(&((payload.len() + 1) as u32).to_be_bytes());
    frame.push(message_type);
    frame.extend_from_slice(payload);
    frame
}

fn ed25519_public_key_blob() -> Vec<u8> {
    let mut blob = Vec::new();
    encode_string(b"ssh-ed25519", &mut blob);
    encode_string(&[0x11; 32], &mut blob);
    blob
}

fn request_identities_expected_frame() -> Vec<u8> {
    let public_key_blob = ed25519_public_key_blob();
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u32.to_be_bytes());
    encode_string(&public_key_blob, &mut payload);
    encode_string(&[], &mut payload);
    encode_frame(protocol::SSH_AGENT_IDENTITIES_ANSWER, &payload)
}

fn sign_request_frame() -> Vec<u8> {
    let public_key_blob = ed25519_public_key_blob();
    let mut payload = Vec::new();
    encode_string(&public_key_blob, &mut payload);
    encode_string(b"ssh-agent-challenge", &mut payload);
    payload.extend_from_slice(&0x0000_0004u32.to_be_bytes());
    encode_frame(protocol::SSH_AGENTC_SIGN_REQUEST, &payload)
}

fn sign_response_expected_frame(signature: &[u8]) -> Vec<u8> {
    let mut signature_blob = Vec::new();
    encode_string(b"ssh-ed25519", &mut signature_blob);
    encode_string(signature, &mut signature_blob);

    let mut payload = Vec::new();
    encode_string(&signature_blob, &mut payload);
    encode_frame(protocol::SSH_AGENT_SIGN_RESPONSE, &payload)
}

#[test]
fn encode_request_identities_response_matches_expected_frame() {
    let public_key_blob = ed25519_public_key_blob();

    let frame = protocol::encode_request_identities_response(&public_key_blob)
        .expect("identity response should encode");

    assert_eq!(frame, request_identities_expected_frame());
}

#[test]
fn parse_sign_request_frame_extracts_key_blob_payload_and_flags() {
    let frame = sign_request_frame();

    let request = protocol::parse_sign_request_frame(&frame).expect("sign request should parse");

    assert_eq!(request.public_key_blob, ed25519_public_key_blob());
    assert_eq!(request.data, b"ssh-agent-challenge");
    assert_eq!(request.flags, 0x0000_0004);
}

#[test]
fn encode_sign_response_matches_expected_frame() {
    let signature = [0x22; 64];

    let frame = protocol::encode_sign_response(&signature).expect("sign response should encode");

    assert_eq!(frame, sign_response_expected_frame(&signature));
}

#[test]
fn parse_sign_request_frame_rejects_malformed_or_truncated_frames() {
    let frame = sign_request_frame();
    let truncated = &frame[..frame.len() - 1];

    let truncated_error = protocol::parse_sign_request_frame(truncated)
        .expect_err("truncated frame should be rejected");
    assert_eq!(
        truncated_error.to_string(),
        "truncated SSH agent frame payload"
    );

    let unsupported = encode_frame(99, &[]);
    let unsupported_error = protocol::parse_sign_request_frame(&unsupported)
        .expect_err("unsupported frame should be rejected");
    assert_eq!(
        unsupported_error.to_string(),
        "unsupported SSH agent message type: 99"
    );
}
