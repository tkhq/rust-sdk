#![doc = include_str!("../README.md")]

use std::num::ParseIntError;

use aws_nitro_enclaves_cose::{
    crypto::{Hash, MessageDigest, SignatureAlgorithm, SigningPublicKey},
    error::CoseError,
    CoseSign1,
};
use aws_nitro_enclaves_nsm_api::api::AttestationDoc;
use p256::ecdsa::{signature::Verifier, VerifyingKey as P256VerifyingKey};
use p384::{
    ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey},
    PublicKey,
};
use serde_bytes::ByteBuf;
use sha2::Digest;
use turnkey_api_key_stamper::Stamp;
use turnkey_client::generated::external::data::v1::{AppProof, BootProof, SignatureScheme};
use turnkey_client::generated::services::coordinator::public::v1::{
    BootProofResponse, GetBootProofRequest,
};
use turnkey_client::{TurnkeyClient, TurnkeyClientError};

mod error;
mod syntactic_validation;
mod types;

pub use error::AppProofError;
pub use error::AttestError;
pub use error::BootProofError;
pub use error::VerifyError;

/// Signing algorithms we expect the certificates to use. Any other
/// algorithms will be considered invalid. NOTE: this list was deduced just
/// by trial and error and thus its unclear if it should include more types.
static AWS_NITRO_CERT_SIG_ALG: &[&webpki::SignatureAlgorithm] = &[&webpki::ECDSA_P384_SHA384];

/// AWS Nitro root CA certificate.
///
/// The root certificate can be downloaded from
/// <https://aws-nitro-enclaves.amazonaws.com/AWS_NitroEnclaves_Root-G1.zip>,
/// and it can be verified using the following SHA256 checksum:
/// `8cf60e2b2efca96c6a9e71e851d00c1b6991cc09eadbe64a6a1d1b1eb9faff7c`.
/// This official hash checksum is over the AWS-provided zip file.
/// For context and additional verification details, see
/// <https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html/>.
///
/// The `aws_root_cert.pem` contents hash as follows via SHA256:
/// `6eb9688305e4bbca67f44b59c29a0661ae930f09b5945b5d1d9ae01125c8d6c0`.
pub const AWS_ROOT_CERT_PEM: &[u8] = std::include_bytes!("../static/aws_root.pem");

pub const EXPECTED_EPHEMERAL_PUBLIC_KEY_LENGTH: usize = 130;

/// Extract a DER encoded certificate from bytes representing a PEM encoded
/// certificate.
pub fn cert_from_pem(pem: &[u8]) -> Result<Vec<u8>, AttestError> {
    let (_, doc) = x509_cert::der::Document::from_pem(&String::from_utf8_lossy(pem))
        .map_err(|_| AttestError::PemDecodingError)?;
    Ok(doc.to_vec())
}

/// Verify that `attestation_doc` matches the specified parameters.
///
/// To learn more about the attestation document fields see:
/// <https://github.com/aws/aws-nitro-enclaves-nsm-api/blob/main/docs/attestation_process.md#22-attestation-document-specification/>.
///
/// # Arguments
///
/// * `attestation_doc` - the attestation document to verify.
/// * `user_data` - expected value of the `user_data` field.
/// * `pcr0` - expected value of PCR index 0.
/// * `pcr1` - expected value of PCR index 1.
/// * `pcr2` - expected value of PCR index 3.
///
/// # Panics
///
/// Panics if any part of verification fails.
pub fn verify_attestation_doc_against_user_input(
    attestation_doc: &AttestationDoc,
    user_data: &[u8],
    pcr0: &[u8],
    pcr1: &[u8],
    pcr2: &[u8],
    pcr3: &[u8],
) -> Result<(), AttestError> {
    if user_data
        != attestation_doc
            .user_data
            .as_ref()
            .ok_or(AttestError::MissingUserData)?
            .to_vec()
    {
        return Err(AttestError::DifferentUserData);
    }

    // nonce is none
    if attestation_doc.nonce.is_some() {
        return Err(AttestError::UnexpectedAttestationDocNonce);
    }

    if pcr0
        != attestation_doc
            .pcrs
            .get(&0)
            .ok_or(AttestError::MissingPcr0)?
            .clone()
            .into_vec()
    {
        return Err(AttestError::DifferentPcr0);
    }

    // pcr1 matches
    if pcr1
        != attestation_doc
            .pcrs
            .get(&1)
            .ok_or(AttestError::MissingPcr1)?
            .clone()
            .into_vec()
    {
        return Err(AttestError::DifferentPcr1);
    }

    // pcr2 matches
    if pcr2
        != attestation_doc
            .pcrs
            .get(&2)
            .ok_or(AttestError::MissingPcr2)?
            .clone()
            .into_vec()
    {
        return Err(AttestError::DifferentPcr2);
    }

    // pcr3 matches
    if pcr3
        != attestation_doc
            .pcrs
            .get(&3)
            .ok_or(AttestError::MissingPcr3)?
            .clone()
            .into_vec()
    {
        return Err(AttestError::DifferentPcr3);
    }

    Ok(())
}

