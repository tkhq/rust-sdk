use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use p256::ecdsa::{Signature, SigningKey};
use p256::ecdsa::signature::Signer;
use p256::FieldBytes;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiStamp {
    public_key: String,
    signature: String,
    scheme: String,
}

pub struct TurnkeyApiKey {
    pub private_key_hex: String,
    pub public_key_hex: String,
}

pub fn stamp(request_body: String, api_key: &TurnkeyApiKey) -> Result<String, StampError> {
    let private_key_bytes = hex::decode(&api_key.private_key_hex).unwrap();
    let signing_key: SigningKey = SigningKey::from_bytes(FieldBytes::from_slice(&private_key_bytes)).unwrap();
    let sig: Signature = signing_key.sign(request_body.as_bytes());

    let stamp = ApiStamp {
        public_key: api_key.public_key_hex.clone(),
        signature: hex::encode(sig.to_der()),
        scheme: "SIGNATURE_SCHEME_TK_API_P256".to_string(),
    };

    let json_stamp = serde_json::to_string(&stamp).unwrap();

    BASE64_URL_SAFE_NO_PAD.encode(json_stamp.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_stamps() {
        let stamp = stamp(
            "hello from TKHQ".to_string(), 
            &TurnkeyApiKey { private_key_hex: "9720de87f61537e481f95f4433bed97b9d60719457c4dd20dac4bbf377f59c69".to_string(), public_key_hex: "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45".to_string()},
        );

        // The stamp should be valid base64
        let decoded_stamp_bytes = BASE64_URL_SAFE_NO_PAD.decode(stamp).unwrap();
        
        // These bytes should be valid UTF8 characters
        let decoded_stamp_string = String::from_utf8(decoded_stamp_bytes).unwrap();
        
        // The resulting string should be valid JSON
        let json_stamp: Value = serde_json::from_str(&decoded_stamp_string).unwrap();

        // And finally: the signature scheme and public key should be correct
        assert_eq!(json_stamp["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
        assert_eq!(json_stamp["publicKey"], "02a1d9ee281053cf73c07678d6c1231216e8434f87662b75f08c66882c2f95ee45");
    }
}
