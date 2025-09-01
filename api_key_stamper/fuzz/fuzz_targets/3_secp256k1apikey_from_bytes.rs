#![no_main]

use libfuzzer_sys::fuzz_target;
use turnkey_api_key_stamper::{TurnkeySecp256k1ApiKey};
use turnkey_api_key_stamper_fuzz::use_secp256k1apikey_if_ok;

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct FuzzSecp256k1ApiKeyStruct {
    private_key_bytes: Box<[u8]>,
    public_key_bytes: Box<[u8]>,
}

fuzz_target!(|data: FuzzSecp256k1ApiKeyStruct| {
    // test without optional public key
    let res1 = TurnkeySecp256k1ApiKey::from_bytes(data.private_key_bytes.clone(), None);
    use_secp256k1apikey_if_ok(&res1);

    // test with optional public key
    let res2 = TurnkeySecp256k1ApiKey::from_bytes(data.private_key_bytes.clone(), Some(data.public_key_bytes.clone()));

    // the fuzzer isn't clever enough to come up with a matching private key and public key pair on its own
    // therefore we don't expect the conversion to succeed without errors
    assert!(res2.is_err());
});
