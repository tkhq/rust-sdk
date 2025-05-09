#![doc = include_str!("../README.md")]

use aws_nitro_enclaves_cose::{
    crypto::{Hash, MessageDigest, SignatureAlgorithm, SigningPublicKey},
    error::CoseError,
    CoseSign1,
};
use aws_nitro_enclaves_nsm_api::api::AttestationDoc;
use p384::{
    ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey},
    PublicKey,
};
use serde_bytes::ByteBuf;

mod error;
mod syntactic_validation;
mod types;

pub use error::AttestError;

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
pub fn attestation_doc_from_der(
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

/// Parses and verifies an AWS nitro attestation, provided as a base64 encoded string.
pub fn parse_and_verify_aws_nitro_attestation<S: AsRef<str>>(
    encoded_attestation: S,
) -> Result<AttestationDoc, AttestError> {
    // Decode the base64 string
    let decoded_bytes = base64::decode(encoded_attestation.as_ref())
        .map_err(|e| AttestError::Base64DecodingError(e.to_string()))?;

    let trusted_root_certificate = cert_from_pem(AWS_ROOT_CERT_PEM).unwrap();

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    attestation_doc_from_der(
        decoded_bytes.as_slice(),
        trusted_root_certificate.as_slice(),
        current_time,
    )
}
