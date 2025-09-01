#![no_main]

use libfuzzer_sys::fuzz_target;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_api_key_stamper_fuzz::{use_p256apikey_if_ok, P256_PRIVATE_KEY_EXPECTED_LENGTH};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct FuzzP256ApiKeyStruct {
    private_key_bytes: Box<[u8]>,
    public_key_bytes: Box<[u8]>,
}

fuzz_target!(|data: FuzzP256ApiKeyStruct| {
    // workaround to avoid internal data conversion panics in TurnkeyP256ApiKey::from_bytes()
    if data.private_key_bytes.len() != P256_PRIVATE_KEY_EXPECTED_LENGTH {
        return;
    }

    // test without optional public key
    let res1 = TurnkeyP256ApiKey::from_bytes(data.private_key_bytes.clone(), None);
    use_p256apikey_if_ok(&res1);

    // test with optional public key
    let res2 = TurnkeyP256ApiKey::from_bytes(
        data.private_key_bytes.clone(),
        Some(data.public_key_bytes.clone()),
    );

    // the fuzzer isn't clever enough to come up with a matching private key and public key pair on its own
    // therefore we don't expect the conversion to succeed without errors
    assert!(res2.is_err());
});
