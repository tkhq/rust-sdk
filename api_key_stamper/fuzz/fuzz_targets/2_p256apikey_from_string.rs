#![no_main]

use libfuzzer_sys::fuzz_target;
use turnkey_api_key_stamper::TurnkeyP256ApiKey;
use turnkey_api_key_stamper_fuzz::{use_p256apikey_if_ok, P256_PRIVATE_KEY_EXPECTED_LENGTH};

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct FuzzP256ApiKeyStruct {
    private_key_hex: String,
    public_key_hex: String,
}

fuzz_target!(|data: FuzzP256ApiKeyStruct| {
    // workaround to avoid internal data conversion panics in TurnkeyP256ApiKey::from_bytes()
    // double expected length due to hex string format
    if data.private_key_hex.len() != P256_PRIVATE_KEY_EXPECTED_LENGTH * 2 {
        return;
    }

    // test without optional public key
    let res1 = TurnkeyP256ApiKey::from_strings(data.private_key_hex.clone(), None);
    use_p256apikey_if_ok(&res1);

    // test with optional public key
    let res2 = TurnkeyP256ApiKey::from_strings(data.private_key_hex, Some(data.public_key_hex));

    // the fuzzer isn't clever enough to come up with a matching private key and public key pair on its own
    // therefore we don't expect the conversion to succeed without errors
    assert!(res2.is_err());
});