/// Extract the DER encoded `AttestationDoc` from the nitro secure module
/// (nsm) provided COSE Sign1 structure.
///
/// WARNING: This will not perform any validation of the attestation doc and
/// should not be used directly in production; instead use
/// [`attestation_doc_from_der`].
///
/// # Arguments
///
/// * `cose_sign1_der` - the DER encoded COSE Sign1 structure containing the
///   attestation document payload.
pub fn unsafe_attestation_doc_from_der(
    cose_sign1_der: &[u8],
) -> Result<AttestationDoc, AttestError> {
    let cose_sign1 = CoseSign1::from_bytes(cose_sign1_der)
        .map_err(|_| AttestError::InvalidCOSESign1Structure)?;

    let raw_attestation_doc = cose_sign1
        .get_payload::<Sha2>(None)
        .map_err(|_| AttestError::InvalidCOSESign1Structure)?;

    AttestationDoc::from_binary(&raw_attestation_doc[..]).map_err(Into::into)
}

/// Extract the DER encoded `AttestationDoc` from the nitro secure module
/// (nsm) provided COSE Sign1 structure. This function will verify the the
/// root certificate authority via the CA bundle and verify that the end
/// entity certificate signed the COSE Sign1 structure.
///
/// # Arguments
///
/// * `cose_sign1_der` - the DER encoded COSE Sign1 structure containing the
///   attestation document payload.
/// * `root_cert` - the DER encoded root certificate. This should be a hardcoded
///   root certificate from amazon and its authenticity should be validated out
///   of band.
/// * `validation_time` - a moment in time that the certificates should be
///   valid. This is measured in seconds since the unix epoch. Most likely this
///   will be the current time.
pub fn parse_and_verify_der_attestation(
    cose_sign1_der: &[u8],
    root_cert: &[u8],
    validation_time: u64, // seconds since unix epoch
) -> Result<AttestationDoc, AttestError> {
    let attestation_doc = unsafe_attestation_doc_from_der(cose_sign1_der)?;
    let cose_sign1 = CoseSign1::from_bytes(cose_sign1_der)
        .map_err(|_| AttestError::InvalidCOSESign1Structure)?;

    syntactic_validation::module_id(&attestation_doc.module_id)?;
    syntactic_validation::digest(attestation_doc.digest)?;
    syntactic_validation::pcrs(&attestation_doc.pcrs)?;
    syntactic_validation::cabundle(&attestation_doc.cabundle)?;
    syntactic_validation::timestamp(attestation_doc.timestamp)?;
    syntactic_validation::public_key(&attestation_doc.public_key)?;
    syntactic_validation::user_data(&attestation_doc.user_data)?;
    syntactic_validation::nonce(&attestation_doc.nonce)?;

    verify_certificate_chain(
        &attestation_doc.cabundle,
        root_cert,
        &attestation_doc.certificate,
        validation_time,
    )?;
    verify_cose_sign1_sig(&attestation_doc.certificate, &cose_sign1)?;
    Ok(attestation_doc)
}

/// Verify the certificate chain against the root & end entity certificates.
fn verify_certificate_chain(
    cabundle: &[ByteBuf],
    root_cert: &[u8],
    end_entity_certificate: &[u8],
    validation_time: u64,
) -> Result<(), AttestError> {
    // Bundle starts with root certificate - we want to replace the root
    // with our hardcoded known certificate, so we remove the root
    // (first element). Ordering is: root cert .. intermediate certs ..
    // end entity cert.
    let intermediate_certs: Vec<_> = cabundle[1..].iter().map(|x| x.as_slice()).collect();

    let anchor = vec![webpki::TrustAnchor::try_from_cert_der(root_cert)?];
    let anchors = webpki::TlsServerTrustAnchors(&anchor);

    let cert = webpki::EndEntityCert::try_from(end_entity_certificate)?;
    cert.verify_is_valid_tls_server_cert(
        AWS_NITRO_CERT_SIG_ALG,
        &anchors,
        &intermediate_certs,
        webpki::Time::from_seconds_since_unix_epoch(validation_time),
    )
    .map_err(AttestError::InvalidCertChain)?;

    Ok(())
}

// Check that cose sign1 structure is signed with the key in the end
// entity certificate.
fn verify_cose_sign1_sig(
    end_entity_certificate: &[u8],
    cose_sign1: &CoseSign1,
) -> Result<(), AttestError> {
    use x509_cert::der::Decode;

    let ee_cert = x509_cert::certificate::Certificate::from_der(end_entity_certificate)
        .map_err(|_| AttestError::FailedToParseCert)?;

    // Expect v3
    if ee_cert.tbs_certificate.version != x509_cert::certificate::Version::V3 {
        return Err(AttestError::InvalidEndEntityCert);
    }

    let pub_key = ee_cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key;
    let key =
        PublicKey::from_sec1_bytes(pub_key).map_err(|_| AttestError::FailedDecodeKeyFromCert)?;
    let key_wrapped = P384PubKey(key);

    // Verify the signature against the extracted public key
    let is_valid_sig = cose_sign1
        .verify_signature::<Sha2>(&key_wrapped)
        .map_err(|_| AttestError::InvalidCOSESign1Signature)?;
    if is_valid_sig {
        Ok(())
    } else {
        Err(AttestError::InvalidCOSESign1Signature)
    }
}

