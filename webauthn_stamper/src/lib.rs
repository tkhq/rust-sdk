use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use passkey_authenticator::{Authenticator, UserCheck, UserValidationMethod};
use passkey_client::{Client, DefaultClientData, WebauthnError};
use passkey_types::{crypto::sha256, ctap2::*, webauthn::*};
use turnkey_client::generated::Attestation;
use turnkey_client::generated::external::webauthn::v1::WebAuthnStamp;
use turnkey_client::generated::immutable::webauthn::v1::AuthenticatorTransport;
use url::Url;

/// A user-verification stub that always approves
pub struct AutoUserValidation;
#[async_trait]
impl UserValidationMethod for AutoUserValidation {
    type PasskeyItem = passkey_types::Passkey;

    async fn check_user<'a>(
        &self,
        _hint: Option<&'a Self::PasskeyItem>,
        presence: bool,
        verification: bool,
    ) -> Result<UserCheck, Ctap2Error> {
        Ok(UserCheck {
            presence,
            verification,
        })
    }

    fn is_verification_enabled(&self) -> Option<bool> {
        Some(true)
    }
    fn is_presence_enabled(&self) -> bool {
        true
    }
}

pub struct WebAuthnStamper {
    pub aaguid: Aaguid,
    pub origin_url: Url,
    pub store: Option<passkey_types::Passkey>,
}

#[derive(Debug)]
pub struct Passkey {
    pub encoded_challenge: String,
    pub attestation: Attestation,
}

impl WebAuthnStamper {
    pub fn new(origin_url: Url) -> Self {
        let aaguid = Aaguid::new_empty();

        WebAuthnStamper {
            aaguid,
            origin_url,
            store: None,
        }
    }

    pub async fn create_passkey(
        &mut self,
        challenge_bytes: &[u8],
        credential_params: PublicKeyCredentialParameters,
        user_entity: PublicKeyCredentialUserEntity,
    ) -> Result<Passkey, WebauthnError> {
        let authenticator = self.get_authenticator();
        let mut client = Client::new(authenticator);

        let credential_request = CredentialCreationOptions {
            public_key: PublicKeyCredentialCreationOptions {
                rp: PublicKeyCredentialRpEntity {
                    id: None,
                    name: user_entity.clone().name,
                },
                user: user_entity,
                challenge: challenge_bytes.into(),
                pub_key_cred_params: vec![credential_params],
                timeout: None,
                exclude_credentials: None,
                authenticator_selection: None,
                hints: None,
                attestation: AttestationConveyancePreference::None,
                attestation_formats: None,
                extensions: None,
            },
        };

        let credential = client
            .register(&self.origin_url, credential_request, DefaultClientData)
            .await?;

        self.store = client.authenticator().store().clone();

        Ok(Passkey {
            encoded_challenge: BASE64_URL_SAFE_NO_PAD.encode(challenge_bytes),
            attestation: Attestation {
                credential_id: BASE64_URL_SAFE_NO_PAD.encode(credential.raw_id.as_slice()),
                client_data_json: BASE64_URL_SAFE_NO_PAD
                    .encode(credential.response.client_data_json.as_slice()),
                attestation_object: BASE64_URL_SAFE_NO_PAD
                    .encode(credential.response.attestation_object.as_slice()),
                transports: credential
                    .response
                    .transports
                    .unwrap_or_default()
                    .iter()
                    .map(|t| match t {
                        passkey_types::webauthn::AuthenticatorTransport::Usb => {
                            AuthenticatorTransport::Usb
                        }
                        passkey_types::webauthn::AuthenticatorTransport::Nfc => {
                            AuthenticatorTransport::Nfc
                        }
                        passkey_types::webauthn::AuthenticatorTransport::Ble => {
                            AuthenticatorTransport::Ble
                        }
                        passkey_types::webauthn::AuthenticatorTransport::Hybrid => {
                            AuthenticatorTransport::Hybrid
                        }
                        passkey_types::webauthn::AuthenticatorTransport::Internal => {
                            AuthenticatorTransport::Internal
                        }
                    })
                    .collect(),
            },
        })
    }

    pub async fn stamp(&self, body: &[u8]) -> Result<WebAuthnStamp, WebauthnError> {
        let request_digest = sha256(body);
        let hex_digest = hex::encode(request_digest);
        let challenge_bytes = hex_digest.as_bytes().to_vec();

        let authenticator = self.get_authenticator();
        let mut client = Client::new(authenticator);
        let credential_request = CredentialRequestOptions {
            public_key: PublicKeyCredentialRequestOptions {
                challenge: challenge_bytes.into(),
                timeout: None,
                rp_id: None,
                allow_credentials: None,
                user_verification: UserVerificationRequirement::default(),
                hints: None,
                attestation: AttestationConveyancePreference::None,
                attestation_formats: None,
                extensions: None,
            },
        };

        let auth_cred = client
            .authenticate(&self.origin_url, credential_request, DefaultClientData)
            .await?;

        let stamp = WebAuthnStamp {
            credential_id: BASE64_URL_SAFE_NO_PAD.encode(auth_cred.raw_id.as_slice()),
            client_data_json: BASE64_URL_SAFE_NO_PAD
                .encode(auth_cred.response.client_data_json.as_slice()),
            authenticator_data: BASE64_URL_SAFE_NO_PAD
                .encode(auth_cred.response.authenticator_data.as_slice()),
            signature: BASE64_URL_SAFE_NO_PAD.encode(auth_cred.response.signature.as_slice()),
        };

        Ok(stamp)
    }

    fn get_authenticator(
        &self,
    ) -> Authenticator<Option<passkey_types::Passkey>, AutoUserValidation> {
        Authenticator::new(self.aaguid, self.store.clone(), AutoUserValidation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coset::iana;
    use passkey_types::crypto::sha256;

    #[tokio::test]
    async fn test_create_passkey() {
        let origin_url = Url::parse("https://example.com").unwrap();
        let mut stamper = WebAuthnStamper::new(origin_url);
        let challenge = sha256(b"test challenge").to_vec();
        let credential_params = PublicKeyCredentialParameters {
            ty: PublicKeyCredentialType::PublicKey,
            alg: iana::Algorithm::ES256,
        };
        let user_entity = PublicKeyCredentialUserEntity {
            id: b"user123".to_vec().into(),
            name: "user123".into(),
            display_name: "User 123".into(),
        };

        let created_cred = stamper
            .create_passkey(&challenge, credential_params, user_entity)
            .await
            .unwrap();

        assert!(created_cred.attestation.credential_id.len() > 0);
    }

    #[tokio::test]
    async fn test_authenticate() {
        let origin_url = Url::parse("https://example.com").unwrap();
        let mut stamper = WebAuthnStamper::new(origin_url);
        let challenge = sha256(b"test challenge").to_vec();
        let credential_params = PublicKeyCredentialParameters {
            ty: PublicKeyCredentialType::PublicKey,
            alg: iana::Algorithm::ES256,
        };
        let user_entity = PublicKeyCredentialUserEntity {
            id: b"user123".to_vec().into(),
            name: "user123".into(),
            display_name: "User 123".into(),
        };
        let created_cred = stamper
            .create_passkey(&challenge, credential_params, user_entity)
            .await
            .unwrap();
        let authenticated_cred = stamper.stamp(&challenge).await.unwrap();

        assert!(authenticated_cred.credential_id.len() > 0);
        assert_eq!(
            authenticated_cred.credential_id,
            created_cred.attestation.credential_id
        );
    }
}
