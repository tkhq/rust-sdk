use crate::config::turnkey::{
    API_BASE_URL_DEV, API_BASE_URL_LOCAL, API_BASE_URL_PREPROD, API_BASE_URL_PROD,
};
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, SecondsFormat, Utc};
use p256::elliptic_curve::sec1::ToEncodedPoint;
use serde::Serialize;
use serde_json::value::RawValue;
use turnkey_enclave_encrypt::{P256Public, server::EnclaveEncryptServer};
use uuid::Uuid;
use zeroize::Zeroizing;

const LOCALHOST_PULL_SECRET_ENCRYPTION_PUBLIC_KEY: &str =
    "02acadf2025005142224d50ebeb93b91b8c57ef243a96cee44f50cf7051e67a3e2";
const DEV_PREPROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY: &str =
    "0276c9844e46525346dfbbf923a8c97194b76776ee93bb85e445bbbee26f4e4643";
const APP_PREPROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY: &str =
    "0396137d57f63a3d368c7978ef6d64b7cbe624df3060f3d465e67ebbc2ec4ad4e4";
const PROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY: &str =
    "0370ef18eaf7706c18d73423f8b247a530e0dc7abb261c513f479b3b9b4a906ffb";

/// Resolve the pull secret encryption key from the active org API URL
fn encryption_public_key_for_api_base_url(api_base_url: &str) -> Result<&'static str> {
    // `tvc` already requires login/active org, so api_base_url is the environment source of truth
    let key = match api_base_url {
        API_BASE_URL_LOCAL => LOCALHOST_PULL_SECRET_ENCRYPTION_PUBLIC_KEY,
        API_BASE_URL_DEV => DEV_PREPROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY,
        API_BASE_URL_PREPROD => APP_PREPROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY,
        API_BASE_URL_PROD => PROD_PULL_SECRET_ENCRYPTION_PUBLIC_KEY,
        _ => {
            anyhow::bail!(
                "unsupported API base URL for pivot pull secret encryption key inference: '{}'. \
                 expected one of: '{}', '{}', '{}', '{}'",
                api_base_url,
                API_BASE_URL_LOCAL,
                API_BASE_URL_DEV,
                API_BASE_URL_PREPROD,
                API_BASE_URL_PROD
            );
        }
    };

    Ok(key)
}