struct P384PubKey(p384::PublicKey);
impl SigningPublicKey for P384PubKey {
    fn get_parameters(&self) -> Result<(SignatureAlgorithm, MessageDigest), CoseError> {
        Ok((SignatureAlgorithm::ES384, MessageDigest::Sha384))
    }

    fn verify(&self, digest: &[u8], signature: &[u8]) -> Result<bool, CoseError> {
        let signature_wrapped =
            Signature::try_from(signature).map_err(|e| CoseError::SignatureError(Box::new(e)))?;

        let verifier = VerifyingKey::from(self.0);
        verifier
            .verify_prehash(digest, &signature_wrapped)
            .map(|_| true)
            .map_err(|e| CoseError::SignatureError(Box::new(e)))
    }
}

struct Sha2;
impl Hash for Sha2 {
    fn hash(digest: MessageDigest, data: &[u8]) -> Result<Vec<u8>, CoseError> {
        use sha2::Digest as _;
        match digest {
            MessageDigest::Sha256 => Ok(sha2::Sha256::digest(data).to_vec()),
            MessageDigest::Sha384 => Ok(sha2::Sha384::digest(data).to_vec()),
            MessageDigest::Sha512 => Ok(sha2::Sha512::digest(data).to_vec()),
        }
    }
}

/// Parses and verifies an AWS nitro attestation, provided as a base64 encoded string (defaults to using current time for validation)
pub fn parse_and_verify_aws_nitro_attestation<S: AsRef<str>>(
    encoded_attestation: S,
    validation_time: Option<std::time::SystemTime>,
) -> Result<AttestationDoc, AttestError> {
    // Decode the base64 string
    let decoded_bytes = base64::decode(encoded_attestation.as_ref())
        .map_err(|e| AttestError::Base64DecodingError(e.to_string()))?;

    let trusted_root_certificate = cert_from_pem(AWS_ROOT_CERT_PEM).unwrap();

    let duration = validation_time
        .unwrap_or_else(std::time::SystemTime::now)
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    parse_and_verify_der_attestation(
        decoded_bytes.as_slice(),
        trusted_root_certificate.as_slice(),
        duration,
    )
}

pub fn get_app_proof_time(app_proof: &AppProof) -> Result<std::time::SystemTime, AppProofError> {
    let parsed: serde_json::Value = serde_json::from_str(&app_proof.proof_payload)
        .map_err(|e| AppProofError::InvalidProofPayload(e.to_string()))?;

    let timestamp_ms_str = parsed["timestampMs"]
        .as_str()
        .ok_or(AppProofError::MissingTimestamp)?;

    let timestamp_ms: u64 = timestamp_ms_str
        .parse()
        .map_err(|e: ParseIntError| AppProofError::InvalidTimestamp(e.to_string()))?;

    let duration = std::time::Duration::from_millis(timestamp_ms);
    Ok(std::time::UNIX_EPOCH + duration)
}

pub fn get_boot_proof_time(
    boot_proof: &BootProof,
) -> Result<std::time::SystemTime, BootProofError> {
    let timestamp = boot_proof
        .created_at
        .as_ref()
        .ok_or(BootProofError::MissingTimestamp)?;

    let seconds: u64 = timestamp
        .seconds
        .parse()
        .map_err(|e: ParseIntError| BootProofError::InvalidTimestamp(e.to_string()))?;

    let nanos: u32 = timestamp
        .nanos
        .parse()
        .map_err(|e: ParseIntError| BootProofError::InvalidTimestamp(e.to_string()))?;

    let duration = std::time::Duration::new(seconds, nanos);
    Ok(std::time::UNIX_EPOCH + duration)
}

/// Verify the app proof's signature
pub fn verify_app_proof_signature(app_proof: &AppProof) -> Result<(), AppProofError> {
    if app_proof.scheme != SignatureScheme::EphemeralKeyP256 {
        return Err(AppProofError::InvalidSignatureScheme);
    }

    let pub_key_bytes = hex::decode(&app_proof.public_key)
        .map_err(|e| AppProofError::InvalidPublicKey(e.to_string()))?;
    if pub_key_bytes.len() != EXPECTED_EPHEMERAL_PUBLIC_KEY_LENGTH {
        return Err(AppProofError::InvalidPublicKey(format!(
            "pub_key_bytes expected len {}, got: {}",
            EXPECTED_EPHEMERAL_PUBLIC_KEY_LENGTH,
            pub_key_bytes.len()
        )));
    }
    let signing_pub_key_bytes = &pub_key_bytes[65..];
    let verifying_key = P256VerifyingKey::from_sec1_bytes(signing_pub_key_bytes)
        .map_err(|e| AppProofError::InvalidSigningPublicKeyBytes(e.to_string()))?;

    let signature_bytes = hex::decode(&app_proof.signature)
        .map_err(|e| AppProofError::InvalidSignature(e.to_string()))?;
    let signature = p256::ecdsa::Signature::from_slice(&signature_bytes)
        .map_err(|e| AppProofError::InvalidSignature(e.to_string()))?;

    let msg = app_proof.proof_payload.as_bytes();
    verifying_key
        .verify(msg, &signature)
        .map_err(|e| AppProofError::FailedSignatureVerification(e.to_string()))?;

    Ok(())
}

