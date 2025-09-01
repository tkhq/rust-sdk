use turnkey_api_key_stamper::{Stamp, StamperError, TurnkeyP256ApiKey, TurnkeySecp256k1ApiKey};

pub const P256_PRIVATE_KEY_EXPECTED_LENGTH: usize = 32;

/// Test public functions of [`TurnkeyP256ApiKey`] if possible
pub fn use_p256apikey_if_ok(apikeyresult: &Result<TurnkeyP256ApiKey, StamperError>) {
    match apikeyresult {
        Ok(res) => {
            // exercise the public key generation codepath
            let _ = res.compressed_public_key();

            // test stamping with dummy message
            let _ = <TurnkeyP256ApiKey as Stamp>::stamp(&res, b"hello").unwrap();
        }
        Err(_) => {
            // do nothing
        }
    }
}

/// Test public functions of [`TurnkeySecp256k1ApiKey`] if possible
pub fn use_secp256k1apikey_if_ok(apikeyresult: &Result<TurnkeySecp256k1ApiKey, StamperError>) {
    match apikeyresult {
        Ok(res) => {
            // exercise the public key generation codepath
            let _ = res.compressed_public_key();

            // test stamping with dummy message
            let _ = <TurnkeySecp256k1ApiKey as Stamp>::stamp(&res, b"hello").unwrap();
        }
        Err(_) => {
            // do nothing
        }
    }
}
