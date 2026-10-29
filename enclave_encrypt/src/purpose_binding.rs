//! Purpose bindings appended to the HPKE AAD to bind a ciphertext to the request fields it was
//! encrypted for.
//!
//! Every binding starts with the length-prefixed `EncryptedIngressPurpose` proto enum string name
//! of its flow, so a binding for one flow never equals a binding for another.

use crate::errors::EnclaveEncryptError;

/// `EncryptedIngressPurpose` enum string name for secrets import.
pub const IMPORT_SECRETS_PURPOSE: &str = "ENCRYPTED_INGRESS_PURPOSE_IMPORT_SECRETS";

/// Binding for one `ImportSecretParams`: its `name` and its `static_properties`, in order.
///
/// An absent `name` is encoded as the empty string; the signer rejects an empty `name`, so the
/// two never collide.
pub fn import_secrets(
    name: Option<&str>,
    static_properties: &[(&str, &str)],
) -> Result<Vec<u8>, EnclaveEncryptError> {
    let mut binding = Vec::new();
    push_field(&mut binding, IMPORT_SECRETS_PURPOSE.as_bytes())?;
    push_field(&mut binding, name.unwrap_or_default().as_bytes())?;
    push_len(&mut binding, static_properties.len())?;
    for (key, value) in static_properties {
        push_field(&mut binding, key.as_bytes())?;
        push_field(&mut binding, value.as_bytes())?;
    }
    Ok(binding)
}

fn push_field(binding: &mut Vec<u8>, field: &[u8]) -> Result<(), EnclaveEncryptError> {
    push_len(binding, field.len())?;
    binding.extend_from_slice(field);
    Ok(())
}

fn push_len(binding: &mut Vec<u8>, len: usize) -> Result<(), EnclaveEncryptError> {
    let len = u32::try_from(len).map_err(|_| EnclaveEncryptError::PurposeBindingFieldTooLong)?;
    binding.extend_from_slice(&len.to_be_bytes());
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use hpke::{Deserializable, Kem as KemTrait};

    use super::*;
    use crate::{Kem, TURNKEY_HPKE_INFO, decrypt};

    const PROPERTIES: &[(&str, &str)] =
        &[("kind", "credit-card"), ("dual-agent-consensus", "true")];
    const PLAINTEXT: &[u8] = br#"{"number":"4111111111111111","expiry":"12/30","cvv":"123"}"#;
    const RECEIVER_PRIVATE: &str =
        "1111111111111111111111111111111111111111111111111111111111111111";
    const RECEIVER_PUBLIC: &str = "040217e617f0b6443928278f96999e69a23a4f2c152bdf6d6cdf66e5b80282d4ed194a7debcb97712d2dda3ca85aa8765a56f45fc758599652f2897c65306e5794";
    const ENCAPPED_PUBLIC: &str = "04d65a93977caa3d1b081852ff57a79e465f1660577304baead505dd3a48589cf350185e895372df6221ea3a137557e473fddb6755f05bd507c3c533fce9c91285";
    const CIPHERTEXT: &str = "9afdba7451b5eacebb7e76a45b9860f968263279fb6b3ae3324a570ff48b0e8149dcc7af23a46ce01a319ca760a40f27da4cb738ccb0d5141ad3ce9738ce99d07e7bce10de3b58ee5454";
    const BINDING: &str = "00000028454e435259505445445f494e47524553535f505552504f53455f494d504f52545f5345435245545300000009636f72702d616d657800000002000000046b696e640000000b6372656469742d63617264000000146475616c2d6167656e742d636f6e73656e7375730000000474727565";

    fn open_vector(binding: &[u8]) -> Result<Vec<u8>, EnclaveEncryptError> {
        let private =
            <Kem as KemTrait>::PrivateKey::from_bytes(&hex::decode(RECEIVER_PRIVATE).unwrap())
                .unwrap();
        let public =
            <Kem as KemTrait>::PublicKey::from_bytes(&hex::decode(RECEIVER_PUBLIC).unwrap())
                .unwrap();
        let encapped =
            <Kem as KemTrait>::EncappedKey::from_bytes(&hex::decode(ENCAPPED_PUBLIC).unwrap())
                .unwrap();
        decrypt(
            &encapped,
            &private,
            &public,
            &hex::decode(CIPHERTEXT).unwrap(),
            TURNKEY_HPKE_INFO,
            binding,
        )
    }

    #[test]
    fn import_secrets_matches_spec_vector() {
        assert_eq!(
            hex::encode(import_secrets(Some("corp-amex"), PROPERTIES).unwrap()),
            BINDING
        );
    }

    #[test]
    fn spec_ciphertext_decrypts_only_with_its_binding() {
        assert_eq!(
            open_vector(&hex::decode(BINDING).unwrap()).unwrap(),
            PLAINTEXT
        );

        let reversed: Vec<_> = PROPERTIES.iter().rev().copied().collect();
        for binding in [
            Vec::new(),
            import_secrets(Some("corp-amex"), &PROPERTIES[..1]).unwrap(),
            import_secrets(Some("corp-visa"), PROPERTIES).unwrap(),
            import_secrets(None, PROPERTIES).unwrap(),
            import_secrets(Some("corp-amex"), &reversed).unwrap(),
        ] {
            assert_eq!(
                open_vector(&binding),
                Err(EnclaveEncryptError::FailedToDecrypt)
            );
        }
    }

    #[test]
    fn absent_name_encodes_as_empty() {
        assert_eq!(
            import_secrets(None, PROPERTIES).unwrap(),
            import_secrets(Some(""), PROPERTIES).unwrap()
        );
    }
}