/// Verify the app proof boot proof pair.
///  - Verify app proof signature
///  - Verify the boot proof
///    - Attestation doc was signed by AWS
///    - Attestation doc's `user_data` is the hash of the qos manifest
///  - Verify the app proof / boot proof connection - that the ephemeral keys match
///
/// To learn more about verifying app proofs and boot proofs, see:
/// <https://whitepaper.turnkey.com/foundations/>.
///
/// # Arguments
///
/// * `app_proof` - an app proof from an activity being verified.
/// * `boot_proof` - the boot proof for the given app proof.
pub fn verify(app_proof: &AppProof, boot_proof: &BootProof) -> Result<(), VerifyError> {
    // 1. Verify App Proof
    verify_app_proof_signature(app_proof)
        .map_err(|e| VerifyError::InvalidAppProof(e.to_string()))?;

    // 2. Verify Boot Proof
    // Attestation docs technically expire after 3 hours, so an app proof generated 3+ hours after an enclave
    // boots up will fail verification due to certificate expiration. This is okay because enclaves are immutable;
    // even if the cert is technically invalid, the code contained within it cannot change. To prevent the cert
    // expiration failure, we pass in the time from the boot proof as validation time
    let boot_proof_time = get_boot_proof_time(boot_proof)
        .map_err(|e| VerifyError::InvalidBootProof(e.to_string()))?;
    let attestation_doc = parse_and_verify_aws_nitro_attestation(
        &boot_proof.aws_attestation_doc_b64,
        Some(boot_proof_time),
    )
    .map_err(|e| VerifyError::InvalidAttestation(e.to_string()))?;

    // Verify manifest digest
    let decoded_boot_proof_manifest = base64::decode(&boot_proof.qos_manifest_b64)
        .map_err(|e| VerifyError::InvalidBootProof(e.to_string()))?;
    let manifest_digest = sha2::Sha256::digest(&decoded_boot_proof_manifest);
    let user_data = attestation_doc
        .user_data
        .expect("validated attestation doc should have user_data");
    if manifest_digest.as_slice() != user_data.as_slice() {
        return Err(VerifyError::DifferentManifest(format!("attestation_doc's user_data doesn't match the hash of the manifest. attestation_doc.user_data: {user_data:?}, manifest_digest: {manifest_digest:?}")));
    }

    // 3. Verify that all the ephemeral public keys match: app proof, boot proof structure, actual attestation doc
    let attestation_pub_key_bytes = attestation_doc
        .public_key
        .expect("validated attestation doc should have public_key");
    let attestation_pub_key = hex::encode(attestation_pub_key_bytes);
    if !(app_proof.public_key == attestation_pub_key
        && attestation_pub_key == boot_proof.ephemeral_public_key_hex)
    {
        return Err(VerifyError::DifferentEphemeralKey(format!("Ephemeral pub keys from app proof: {}, boot proof structure {}, and attestation doc {} should all match", app_proof.public_key, boot_proof.ephemeral_public_key_hex, attestation_pub_key)));
    }

    Ok(())
}

