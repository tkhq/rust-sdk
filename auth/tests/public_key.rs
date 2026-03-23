use turnkey_auth::ssh::encode_public_key_line;

#[test]
fn encode_public_key_line_matches_openssh_format() {
    let public_key = [0x66; 32];

    let line = encode_public_key_line(&public_key, None).expect("public key encoding should work");

    assert_eq!(
        line,
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm"
    );
}

#[test]
fn encode_public_key_line_appends_comment_when_present() {
    let public_key = [0x66; 32];

    let line = encode_public_key_line(&public_key, Some("turnkey-auth"))
        .expect("public key encoding should work");

    assert_eq!(
        line,
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIGZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZm turnkey-auth"
    );
}