/// Ownership and acceptance deadline for an encrypted pull secret.
pub struct PullSecretMeta {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

/// Encrypt the pivot container pull secret with the appropriate public key for the environment.
pub fn encrypt_pivot_pull_secret(
    pull_secret: &RawValue,
    api_base_url: &str,
    metadata: PullSecretMeta,
) -> Result<String> {
    let PullSecretMeta {
        user_id,
        organization_id,
        expires_at,
    } = metadata;
    if expires_at <= Utc::now() {
        anyhow::bail!("pull secret expiry must be in the future");
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Envelope<'a> {
        version: u8,
        user_id: Uuid,
        organization_id: Uuid,
        expires_at: String,
        pull_secret: &'a RawValue,
    }
    // Covers the two UUIDs, version, timestamp, field names, and JSON punctuation.
    // Revisit if the envelope changes: growth can leave unwiped plaintext in old allocations.
    const ENVELOPE_METADATA_CAPACITY: usize = 512;

    let envelope = Envelope {
        version: 1,
        user_id,
        organization_id,
        expires_at: expires_at.to_rfc3339_opts(SecondsFormat::AutoSi, true),
        pull_secret,
    };
    let capacity = pull_secret.get().len() + ENVELOPE_METADATA_CAPACITY;
    let mut plaintext = Zeroizing::new(Vec::with_capacity(capacity));
    serde_json::to_writer(&mut *plaintext, &envelope).context("serialize pull secret envelope")?;

    // convert the hex public key to uncompressed bytes and then to P256Public
    let target_public_key_hex = encryption_public_key_for_api_base_url(api_base_url)?;
    let target_public_key_compressed = hex::decode(target_public_key_hex)
        .context("failed to decode pivot pull secret encryption public key")?;
    let target_public_key = p256::PublicKey::from_sec1_bytes(&target_public_key_compressed)
        .context("failed to parse pivot pull secret encryption public key")?;
    let target_public_key_uncompressed = target_public_key.to_encoded_point(false);
    let target_public_key = P256Public(
        target_public_key_uncompressed
            .as_bytes()
            .try_into()
            .map_err(|_| anyhow!("invalid uncompressed p256 public key length"))?,
    );

    // we are not interacting with enclaves nor performing auth here,
    // but reusing the encryption function to perform the same HPKE logic
    EnclaveEncryptServer::auth_encrypt(&target_public_key, &plaintext)
        .context("failed to encrypt pivot pull secret")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hpke::{Deserializable, Kem as KemTrait};
    use turnkey_enclave_encrypt::client::EnclaveEncryptClient;

    type Kem = hpke::kem::DhP256HkdfSha256;

    // Known test-only LOCALHOST CoordinatorP256EncryptionPrivateKey.
    // This is NOT secret — it is the local development key, safe to commit.
    const LOCALHOST_PRIVATE_KEY_HEX: &str =
        "6e7c11c15cda7ee898cd60f18637b409cc386e1eacef9dc881dd958ae5e6f5f7";

    #[test]
    fn encrypt_pivot_pull_secret_roundtrip() {
        let fake_pull_secret =
            r#"{"auths": {"ghcr.io": {"username": "user", "password": "pass"}}}"#;

        // Encrypt using the LOCALHOST public key
        let encrypted = encrypt_pivot_pull_secret(
            serde_json::from_str(fake_pull_secret).unwrap(),
            API_BASE_URL_LOCAL,
            PullSecretMeta {
                user_id: Uuid::from_u128(1),
                organization_id: Uuid::from_u128(2),
                expires_at: "2100-01-01T00:00:00Z".parse().unwrap(),
            },
        )
        .unwrap();

        // Build HPKE key pair from the known LOCALHOST private key
        let private_key_bytes = hex::decode(LOCALHOST_PRIVATE_KEY_HEX).unwrap();
        let target_private = <Kem as KemTrait>::PrivateKey::from_bytes(&private_key_bytes).unwrap();

        let public_key_compressed =
            hex::decode(LOCALHOST_PULL_SECRET_ENCRYPTION_PUBLIC_KEY).unwrap();
        let public_key_uncompressed = p256::PublicKey::from_sec1_bytes(&public_key_compressed)
            .unwrap()
            .to_encoded_point(false)
            .to_bytes()
            .to_vec();
        let target_public =
            <Kem as KemTrait>::PublicKey::from_bytes(&public_key_uncompressed).unwrap();

        // Construct a client for decryption (enclave_auth_key is unused by auth_decrypt)
        let dummy_signing_key = p256::ecdsa::SigningKey::from_slice(&private_key_bytes).unwrap();
        let mut client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            *dummy_signing_key.verifying_key(),
            target_public,
            target_private,
        );

        let decrypted = client.auth_decrypt(&encrypted).unwrap();
        let envelope: serde_json::Value = serde_json::from_slice(&decrypted).unwrap();
        assert_eq!(
            envelope,
            serde_json::json!({
                "version": 1,
                "userId": "00000000-0000-0000-0000-000000000001",
                "organizationId": "00000000-0000-0000-0000-000000000002",
                "expiresAt": "2100-01-01T00:00:00Z",
                "pullSecret": serde_json::from_str::<serde_json::Value>(fake_pull_secret).unwrap(),
            })
        );
    }

    #[test]
    fn rejects_expired_envelopes() {
        let error = encrypt_pivot_pull_secret(
            serde_json::from_str(r#"{"auths":{"ghcr.io":{"username":"user","password":"pass"}}}"#)
                .unwrap(),
            API_BASE_URL_LOCAL,
            PullSecretMeta {
                user_id: Uuid::from_u128(1),
                organization_id: Uuid::from_u128(2),
                expires_at: "2020-01-01T00:00:00Z".parse().unwrap(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "pull secret expiry must be in the future"
        );
    }

    #[test]
    fn decrypts_shared_javascript_fixture() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/pull-secret-envelope.json"))
                .unwrap();
        let private_key_bytes = hex::decode(LOCALHOST_PRIVATE_KEY_HEX).unwrap();
        let target_private = <Kem as KemTrait>::PrivateKey::from_bytes(&private_key_bytes).unwrap();
        let public = p256::PublicKey::from_sec1_bytes(
            &hex::decode(LOCALHOST_PULL_SECRET_ENCRYPTION_PUBLIC_KEY).unwrap(),
        )
        .unwrap();
        let target_public =
            <Kem as KemTrait>::PublicKey::from_bytes(public.to_encoded_point(false).as_bytes())
                .unwrap();
        let signing = p256::ecdsa::SigningKey::from_slice(&private_key_bytes).unwrap();
        let mut client = EnclaveEncryptClient::from_enclave_auth_key_and_target_key(
            *signing.verifying_key(),
            target_public,
            target_private,
        );
        let decrypted = client
            .auth_decrypt(fixture["ciphertext"].as_str().unwrap())
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&decrypted).unwrap(),
            fixture["envelope"]
        );
    }
}