/// Wrapper around TurnkeyClient::get_boot_proof that fetches the boot proof for the given app proof
pub async fn get_boot_proof_for_app_proof<S: Stamp>(
    client: &TurnkeyClient<S>,
    organization_id: String,
    app_proof: &AppProof,
) -> Result<BootProofResponse, TurnkeyClientError> {
    let request = GetBootProofRequest {
        organization_id,
        ephemeral_key: app_proof.public_key.clone(),
    };
    client.get_boot_proof(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use turnkey_client::generated::external::data::v1::Timestamp;

    fn test_app_proof1() -> AppProof {
        AppProof {
            scheme: SignatureScheme::EphemeralKeyP256,
            public_key: "043144c138d522996833f9a5352b98da2ba669f1ab3f712b243a1993c89c7d682f2163c4c63c6625c483cd5e8714e9a2039e1ec05cebadf469a4adb7e721d16e19047b2846a313c6f3c543c23ae9779118c2411b28bd5d76b25572903c360240c841f4c4a3af4a297d770e9d8bceb5e7e5be31a8fe16e571b452e3de99ffcebcb2e1".to_string(),
            proof_payload: "{\"type\":\"APP_PROOF_TYPE_ADDRESS_DERIVATION\",\"timestampMs\":\"1758058763571\",\"addressDerivationProof\":{\"organizationId\":\"3a2de333-972b-45bf-8a7a-83c4167b81e2\",\"walletId\":\"ad8a06ec-c2f4-55f2-ac43-65991efbfdd1\",\"derivationPath\":\"m/44\'/60\'/0\'/0/0\",\"address\":\"0xb84b4730Cd81Bc82Ee6B1dE6c343Ebd7928138DC\"}}".to_string(),
            signature: "712bd9b77bd19430c90dbd8fde1d5b1d1d6f18ec7e9534b7a68a42544c76ba75a7cf1fc39583df3bed3ca394b721c6ab1fef8db7794e5ef6a97b1ca45d7a5247".to_string(),
        }
    }

    fn test_app_proof2() -> AppProof {
        AppProof {
            scheme: SignatureScheme::EphemeralKeyP256,
            public_key: "04ab52e052b94bc7f3badb1e0f3500d11062c9f44307bf76091b5c5f48cace9d35caba26028238e391f735c0a74996f98f69c593916aa5fcd6b3c7a56d33b5a31d0435d0354ae36981a010965d13fe6539827be12a17fcbb0fa35586c5b7732c4e8dc6a0efa81e762633f271d05d5440b2d4e97df48912d214c895bc42c7c61d076e".to_string(),
            proof_payload: "{\"type\":\"APP_PROOF_TYPE_ADDRESS_DERIVATION\",\"timestampMs\":\"1758058843846\",\"addressDerivationProof\":{\"organizationId\":\"81f458bd-f42f-4b8f-b487-9a232afc9767\",\"walletId\":\"568df378-e609-50e2-877d-86e6a7b78f66\",\"derivationPath\":\"m/44\'/60\'/0\'/0/0\",\"address\":\"0x5A1541859016c22265F5CB39fFd1beF6c48D7ea7\"}}".to_string(),
            signature: "5a3e694a551c6688c1ce5f43de2d0e99b4a77ff03f01b3dddf66ae6c44cb4777583c49f25354511d1d68defd710553be30eab95c2b3c8f7161ed3d792c7a09d7".to_string(),
        }
    }

    fn test_boot_proof1() -> BootProof {
        BootProof {
            ephemeral_public_key_hex: "043144c138d522996833f9a5352b98da2ba669f1ab3f712b243a1993c89c7d682f2163c4c63c6625c483cd5e8714e9a2039e1ec05cebadf469a4adb7e721d16e19047b2846a313c6f3c543c23ae9779118c2411b28bd5d76b25572903c360240c841f4c4a3af4a297d770e9d8bceb5e7e5be31a8fe16e571b452e3de99ffcebcb2e1".to_string(),
            aws_attestation_doc_b64: "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".to_string(),
            qos_manifest_b64: "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".to_string(),
            qos_manifest_envelope_b64: "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".to_string(),
            deployment_label: "r2025.9.2".to_string(),
            enclave_app: "signer".to_string(),
            owner: "tkhq".to_string(),
            created_at: Some(Timestamp{seconds: "1758057949".to_string(), nanos: "436158000".to_string()}),
        }
    }

    fn test_boot_proof2() -> BootProof {
        BootProof {
            ephemeral_public_key_hex: "04ab52e052b94bc7f3badb1e0f3500d11062c9f44307bf76091b5c5f48cace9d35caba26028238e391f735c0a74996f98f69c593916aa5fcd6b3c7a56d33b5a31d0435d0354ae36981a010965d13fe6539827be12a17fcbb0fa35586c5b7732c4e8dc6a0efa81e762633f271d05d5440b2d4e97df48912d214c895bc42c7c61d076e".to_string(),
            aws_attestation_doc_b64: "hEShATgioFkRp79pbW9kdWxlX2lkeCdpLTAyZGM0MGQyMjU2NTQ0YjNhLWVuYzAxOTk1NDY5YTk2ZjVhMmJmZGlnZXN0ZlNIQTM4NGl0aW1lc3RhbXAbAAABmVRpwx5kcGNyc7AAWDD2cHao+XlrkNfw6xSOxpJvZv4EyAhhFRkWlh997HFbPIo25ZCOlVHCAEhxnaE0sgcBWDD2cHao+XlrkNfw6xSOxpJvZv4EyAhhFRkWlh997HFbPIo25ZCOlVHCAEhxnaE0sgcCWDAhue+8GEgHZi6WbTTzkIITCe6saAIwl5iCYpa/PovsfBDtswlIyQumcxD3uWT8UAoDWDCGTpCVqZR6sUaYEiNwwTuvIxg/TpkRlTz1uQmknbAPQ/RGcHMUZ02TCZdPPMSyRygEWDALJcHPFY9UQbk5Fw50/hhHlURNQz4NmO5r9M+9J2Uu5lSG74EF1n5rC2StM/WNXsEFWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAGWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAJWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAALWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAANWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAOWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAPWDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABrY2VydGlmaWNhdGVZAoAwggJ8MIICAaADAgECAhABmVRpqW9aKwAAAABoydVwMAoGCCqGSM49BAMDMIGOMQswCQYDVQQGEwJVUzETMBEGA1UECAwKV2FzaGluZ3RvbjEQMA4GA1UEBwwHU2VhdHRsZTEPMA0GA1UECgwGQW1hem9uMQwwCgYDVQQLDANBV1MxOTA3BgNVBAMMMGktMDJkYzQwZDIyNTY1NDRiM2EudXMtZWFzdC0xLmF3cy5uaXRyby1lbmNsYXZlczAeFw0yNTA5MTYyMTIzNTdaFw0yNTA5MTcwMDI0MDBaMIGTMQswCQYDVQQGEwJVUzETMBEGA1UECAwKV2FzaGluZ3RvbjEQMA4GA1UEBwwHU2VhdHRsZTEPMA0GA1UECgwGQW1hem9uMQwwCgYDVQQLDANBV1MxPjA8BgNVBAMMNWktMDJkYzQwZDIyNTY1NDRiM2EtZW5jMDE5OTU0NjlhOTZmNWEyYi51cy1lYXN0LTEuYXdzMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEDKryPxVZzUAyWyxrS/k7ETbWMN1lzYUkU00t1BsqAlBRVSfWLbESVjDgzmOUWKzgKp1M2X8irOKxsS3T8RDvgyLyH15lIQipOZIfT4IKvvpKIEmL61bo1BNSXR2jqBwNox0wGzAMBgNVHRMBAf8EAjAAMAsGA1UdDwQEAwIGwDAKBggqhkjOPQQDAwNpADBmAjEA27alZeWLAq6ii4oNre64t0frNUWPBFqLT3lEnHA9wsVlYKYyTwzkb8Z83MmgXzg8AjEAihHKWwWcwuX6XSTaCDyFMNeSNCjQyeVIR6ycHxn+smG/SroGz11q6rRqlu/lm4c9aGNhYnVuZGxlhFkCFTCCAhEwggGWoAMCAQICEQD5MXVoG5Cv4R1GzLTk5/hWMAoGCCqGSM49BAMDMEkxCzAJBgNVBAYTAlVTMQ8wDQYDVQQKDAZBbWF6b24xDDAKBgNVBAsMA0FXUzEbMBkGA1UEAwwSYXdzLm5pdHJvLWVuY2xhdmVzMB4XDTE5MTAyODEzMjgwNVoXDTQ5MTAyODE0MjgwNVowSTELMAkGA1UEBhMCVVMxDzANBgNVBAoMBkFtYXpvbjEMMAoGA1UECwwDQVdTMRswGQYDVQQDDBJhd3Mubml0cm8tZW5jbGF2ZXMwdjAQBgcqhkjOPQIBBgUrgQQAIgNiAAT8AlTrpgjB82hw4prakL5GODKSc26JS//2ctmJREtQUeU0pLH22+PAvFgaMrexdgcO3hLWmj/qIRtm51LPfdHdCV9vE3D0FwhD2dwQASHkz2MBKAlmRIfJeWKEME3FP/SjQjBAMA8GA1UdEwEB/wQFMAMBAf8wHQYDVR0OBBYEFJAltQ3ZBUfnlsOW+nKdz5mp30uWMA4GA1UdDwEB/wQEAwIBhjAKBggqhkjOPQQDAwNpADBmAjEAo38vkaHJvV7nuGJ8FpjSVQOOHwND+VtjqWKMPTmAlUWhHry/LjtV2K7ucbTD1q3zAjEAovObFgWycCil3UugabUBbmW0+96P4AYdalMZf5za9dlDvGH8K+sDy2/ujSMC89/2WQLDMIICvzCCAkSgAwIBAgIQUBf0g28nmLdo5SxGZy29HTAKBggqhkjOPQQDAzBJMQswCQYDVQQGEwJVUzEPMA0GA1UECgwGQW1hem9uMQwwCgYDVQQLDANBV1MxGzAZBgNVBAMMEmF3cy5uaXRyby1lbmNsYXZlczAeFw0yNTA5MTMwMTA3NDVaFw0yNTEwMDMwMjA3NDVaMGQxCzAJBgNVBAYTAlVTMQ8wDQYDVQQKDAZBbWF6b24xDDAKBgNVBAsMA0FXUzE2MDQGA1UEAwwtNzUzMGQwOTQyZDMwYzhiNi51cy1lYXN0LTEuYXdzLm5pdHJvLWVuY2xhdmVzMHYwEAYHKoZIzj0CAQYFK4EEACIDYgAEMbWjR2n4+3wS0MFR240sw5R9GhjIS+8xxYqSr03Rh9R1/GTVcx+g20AP31E2M0yEB30mcnsfpa/GsKA0A6CEjF484RtVxvLdFKVD6kESf7GzKljPxTh+FHQk/C/dErvmo4HVMIHSMBIGA1UdEwEB/wQIMAYBAf8CAQIwHwYDVR0jBBgwFoAUkCW1DdkFR+eWw5b6cp3PmanfS5YwHQYDVR0OBBYEFJYqWMGrtX55sQIeHOPTV4hVg0tgMA4GA1UdDwEB/wQEAwIBhjBsBgNVHR8EZTBjMGGgX6BdhltodHRwOi8vYXdzLW5pdHJvLWVuY2xhdmVzLWNybC5zMy5hbWF6b25hd3MuY29tL2NybC9hYjQ5NjBjYy03ZDYzLTQyYmQtOWU5Zi01OTMzOGNiNjdmODQuY3JsMAoGCCqGSM49BAMDA2kAMGYCMQCsUWPFTRTebtdOCAkHhYy0Ft8jcAfEfoqe0Nn7IAjHLYxtXCMbHggTM8j+jeQnjEECMQDPQf8Jp8uLnAJmkROJ/XIccjRWgxegSVsb4VwHGjWumD1birKZO9pnI/YZGaAe/XRZAxowggMWMIICm6ADAgECAhEAkvFbXHAWqxs1JZoZg75PQDAKBggqhkjOPQQDAzBkMQswCQYDVQQGEwJVUzEPMA0GA1UECgwGQW1hem9uMQwwCgYDVQQLDANBV1MxNjA0BgNVBAMMLTc1MzBkMDk0MmQzMGM4YjYudXMtZWFzdC0xLmF3cy5uaXRyby1lbmNsYXZlczAeFw0yNTA5MTYwNTQwMzdaFw0yNTA5MjEyMzQwMzdaMIGJMTwwOgYDVQQDDDNlZmMxNjU5NzFlYzEwYmZkLnpvbmFsLnVzLWVhc3QtMS5hd3Mubml0cm8tZW5jbGF2ZXMxDDAKBgNVBAsMA0FXUzEPMA0GA1UECgwGQW1hem9uMQswCQYDVQQGEwJVUzELMAkGA1UECAwCV0ExEDAOBgNVBAcMB1NlYXR0bGUwdjAQBgcqhkjOPQIBBgUrgQQAIgNiAATCx4VrBgCNBN812GoB/AvMRk0IKKJXx0lGLacsUzXAzjoWublmxkmGFpxnxfGBKwkz02xVmntiS2Db1oxER5IWP7CpFFtxDaWnxpsQXE6vV7zsQqMBPH6DSkKN4THcs1GjgeowgecwEgYDVR0TAQH/BAgwBgEB/wIBATAfBgNVHSMEGDAWgBSWKljBq7V+ebECHhzj01eIVYNLYDAdBgNVHQ4EFgQUVa2Vp/xrs3xFBT53XgNtO5oEr7UwDgYDVR0PAQH/BAQDAgGGMIGABgNVHR8EeTB3MHWgc6Bxhm9odHRwOi8vY3JsLXVzLWVhc3QtMS1hd3Mtbml0cm8tZW5jbGF2ZXMuczMudXMtZWFzdC0xLmFtYXpvbmF3cy5jb20vY3JsL2IzYjJiYzE1LTM5OTctNDIxNi1iMzhlLWY1N2U1ZDhhY2FmNS5jcmwwCgYIKoZIzj0EAwMDaQAwZgIxAOAqbP45jKX9PqrftFFbPzf0lV2XLpSLQFL4GCfqXsorfaml0KOKq0lKrBiTi7hUKwIxAOzdB3DvJpY0bitpgbjf8onNW/0BOLomI+Cx7j1HVTcHNTgsRg5EXUQkMQ1Rmsq+vVkCwjCCAr4wggJEoAMCAQICFGruZl3aKwP5Hq/4JivIwHGh1H7CMAoGCCqGSM49BAMDMIGJMTwwOgYDVQQDDDNlZmMxNjU5NzFlYzEwYmZkLnpvbmFsLnVzLWVhc3QtMS5hd3Mubml0cm8tZW5jbGF2ZXMxDDAKBgNVBAsMA0FXUzEPMA0GA1UECgwGQW1hem9uMQswCQYDVQQGEwJVUzELMAkGA1UECAwCV0ExEDAOBgNVBAcMB1NlYXR0bGUwHhcNMjUwOTE2MTEwMzIzWhcNMjUwOTE3MTEwMzIzWjCBjjELMAkGA1UEBhMCVVMxEzARBgNVBAgMCldhc2hpbmd0b24xEDAOBgNVBAcMB1NlYXR0bGUxDzANBgNVBAoMBkFtYXpvbjEMMAoGA1UECwwDQVdTMTkwNwYDVQQDDDBpLTAyZGM0MGQyMjU2NTQ0YjNhLnVzLWVhc3QtMS5hd3Mubml0cm8tZW5jbGF2ZXMwdjAQBgcqhkjOPQIBBgUrgQQAIgNiAAT5lcCYEFjepZjE0pn75wc7oUjm5EHz6St++qwYPSW8jH1QiSJWZsL0kSrsY9a6ZMwAke4dfMtyp9BvSZ4Ya8KkbR+RmpGSKfdjULVbAwQAyAByOfcAcAU8LWsrXn4L+rmjZjBkMBIGA1UdEwEB/wQIMAYBAf8CAQAwDgYDVR0PAQH/BAQDAgIEMB0GA1UdDgQWBBRvN5hMRhP1WJnQHxU04f9A90ptmjAfBgNVHSMEGDAWgBRVrZWn/GuzfEUFPndeA207mgSvtTAKBggqhkjOPQQDAwNoADBlAjEA8JNexd0JNW8zyu5ht3oyF4dQqFbTWow7RRXd1Fk9mPiBLQMhKkAYH4/4wsT/FgSVAjAoPuboktN3WLNH7CSabxutc+9kxEMBCQnSjjzqwJ8ly1Zpt5d58yaLMRr2g7s8aytqcHVibGljX2tleViCBKtS4FK5S8fzutseDzUA0RBiyfRDB792CRtcX0jKzp01yromAoI445H3NcCnSZb5j2nFk5FqpfzWs8elbTO1ox0ENdA1SuNpgaAQll0T/mU5gnvhKhf8uw+jVYbFt3MsTo3GoO+oHnYmM/Jx0F1UQLLU6X30iRLSFMiVvELHxh0Hbml1c2VyX2RhdGFYICSp4sCwzia8vD5lFy+LBrEA1ecSWR9gSDLL6p/1zgK+ZW5vbmNl9v9YYNjtasG5hCb/0b57m/Tz20awgJORLk671VuOaPD0pGpeVp7edFHRci94te1uWCZ9NcM2N1BQ9XQUNaTUc1FUIuDUvrEaOUvIeBN0TMwdniwdQQKeo0NWikSgvaFJ8sEU1w==".to_string(),
            qos_manifest_b64: "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".to_string(),
            qos_manifest_envelope_b64: "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".to_string(),
            deployment_label: "r2025.9.2".to_string(),
            enclave_app: "signer".to_string(),
            owner: "tkhq".to_string(),
            created_at: Some(Timestamp{seconds: "1758057841".to_string(), nanos: "448145000".to_string()}),
        }
    }

    #[test]
    fn test_should_verify_app_proof_signature() {
        assert!(verify_app_proof_signature(&test_app_proof1()).is_ok());
        assert!(verify_app_proof_signature(&test_app_proof2()).is_ok());
    }

    #[test]
    fn test_should_verify() {
        assert!(verify(&test_app_proof1(), &test_boot_proof1()).is_ok());
        assert!(verify(&test_app_proof2(), &test_boot_proof2()).is_ok());
    }

    #[test]
    fn test_get_app_proof_time() {
        let app_proof = test_app_proof1();
        let time_result = get_app_proof_time(&app_proof);
        assert!(time_result.is_ok());

        let time = time_result.unwrap();
        let duration_since_epoch = time.duration_since(std::time::UNIX_EPOCH).unwrap();
        assert_eq!(duration_since_epoch.as_millis(), 1758058763571);
    }

    #[test]
    fn test_get_boot_proof_time() {
        let boot_proof = test_boot_proof1();
        let time_result = get_boot_proof_time(&boot_proof);
        assert!(time_result.is_ok());

        let time = time_result.unwrap();
        let duration_since_epoch = time.duration_since(std::time::UNIX_EPOCH).unwrap();
        assert_eq!(duration_since_epoch.as_secs(), 1758057949);
        assert_eq!(duration_since_epoch.subsec_nanos(), 436158000);
    }

    #[test]
    fn test_should_not_verify_with_malformed_app_proofs() {
        let app_proof1 = test_app_proof1();
        let app_proof2 = test_app_proof2();
        let mut malformed_app_proof2 = test_app_proof2();

        // Wrong publicKey - should cause signature verification failure
        malformed_app_proof2.public_key = app_proof1.public_key;
        let result = verify_app_proof_signature(&malformed_app_proof2);
        assert!(matches!(
            result,
            Err(AppProofError::FailedSignatureVerification(_))
        ));
        malformed_app_proof2.public_key = app_proof2.public_key;

        // Wrong proofPayload - should cause signature verification failure
        malformed_app_proof2.proof_payload = app_proof1.proof_payload;
        let result = verify_app_proof_signature(&malformed_app_proof2);
        assert!(matches!(
            result,
            Err(AppProofError::FailedSignatureVerification(_))
        ));
        malformed_app_proof2.proof_payload = app_proof2.proof_payload;

        // Wrong signature - should cause signature verification failure
        malformed_app_proof2.signature = app_proof1.signature;
        let result = verify_app_proof_signature(&malformed_app_proof2);
        assert!(matches!(
            result,
            Err(AppProofError::FailedSignatureVerification(_))
        ));
    }

    #[test]
    fn test_should_not_verify_with_malformed_boot_proofs() {
        let app_proof1 = test_app_proof1();
        let boot_proof1 = test_boot_proof1();
        let mut malformed_boot_proof1 = test_boot_proof1();
        let boot_proof2 = test_boot_proof2();

        // Wrong ephemeral key - should cause ephemeral key mismatch
        malformed_boot_proof1.ephemeral_public_key_hex = boot_proof2.ephemeral_public_key_hex;
        let result = verify(&app_proof1, &malformed_boot_proof1);
        assert!(matches!(result, Err(VerifyError::DifferentEphemeralKey(_))));
        malformed_boot_proof1.ephemeral_public_key_hex = boot_proof1.ephemeral_public_key_hex;

        // Wrong awsAttestationDocB64 - causes ephemeral key mistmatch error because the boot proofs have the same qos manifest, so the user data check doesn't fail
        malformed_boot_proof1.aws_attestation_doc_b64 = boot_proof2.aws_attestation_doc_b64;
        let result = verify(&app_proof1, &malformed_boot_proof1);
        assert!(matches!(result, Err(VerifyError::DifferentEphemeralKey(_))));
        malformed_boot_proof1.aws_attestation_doc_b64 = boot_proof1.aws_attestation_doc_b64;

        // Wrong qosManifestB64 (randomly generated) - should cause user_data verification failure
        malformed_boot_proof1.qos_manifest_b64 =
            "puwpDDoA3dqZpFV0nDnSgj2iFyy2VDUnDSE7u+awts0=".to_string();
        let result = verify(&app_proof1, &malformed_boot_proof1);
        assert!(matches!(result, Err(VerifyError::DifferentManifest(_))));
    }
}
