#[derive(Debug)]
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Clone, PartialEq)]
pub struct PolicyEvaluation {
    pub policy_id: ::prost::alloc::string::String,
    pub outcome: Outcome,
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathFormat {
    #[serde(rename = "PATH_FORMAT_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "PATH_FORMAT_BIP32")]
    Bip32 = 1,
}
impl PathFormat {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "PATH_FORMAT_UNSPECIFIED",
            Self::Bip32 => "PATH_FORMAT_BIP32",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PATH_FORMAT_UNSPECIFIED" => Some(Self::Unspecified),
            "PATH_FORMAT_BIP32" => Some(Self::Bip32),
            _ => None,
        }
    }
}
/// Cryptographic Curve used to generate a given API key
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ApiKeyCurve {
    #[serde(rename = "API_KEY_CURVE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "API_KEY_CURVE_P256")]
    P256 = 2,
    #[serde(rename = "API_KEY_CURVE_SECP256K1")]
    Secp256k1 = 3,
    #[serde(rename = "API_KEY_CURVE_ED25519")]
    Ed25519 = 4,
}
impl ApiKeyCurve {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "API_KEY_CURVE_UNSPECIFIED",
            Self::P256 => "API_KEY_CURVE_P256",
            Self::Secp256k1 => "API_KEY_CURVE_SECP256K1",
            Self::Ed25519 => "API_KEY_CURVE_ED25519",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "API_KEY_CURVE_UNSPECIFIED" => Some(Self::Unspecified),
            "API_KEY_CURVE_P256" => Some(Self::P256),
            "API_KEY_CURVE_SECP256K1" => Some(Self::Secp256k1),
            "API_KEY_CURVE_ED25519" => Some(Self::Ed25519),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClientSignatureScheme {
    #[serde(rename = "CLIENT_SIGNATURE_SCHEME_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "CLIENT_SIGNATURE_SCHEME_API_P256")]
    ApiP256 = 1,
}
impl ClientSignatureScheme {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CLIENT_SIGNATURE_SCHEME_UNSPECIFIED",
            Self::ApiP256 => "CLIENT_SIGNATURE_SCHEME_API_P256",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CLIENT_SIGNATURE_SCHEME_UNSPECIFIED" => Some(Self::Unspecified),
            "CLIENT_SIGNATURE_SCHEME_API_P256" => Some(Self::ApiP256),
            _ => None,
        }
    }
}
/// Cryptographic Curve used to generate a given Private Key.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Curve {
    #[serde(rename = "CURVE_UNSPECIFIED")]
    Unspecified = 0,
    /// Curve SECP256K1 as defined in <https://www.secg.org/sec2-v2.pdf>
    #[serde(rename = "CURVE_SECP256K1")]
    Secp256k1 = 1,
    /// Curve ED25519 as defined in <https://www.rfc-editor.org/rfc/rfc8032>
    #[serde(rename = "CURVE_ED25519")]
    Ed25519 = 2,
    /// Curve NIST P-256 (secp256r1) as defined in <https://csrc.nist.gov/csrc/media/events/workshop-on-elliptic-curve-cryptography-standards/documents/papers/session6-adalier-mehmet.pdf>
    #[serde(rename = "CURVE_P256")]
    P256 = 3,
}
impl Curve {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CURVE_UNSPECIFIED",
            Self::Secp256k1 => "CURVE_SECP256K1",
            Self::Ed25519 => "CURVE_ED25519",
            Self::P256 => "CURVE_P256",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CURVE_UNSPECIFIED" => Some(Self::Unspecified),
            "CURVE_SECP256K1" => Some(Self::Secp256k1),
            "CURVE_ED25519" => Some(Self::Ed25519),
            "CURVE_P256" => Some(Self::P256),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AddressFormat {
    #[serde(rename = "ADDRESS_FORMAT_UNSPECIFIED")]
    Unspecified = 0,
    /// 04<X_COORDINATE><Y_COORDINATE>
    #[serde(rename = "ADDRESS_FORMAT_UNCOMPRESSED")]
    Uncompressed = 1,
    /// 02 or 03, followed by the X coordinate
    #[serde(rename = "ADDRESS_FORMAT_COMPRESSED")]
    Compressed = 2,
    #[serde(rename = "ADDRESS_FORMAT_ETHEREUM")]
    Ethereum = 3,
    #[serde(rename = "ADDRESS_FORMAT_SOLANA")]
    Solana = 4,
    #[serde(rename = "ADDRESS_FORMAT_COSMOS")]
    Cosmos = 5,
    #[serde(rename = "ADDRESS_FORMAT_TRON")]
    Tron = 6,
    #[serde(rename = "ADDRESS_FORMAT_SUI")]
    Sui = 7,
    #[serde(rename = "ADDRESS_FORMAT_APTOS")]
    Aptos = 8,
    /// Bitcoin Mainnet address types
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_MAINNET_P2PKH")]
    BitcoinMainnetP2pkh = 9,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_MAINNET_P2SH")]
    BitcoinMainnetP2sh = 10,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WPKH")]
    BitcoinMainnetP2wpkh = 11,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WSH")]
    BitcoinMainnetP2wsh = 12,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_MAINNET_P2TR")]
    BitcoinMainnetP2tr = 13,
    /// Bitcoin Testnet address types
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_TESTNET_P2PKH")]
    BitcoinTestnetP2pkh = 14,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_TESTNET_P2SH")]
    BitcoinTestnetP2sh = 15,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WPKH")]
    BitcoinTestnetP2wpkh = 16,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WSH")]
    BitcoinTestnetP2wsh = 17,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_TESTNET_P2TR")]
    BitcoinTestnetP2tr = 18,
    /// Bitcoin Signet address types
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_SIGNET_P2PKH")]
    BitcoinSignetP2pkh = 19,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_SIGNET_P2SH")]
    BitcoinSignetP2sh = 20,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WPKH")]
    BitcoinSignetP2wpkh = 21,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WSH")]
    BitcoinSignetP2wsh = 22,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_SIGNET_P2TR")]
    BitcoinSignetP2tr = 23,
    /// Bitcoin Regtest address types
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_REGTEST_P2PKH")]
    BitcoinRegtestP2pkh = 24,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_REGTEST_P2SH")]
    BitcoinRegtestP2sh = 25,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WPKH")]
    BitcoinRegtestP2wpkh = 26,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WSH")]
    BitcoinRegtestP2wsh = 27,
    #[serde(rename = "ADDRESS_FORMAT_BITCOIN_REGTEST_P2TR")]
    BitcoinRegtestP2tr = 28,
    #[serde(rename = "ADDRESS_FORMAT_SEI")]
    Sei = 29,
    #[serde(rename = "ADDRESS_FORMAT_XLM")]
    Xlm = 30,
    /// Doge Addresses
    #[serde(rename = "ADDRESS_FORMAT_DOGE_MAINNET")]
    DogeMainnet = 31,
    #[serde(rename = "ADDRESS_FORMAT_DOGE_TESTNET")]
    DogeTestnet = 32,
    /// TON Addresses
    #[serde(rename = "ADDRESS_FORMAT_TON_V3R2")]
    TonV3r2 = 33,
    #[serde(rename = "ADDRESS_FORMAT_TON_V4R2")]
    TonV4r2 = 34,
    #[serde(rename = "ADDRESS_FORMAT_TON_V5R1")]
    TonV5r1 = 36,
    #[serde(rename = "ADDRESS_FORMAT_XRP")]
    Xrp = 35,
}
impl AddressFormat {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ADDRESS_FORMAT_UNSPECIFIED",
            Self::Uncompressed => "ADDRESS_FORMAT_UNCOMPRESSED",
            Self::Compressed => "ADDRESS_FORMAT_COMPRESSED",
            Self::Ethereum => "ADDRESS_FORMAT_ETHEREUM",
            Self::Solana => "ADDRESS_FORMAT_SOLANA",
            Self::Cosmos => "ADDRESS_FORMAT_COSMOS",
            Self::Tron => "ADDRESS_FORMAT_TRON",
            Self::Sui => "ADDRESS_FORMAT_SUI",
            Self::Aptos => "ADDRESS_FORMAT_APTOS",
            Self::BitcoinMainnetP2pkh => "ADDRESS_FORMAT_BITCOIN_MAINNET_P2PKH",
            Self::BitcoinMainnetP2sh => "ADDRESS_FORMAT_BITCOIN_MAINNET_P2SH",
            Self::BitcoinMainnetP2wpkh => "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WPKH",
            Self::BitcoinMainnetP2wsh => "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WSH",
            Self::BitcoinMainnetP2tr => "ADDRESS_FORMAT_BITCOIN_MAINNET_P2TR",
            Self::BitcoinTestnetP2pkh => "ADDRESS_FORMAT_BITCOIN_TESTNET_P2PKH",
            Self::BitcoinTestnetP2sh => "ADDRESS_FORMAT_BITCOIN_TESTNET_P2SH",
            Self::BitcoinTestnetP2wpkh => "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WPKH",
            Self::BitcoinTestnetP2wsh => "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WSH",
            Self::BitcoinTestnetP2tr => "ADDRESS_FORMAT_BITCOIN_TESTNET_P2TR",
            Self::BitcoinSignetP2pkh => "ADDRESS_FORMAT_BITCOIN_SIGNET_P2PKH",
            Self::BitcoinSignetP2sh => "ADDRESS_FORMAT_BITCOIN_SIGNET_P2SH",
            Self::BitcoinSignetP2wpkh => "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WPKH",
            Self::BitcoinSignetP2wsh => "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WSH",
            Self::BitcoinSignetP2tr => "ADDRESS_FORMAT_BITCOIN_SIGNET_P2TR",
            Self::BitcoinRegtestP2pkh => "ADDRESS_FORMAT_BITCOIN_REGTEST_P2PKH",
            Self::BitcoinRegtestP2sh => "ADDRESS_FORMAT_BITCOIN_REGTEST_P2SH",
            Self::BitcoinRegtestP2wpkh => "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WPKH",
            Self::BitcoinRegtestP2wsh => "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WSH",
            Self::BitcoinRegtestP2tr => "ADDRESS_FORMAT_BITCOIN_REGTEST_P2TR",
            Self::Sei => "ADDRESS_FORMAT_SEI",
            Self::Xlm => "ADDRESS_FORMAT_XLM",
            Self::DogeMainnet => "ADDRESS_FORMAT_DOGE_MAINNET",
            Self::DogeTestnet => "ADDRESS_FORMAT_DOGE_TESTNET",
            Self::TonV3r2 => "ADDRESS_FORMAT_TON_V3R2",
            Self::TonV4r2 => "ADDRESS_FORMAT_TON_V4R2",
            Self::TonV5r1 => "ADDRESS_FORMAT_TON_V5R1",
            Self::Xrp => "ADDRESS_FORMAT_XRP",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ADDRESS_FORMAT_UNSPECIFIED" => Some(Self::Unspecified),
            "ADDRESS_FORMAT_UNCOMPRESSED" => Some(Self::Uncompressed),
            "ADDRESS_FORMAT_COMPRESSED" => Some(Self::Compressed),
            "ADDRESS_FORMAT_ETHEREUM" => Some(Self::Ethereum),
            "ADDRESS_FORMAT_SOLANA" => Some(Self::Solana),
            "ADDRESS_FORMAT_COSMOS" => Some(Self::Cosmos),
            "ADDRESS_FORMAT_TRON" => Some(Self::Tron),
            "ADDRESS_FORMAT_SUI" => Some(Self::Sui),
            "ADDRESS_FORMAT_APTOS" => Some(Self::Aptos),
            "ADDRESS_FORMAT_BITCOIN_MAINNET_P2PKH" => Some(Self::BitcoinMainnetP2pkh),
            "ADDRESS_FORMAT_BITCOIN_MAINNET_P2SH" => Some(Self::BitcoinMainnetP2sh),
            "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WPKH" => Some(Self::BitcoinMainnetP2wpkh),
            "ADDRESS_FORMAT_BITCOIN_MAINNET_P2WSH" => Some(Self::BitcoinMainnetP2wsh),
            "ADDRESS_FORMAT_BITCOIN_MAINNET_P2TR" => Some(Self::BitcoinMainnetP2tr),
            "ADDRESS_FORMAT_BITCOIN_TESTNET_P2PKH" => Some(Self::BitcoinTestnetP2pkh),
            "ADDRESS_FORMAT_BITCOIN_TESTNET_P2SH" => Some(Self::BitcoinTestnetP2sh),
            "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WPKH" => Some(Self::BitcoinTestnetP2wpkh),
            "ADDRESS_FORMAT_BITCOIN_TESTNET_P2WSH" => Some(Self::BitcoinTestnetP2wsh),
            "ADDRESS_FORMAT_BITCOIN_TESTNET_P2TR" => Some(Self::BitcoinTestnetP2tr),
            "ADDRESS_FORMAT_BITCOIN_SIGNET_P2PKH" => Some(Self::BitcoinSignetP2pkh),
            "ADDRESS_FORMAT_BITCOIN_SIGNET_P2SH" => Some(Self::BitcoinSignetP2sh),
            "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WPKH" => Some(Self::BitcoinSignetP2wpkh),
            "ADDRESS_FORMAT_BITCOIN_SIGNET_P2WSH" => Some(Self::BitcoinSignetP2wsh),
            "ADDRESS_FORMAT_BITCOIN_SIGNET_P2TR" => Some(Self::BitcoinSignetP2tr),
            "ADDRESS_FORMAT_BITCOIN_REGTEST_P2PKH" => Some(Self::BitcoinRegtestP2pkh),
            "ADDRESS_FORMAT_BITCOIN_REGTEST_P2SH" => Some(Self::BitcoinRegtestP2sh),
            "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WPKH" => Some(Self::BitcoinRegtestP2wpkh),
            "ADDRESS_FORMAT_BITCOIN_REGTEST_P2WSH" => Some(Self::BitcoinRegtestP2wsh),
            "ADDRESS_FORMAT_BITCOIN_REGTEST_P2TR" => Some(Self::BitcoinRegtestP2tr),
            "ADDRESS_FORMAT_SEI" => Some(Self::Sei),
            "ADDRESS_FORMAT_XLM" => Some(Self::Xlm),
            "ADDRESS_FORMAT_DOGE_MAINNET" => Some(Self::DogeMainnet),
            "ADDRESS_FORMAT_DOGE_TESTNET" => Some(Self::DogeTestnet),
            "ADDRESS_FORMAT_TON_V3R2" => Some(Self::TonV3r2),
            "ADDRESS_FORMAT_TON_V4R2" => Some(Self::TonV4r2),
            "ADDRESS_FORMAT_TON_V5R1" => Some(Self::TonV5r1),
            "ADDRESS_FORMAT_XRP" => Some(Self::Xrp),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HashFunction {
    #[serde(rename = "HASH_FUNCTION_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "HASH_FUNCTION_NO_OP")]
    NoOp = 1,
    #[serde(rename = "HASH_FUNCTION_SHA256")]
    Sha256 = 2,
    #[serde(rename = "HASH_FUNCTION_KECCAK256")]
    Keccak256 = 3,
    #[serde(rename = "HASH_FUNCTION_NOT_APPLICABLE")]
    NotApplicable = 4,
}
impl HashFunction {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "HASH_FUNCTION_UNSPECIFIED",
            Self::NoOp => "HASH_FUNCTION_NO_OP",
            Self::Sha256 => "HASH_FUNCTION_SHA256",
            Self::Keccak256 => "HASH_FUNCTION_KECCAK256",
            Self::NotApplicable => "HASH_FUNCTION_NOT_APPLICABLE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "HASH_FUNCTION_UNSPECIFIED" => Some(Self::Unspecified),
            "HASH_FUNCTION_NO_OP" => Some(Self::NoOp),
            "HASH_FUNCTION_SHA256" => Some(Self::Sha256),
            "HASH_FUNCTION_KECCAK256" => Some(Self::Keccak256),
            "HASH_FUNCTION_NOT_APPLICABLE" => Some(Self::NotApplicable),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PayloadEncoding {
    /// Default value if payload encoding is not set explicitly
    #[serde(rename = "PAYLOAD_ENCODING_UNSPECIFIED")]
    Unspecified = 0,
    /// Payload is encoded in hexadecimal
    /// We accept 0x-prefixed or non-0x prefixed payloads.
    /// We accept any casing (uppercase, lowercase, or mixed)
    #[serde(rename = "PAYLOAD_ENCODING_HEXADECIMAL")]
    Hexadecimal = 1,
    /// Payload is encoded as utf-8 text
    /// Will be converted to bytes for signature with Rust's standard String.as_bytes()
    #[serde(rename = "PAYLOAD_ENCODING_TEXT_UTF8")]
    TextUtf8 = 2,
    /// Payload is encoded as EIP-712 typed data
    /// See JSON schema definition in EIP-712 documentation here: <https://eips.ethereum.org/EIPS/eip-712#parameters>
    /// Will be converted to bytes for signing using serde_json::from_str
    #[serde(rename = "PAYLOAD_ENCODING_EIP712")]
    Eip712 = 3,
    /// Payload is encoded as an EIP-7702 Authorization
    /// See spec here: <https://eips.ethereum.org/EIPS/eip-7702#behavior>
    /// Will be converted to bytes for signing using serde_json::from_str
    #[serde(rename = "PAYLOAD_ENCODING_EIP7702_AUTHORIZATION")]
    Eip7702Authorization = 4,
}
impl PayloadEncoding {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "PAYLOAD_ENCODING_UNSPECIFIED",
            Self::Hexadecimal => "PAYLOAD_ENCODING_HEXADECIMAL",
            Self::TextUtf8 => "PAYLOAD_ENCODING_TEXT_UTF8",
            Self::Eip712 => "PAYLOAD_ENCODING_EIP712",
            Self::Eip7702Authorization => "PAYLOAD_ENCODING_EIP7702_AUTHORIZATION",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "PAYLOAD_ENCODING_UNSPECIFIED" => Some(Self::Unspecified),
            "PAYLOAD_ENCODING_HEXADECIMAL" => Some(Self::Hexadecimal),
            "PAYLOAD_ENCODING_TEXT_UTF8" => Some(Self::TextUtf8),
            "PAYLOAD_ENCODING_EIP712" => Some(Self::Eip712),
            "PAYLOAD_ENCODING_EIP7702_AUTHORIZATION" => Some(Self::Eip7702Authorization),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MnemonicLanguage {
    #[serde(rename = "MNEMONIC_LANGUAGE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "MNEMONIC_LANGUAGE_ENGLISH")]
    English = 1,
    #[serde(rename = "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE")]
    SimplifiedChinese = 2,
    #[serde(rename = "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE")]
    TraditionalChinese = 3,
    #[serde(rename = "MNEMONIC_LANGUAGE_CZECH")]
    Czech = 4,
    #[serde(rename = "MNEMONIC_LANGUAGE_FRENCH")]
    French = 5,
    #[serde(rename = "MNEMONIC_LANGUAGE_ITALIAN")]
    Italian = 6,
    #[serde(rename = "MNEMONIC_LANGUAGE_JAPANESE")]
    Japanese = 7,
    #[serde(rename = "MNEMONIC_LANGUAGE_KOREAN")]
    Korean = 8,
    #[serde(rename = "MNEMONIC_LANGUAGE_SPANISH")]
    Spanish = 9,
}
impl MnemonicLanguage {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "MNEMONIC_LANGUAGE_UNSPECIFIED",
            Self::English => "MNEMONIC_LANGUAGE_ENGLISH",
            Self::SimplifiedChinese => "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE",
            Self::TraditionalChinese => "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE",
            Self::Czech => "MNEMONIC_LANGUAGE_CZECH",
            Self::French => "MNEMONIC_LANGUAGE_FRENCH",
            Self::Italian => "MNEMONIC_LANGUAGE_ITALIAN",
            Self::Japanese => "MNEMONIC_LANGUAGE_JAPANESE",
            Self::Korean => "MNEMONIC_LANGUAGE_KOREAN",
            Self::Spanish => "MNEMONIC_LANGUAGE_SPANISH",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MNEMONIC_LANGUAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "MNEMONIC_LANGUAGE_ENGLISH" => Some(Self::English),
            "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE" => Some(Self::SimplifiedChinese),
            "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE" => Some(Self::TraditionalChinese),
            "MNEMONIC_LANGUAGE_CZECH" => Some(Self::Czech),
            "MNEMONIC_LANGUAGE_FRENCH" => Some(Self::French),
            "MNEMONIC_LANGUAGE_ITALIAN" => Some(Self::Italian),
            "MNEMONIC_LANGUAGE_JAPANESE" => Some(Self::Japanese),
            "MNEMONIC_LANGUAGE_KOREAN" => Some(Self::Korean),
            "MNEMONIC_LANGUAGE_SPANISH" => Some(Self::Spanish),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Effect {
    #[serde(rename = "EFFECT_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "EFFECT_ALLOW")]
    Allow = 1,
    #[serde(rename = "EFFECT_DENY")]
    Deny = 2,
}
impl Effect {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "EFFECT_UNSPECIFIED",
            Self::Allow => "EFFECT_ALLOW",
            Self::Deny => "EFFECT_DENY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EFFECT_UNSPECIFIED" => Some(Self::Unspecified),
            "EFFECT_ALLOW" => Some(Self::Allow),
            "EFFECT_DENY" => Some(Self::Deny),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessType {
    #[serde(rename = "ACCESS_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "ACCESS_TYPE_WEB")]
    Web = 1,
    #[serde(rename = "ACCESS_TYPE_API")]
    Api = 2,
    #[serde(rename = "ACCESS_TYPE_ALL")]
    All = 3,
}
impl AccessType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "ACCESS_TYPE_UNSPECIFIED",
            Self::Web => "ACCESS_TYPE_WEB",
            Self::Api => "ACCESS_TYPE_API",
            Self::All => "ACCESS_TYPE_ALL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACCESS_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "ACCESS_TYPE_WEB" => Some(Self::Web),
            "ACCESS_TYPE_API" => Some(Self::Api),
            "ACCESS_TYPE_ALL" => Some(Self::All),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CredentialType {
    #[serde(rename = "CREDENTIAL_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR")]
    WebauthnAuthenticator = 1,
    #[serde(rename = "CREDENTIAL_TYPE_API_KEY_P256")]
    ApiKeyP256 = 2,
    #[serde(rename = "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256")]
    RecoverUserKeyP256 = 3,
    #[serde(rename = "CREDENTIAL_TYPE_API_KEY_SECP256K1")]
    ApiKeySecp256k1 = 4,
    #[serde(rename = "CREDENTIAL_TYPE_EMAIL_AUTH_KEY_P256")]
    EmailAuthKeyP256 = 5,
    #[serde(rename = "CREDENTIAL_TYPE_API_KEY_ED25519")]
    ApiKeyEd25519 = 6,
    #[serde(rename = "CREDENTIAL_TYPE_OTP_AUTH_KEY_P256")]
    OtpAuthKeyP256 = 7,
    #[serde(rename = "CREDENTIAL_TYPE_READ_WRITE_SESSION_KEY_P256")]
    ReadWriteSessionKeyP256 = 8,
    #[serde(rename = "CREDENTIAL_TYPE_OAUTH_KEY_P256")]
    OauthKeyP256 = 9,
    #[serde(rename = "CREDENTIAL_TYPE_LOGIN")]
    Login = 10,
}
impl CredentialType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "CREDENTIAL_TYPE_UNSPECIFIED",
            Self::WebauthnAuthenticator => "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR",
            Self::ApiKeyP256 => "CREDENTIAL_TYPE_API_KEY_P256",
            Self::RecoverUserKeyP256 => "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256",
            Self::ApiKeySecp256k1 => "CREDENTIAL_TYPE_API_KEY_SECP256K1",
            Self::EmailAuthKeyP256 => "CREDENTIAL_TYPE_EMAIL_AUTH_KEY_P256",
            Self::ApiKeyEd25519 => "CREDENTIAL_TYPE_API_KEY_ED25519",
            Self::OtpAuthKeyP256 => "CREDENTIAL_TYPE_OTP_AUTH_KEY_P256",
            Self::ReadWriteSessionKeyP256 => {
                "CREDENTIAL_TYPE_READ_WRITE_SESSION_KEY_P256"
            }
            Self::OauthKeyP256 => "CREDENTIAL_TYPE_OAUTH_KEY_P256",
            Self::Login => "CREDENTIAL_TYPE_LOGIN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CREDENTIAL_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR" => Some(Self::WebauthnAuthenticator),
            "CREDENTIAL_TYPE_API_KEY_P256" => Some(Self::ApiKeyP256),
            "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256" => Some(Self::RecoverUserKeyP256),
            "CREDENTIAL_TYPE_API_KEY_SECP256K1" => Some(Self::ApiKeySecp256k1),
            "CREDENTIAL_TYPE_EMAIL_AUTH_KEY_P256" => Some(Self::EmailAuthKeyP256),
            "CREDENTIAL_TYPE_API_KEY_ED25519" => Some(Self::ApiKeyEd25519),
            "CREDENTIAL_TYPE_OTP_AUTH_KEY_P256" => Some(Self::OtpAuthKeyP256),
            "CREDENTIAL_TYPE_READ_WRITE_SESSION_KEY_P256" => {
                Some(Self::ReadWriteSessionKeyP256)
            }
            "CREDENTIAL_TYPE_OAUTH_KEY_P256" => Some(Self::OauthKeyP256),
            "CREDENTIAL_TYPE_LOGIN" => Some(Self::Login),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeatureName {
    #[serde(rename = "FEATURE_NAME_UNSPECIFIED")]
    Unspecified = 0,
    /// to be deprecated in favor of rename: `FEATURE_NAME_EMAIL_RECOVERY`
    #[serde(rename = "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY")]
    RootUserEmailRecovery = 1,
    #[serde(rename = "FEATURE_NAME_WEBAUTHN_ORIGINS")]
    WebauthnOrigins = 2,
    #[serde(rename = "FEATURE_NAME_EMAIL_AUTH")]
    EmailAuth = 3,
    #[serde(rename = "FEATURE_NAME_EMAIL_RECOVERY")]
    EmailRecovery = 4,
    #[serde(rename = "FEATURE_NAME_WEBHOOK")]
    Webhook = 5,
    #[serde(rename = "FEATURE_NAME_SMS_AUTH")]
    SmsAuth = 6,
    #[serde(rename = "FEATURE_NAME_OTP_EMAIL_AUTH")]
    OtpEmailAuth = 7,
    #[serde(rename = "FEATURE_NAME_AUTH_PROXY")]
    AuthProxy = 8,
}
impl FeatureName {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FEATURE_NAME_UNSPECIFIED",
            Self::RootUserEmailRecovery => "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY",
            Self::WebauthnOrigins => "FEATURE_NAME_WEBAUTHN_ORIGINS",
            Self::EmailAuth => "FEATURE_NAME_EMAIL_AUTH",
            Self::EmailRecovery => "FEATURE_NAME_EMAIL_RECOVERY",
            Self::Webhook => "FEATURE_NAME_WEBHOOK",
            Self::SmsAuth => "FEATURE_NAME_SMS_AUTH",
            Self::OtpEmailAuth => "FEATURE_NAME_OTP_EMAIL_AUTH",
            Self::AuthProxy => "FEATURE_NAME_AUTH_PROXY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FEATURE_NAME_UNSPECIFIED" => Some(Self::Unspecified),
            "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY" => Some(Self::RootUserEmailRecovery),
            "FEATURE_NAME_WEBAUTHN_ORIGINS" => Some(Self::WebauthnOrigins),
            "FEATURE_NAME_EMAIL_AUTH" => Some(Self::EmailAuth),
            "FEATURE_NAME_EMAIL_RECOVERY" => Some(Self::EmailRecovery),
            "FEATURE_NAME_WEBHOOK" => Some(Self::Webhook),
            "FEATURE_NAME_SMS_AUTH" => Some(Self::SmsAuth),
            "FEATURE_NAME_OTP_EMAIL_AUTH" => Some(Self::OtpEmailAuth),
            "FEATURE_NAME_AUTH_PROXY" => Some(Self::AuthProxy),
            _ => None,
        }
    }
}
/// The supported Fiat On Ramp Providers.
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiatOnRampProvider {
    #[serde(rename = "FIAT_ON_RAMP_PROVIDER_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "FIAT_ON_RAMP_PROVIDER_COINBASE")]
    Coinbase = 1,
    #[serde(rename = "FIAT_ON_RAMP_PROVIDER_MOONPAY")]
    Moonpay = 2,
}
impl FiatOnRampProvider {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIAT_ON_RAMP_PROVIDER_UNSPECIFIED",
            Self::Coinbase => "FIAT_ON_RAMP_PROVIDER_COINBASE",
            Self::Moonpay => "FIAT_ON_RAMP_PROVIDER_MOONPAY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIAT_ON_RAMP_PROVIDER_UNSPECIFIED" => Some(Self::Unspecified),
            "FIAT_ON_RAMP_PROVIDER_COINBASE" => Some(Self::Coinbase),
            "FIAT_ON_RAMP_PROVIDER_MOONPAY" => Some(Self::Moonpay),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiatOnRampCryptoCurrency {
    /// Unspecified
    #[serde(rename = "FIAT_ON_RAMP_CRYPTO_CURRENCY_UNSPECIFIED")]
    Unspecified = 0,
    /// Bitcoin
    #[serde(rename = "FIAT_ON_RAMP_CRYPTO_CURRENCY_BTC")]
    Btc = 1,
    /// Ethereum
    #[serde(rename = "FIAT_ON_RAMP_CRYPTO_CURRENCY_ETH")]
    Eth = 2,
    /// Solana
    #[serde(rename = "FIAT_ON_RAMP_CRYPTO_CURRENCY_SOL")]
    Sol = 3,
    /// USDC
    #[serde(rename = "FIAT_ON_RAMP_CRYPTO_CURRENCY_USDC")]
    Usdc = 4,
}
impl FiatOnRampCryptoCurrency {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIAT_ON_RAMP_CRYPTO_CURRENCY_UNSPECIFIED",
            Self::Btc => "FIAT_ON_RAMP_CRYPTO_CURRENCY_BTC",
            Self::Eth => "FIAT_ON_RAMP_CRYPTO_CURRENCY_ETH",
            Self::Sol => "FIAT_ON_RAMP_CRYPTO_CURRENCY_SOL",
            Self::Usdc => "FIAT_ON_RAMP_CRYPTO_CURRENCY_USDC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIAT_ON_RAMP_CRYPTO_CURRENCY_UNSPECIFIED" => Some(Self::Unspecified),
            "FIAT_ON_RAMP_CRYPTO_CURRENCY_BTC" => Some(Self::Btc),
            "FIAT_ON_RAMP_CRYPTO_CURRENCY_ETH" => Some(Self::Eth),
            "FIAT_ON_RAMP_CRYPTO_CURRENCY_SOL" => Some(Self::Sol),
            "FIAT_ON_RAMP_CRYPTO_CURRENCY_USDC" => Some(Self::Usdc),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiatOnRampCurrency {
    /// Unspecified
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_UNSPECIFIED")]
    Unspecified = 0,
    /// Australian Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_AUD")]
    Aud = 1,
    /// Bulgarian Lev
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_BGN")]
    Bgn = 2,
    /// Brazilian Real
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_BRL")]
    Brl = 3,
    /// Canadian Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_CAD")]
    Cad = 4,
    /// Swiss Franc
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_CHF")]
    Chf = 5,
    /// Colombian Peso
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_COP")]
    Cop = 6,
    /// Czech Koruna
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_CZK")]
    Czk = 7,
    /// Danish Krone
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_DKK")]
    Dkk = 8,
    /// Dominican Peso
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_DOP")]
    Dop = 9,
    /// Egyptian Pound
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_EGP")]
    Egp = 10,
    /// Euro
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_EUR")]
    Eur = 11,
    /// Pound Sterling
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_GBP")]
    Gbp = 12,
    /// Hong Kong Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_HKD")]
    Hkd = 13,
    /// Indonesian Rupiah
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_IDR")]
    Idr = 14,
    /// Israeli New Shekel
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_ILS")]
    Ils = 15,
    /// Jordanian Dinar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_JOD")]
    Jod = 16,
    /// Kenyan Shilling
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_KES")]
    Kes = 17,
    /// Kuwaiti Dinar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_KWD")]
    Kwd = 18,
    /// Sri Lankan Rupee
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_LKR")]
    Lkr = 19,
    /// Mexican Peso
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_MXN")]
    Mxn = 20,
    /// Nigerian Naira
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_NGN")]
    Ngn = 21,
    /// Norwegian Krone
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_NOK")]
    Nok = 22,
    /// New Zealand Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_NZD")]
    Nzd = 23,
    /// Omani Rial
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_OMR")]
    Omr = 24,
    /// Peruvian Sol
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_PEN")]
    Pen = 25,
    /// Polish Złoty
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_PLN")]
    Pln = 26,
    /// Romanian Leu
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_RON")]
    Ron = 27,
    /// Swedish Krona
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_SEK")]
    Sek = 28,
    /// Thai Baht
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_THB")]
    Thb = 29,
    /// Turkish Lira
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_TRY")]
    Try = 30,
    /// Taiwan Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_TWD")]
    Twd = 31,
    /// US Dollar
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_USD")]
    Usd = 32,
    /// Vietnamese Dong
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_VND")]
    Vnd = 33,
    /// South African Rand
    #[serde(rename = "FIAT_ON_RAMP_CURRENCY_ZAR")]
    Zar = 34,
}
impl FiatOnRampCurrency {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIAT_ON_RAMP_CURRENCY_UNSPECIFIED",
            Self::Aud => "FIAT_ON_RAMP_CURRENCY_AUD",
            Self::Bgn => "FIAT_ON_RAMP_CURRENCY_BGN",
            Self::Brl => "FIAT_ON_RAMP_CURRENCY_BRL",
            Self::Cad => "FIAT_ON_RAMP_CURRENCY_CAD",
            Self::Chf => "FIAT_ON_RAMP_CURRENCY_CHF",
            Self::Cop => "FIAT_ON_RAMP_CURRENCY_COP",
            Self::Czk => "FIAT_ON_RAMP_CURRENCY_CZK",
            Self::Dkk => "FIAT_ON_RAMP_CURRENCY_DKK",
            Self::Dop => "FIAT_ON_RAMP_CURRENCY_DOP",
            Self::Egp => "FIAT_ON_RAMP_CURRENCY_EGP",
            Self::Eur => "FIAT_ON_RAMP_CURRENCY_EUR",
            Self::Gbp => "FIAT_ON_RAMP_CURRENCY_GBP",
            Self::Hkd => "FIAT_ON_RAMP_CURRENCY_HKD",
            Self::Idr => "FIAT_ON_RAMP_CURRENCY_IDR",
            Self::Ils => "FIAT_ON_RAMP_CURRENCY_ILS",
            Self::Jod => "FIAT_ON_RAMP_CURRENCY_JOD",
            Self::Kes => "FIAT_ON_RAMP_CURRENCY_KES",
            Self::Kwd => "FIAT_ON_RAMP_CURRENCY_KWD",
            Self::Lkr => "FIAT_ON_RAMP_CURRENCY_LKR",
            Self::Mxn => "FIAT_ON_RAMP_CURRENCY_MXN",
            Self::Ngn => "FIAT_ON_RAMP_CURRENCY_NGN",
            Self::Nok => "FIAT_ON_RAMP_CURRENCY_NOK",
            Self::Nzd => "FIAT_ON_RAMP_CURRENCY_NZD",
            Self::Omr => "FIAT_ON_RAMP_CURRENCY_OMR",
            Self::Pen => "FIAT_ON_RAMP_CURRENCY_PEN",
            Self::Pln => "FIAT_ON_RAMP_CURRENCY_PLN",
            Self::Ron => "FIAT_ON_RAMP_CURRENCY_RON",
            Self::Sek => "FIAT_ON_RAMP_CURRENCY_SEK",
            Self::Thb => "FIAT_ON_RAMP_CURRENCY_THB",
            Self::Try => "FIAT_ON_RAMP_CURRENCY_TRY",
            Self::Twd => "FIAT_ON_RAMP_CURRENCY_TWD",
            Self::Usd => "FIAT_ON_RAMP_CURRENCY_USD",
            Self::Vnd => "FIAT_ON_RAMP_CURRENCY_VND",
            Self::Zar => "FIAT_ON_RAMP_CURRENCY_ZAR",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIAT_ON_RAMP_CURRENCY_UNSPECIFIED" => Some(Self::Unspecified),
            "FIAT_ON_RAMP_CURRENCY_AUD" => Some(Self::Aud),
            "FIAT_ON_RAMP_CURRENCY_BGN" => Some(Self::Bgn),
            "FIAT_ON_RAMP_CURRENCY_BRL" => Some(Self::Brl),
            "FIAT_ON_RAMP_CURRENCY_CAD" => Some(Self::Cad),
            "FIAT_ON_RAMP_CURRENCY_CHF" => Some(Self::Chf),
            "FIAT_ON_RAMP_CURRENCY_COP" => Some(Self::Cop),
            "FIAT_ON_RAMP_CURRENCY_CZK" => Some(Self::Czk),
            "FIAT_ON_RAMP_CURRENCY_DKK" => Some(Self::Dkk),
            "FIAT_ON_RAMP_CURRENCY_DOP" => Some(Self::Dop),
            "FIAT_ON_RAMP_CURRENCY_EGP" => Some(Self::Egp),
            "FIAT_ON_RAMP_CURRENCY_EUR" => Some(Self::Eur),
            "FIAT_ON_RAMP_CURRENCY_GBP" => Some(Self::Gbp),
            "FIAT_ON_RAMP_CURRENCY_HKD" => Some(Self::Hkd),
            "FIAT_ON_RAMP_CURRENCY_IDR" => Some(Self::Idr),
            "FIAT_ON_RAMP_CURRENCY_ILS" => Some(Self::Ils),
            "FIAT_ON_RAMP_CURRENCY_JOD" => Some(Self::Jod),
            "FIAT_ON_RAMP_CURRENCY_KES" => Some(Self::Kes),
            "FIAT_ON_RAMP_CURRENCY_KWD" => Some(Self::Kwd),
            "FIAT_ON_RAMP_CURRENCY_LKR" => Some(Self::Lkr),
            "FIAT_ON_RAMP_CURRENCY_MXN" => Some(Self::Mxn),
            "FIAT_ON_RAMP_CURRENCY_NGN" => Some(Self::Ngn),
            "FIAT_ON_RAMP_CURRENCY_NOK" => Some(Self::Nok),
            "FIAT_ON_RAMP_CURRENCY_NZD" => Some(Self::Nzd),
            "FIAT_ON_RAMP_CURRENCY_OMR" => Some(Self::Omr),
            "FIAT_ON_RAMP_CURRENCY_PEN" => Some(Self::Pen),
            "FIAT_ON_RAMP_CURRENCY_PLN" => Some(Self::Pln),
            "FIAT_ON_RAMP_CURRENCY_RON" => Some(Self::Ron),
            "FIAT_ON_RAMP_CURRENCY_SEK" => Some(Self::Sek),
            "FIAT_ON_RAMP_CURRENCY_THB" => Some(Self::Thb),
            "FIAT_ON_RAMP_CURRENCY_TRY" => Some(Self::Try),
            "FIAT_ON_RAMP_CURRENCY_TWD" => Some(Self::Twd),
            "FIAT_ON_RAMP_CURRENCY_USD" => Some(Self::Usd),
            "FIAT_ON_RAMP_CURRENCY_VND" => Some(Self::Vnd),
            "FIAT_ON_RAMP_CURRENCY_ZAR" => Some(Self::Zar),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiatOnRampBlockchainNetwork {
    /// Unspecified
    #[serde(rename = "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_UNSPECIFIED")]
    Unspecified = 0,
    /// bitcoin
    #[serde(rename = "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BITCOIN")]
    Bitcoin = 1,
    /// ethereum
    #[serde(rename = "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_ETHEREUM")]
    Ethereum = 2,
    /// solana
    #[serde(rename = "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_SOLANA")]
    Solana = 3,
    /// base
    #[serde(rename = "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BASE")]
    Base = 4,
}
impl FiatOnRampBlockchainNetwork {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_UNSPECIFIED",
            Self::Bitcoin => "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BITCOIN",
            Self::Ethereum => "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_ETHEREUM",
            Self::Solana => "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_SOLANA",
            Self::Base => "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BASE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_UNSPECIFIED" => Some(Self::Unspecified),
            "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BITCOIN" => Some(Self::Bitcoin),
            "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_ETHEREUM" => Some(Self::Ethereum),
            "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_SOLANA" => Some(Self::Solana),
            "FIAT_ON_RAMP_BLOCKCHAIN_NETWORK_BASE" => Some(Self::Base),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FiatOnRampPaymentMethod {
    /// Unspecified
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_UNSPECIFIED")]
    Unspecified = 0,
    /// Shared methods (supported by both MoonPay and Coinbase)
    ///
    /// MoonPay: CREDIT_DEBIT_CARD, Coinbase: CARD
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_CREDIT_DEBIT_CARD")]
    CreditDebitCard = 1,
    /// MoonPay: APPLE_PAY, Coinbase: APPLE_PAY
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_APPLE_PAY")]
    ApplePay = 2,
    /// MoonPay-specific methods
    ///
    /// MoonPay: GBP_BANK_TRANSFER
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_BANK_TRANSFER")]
    GbpBankTransfer = 3,
    /// MoonPay: GBP_OPEN_BANKING_PAYMENT
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_OPEN_BANKING_PAYMENT")]
    GbpOpenBankingPayment = 4,
    /// MoonPay: GOOGLE_PAY
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_GOOGLE_PAY")]
    GooglePay = 5,
    /// MoonPay: SEPA_BANK_TRANSFER
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_SEPA_BANK_TRANSFER")]
    SepaBankTransfer = 6,
    /// MoonPay: PIX_INSTANT_PAYMENT
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_PIX_INSTANT_PAYMENT")]
    PixInstantPayment = 7,
    /// MoonPay: PAYPAL
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_PAYPAL")]
    Paypal = 8,
    /// MoonPay: VENMO
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_VENMO")]
    Venmo = 9,
    /// MoonPay: MOONPAY_BALANCE
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_MOONPAY_BALANCE")]
    MoonpayBalance = 10,
    /// Coinbase-specific methods
    ///
    /// Coinbase: CRYPTO_ACCOUNT
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_CRYPTO_ACCOUNT")]
    CryptoAccount = 11,
    /// Coinbase: FIAT_WALLET
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_FIAT_WALLET")]
    FiatWallet = 12,
    /// Coinbase: ACH_BANK_ACCOUNT
    #[serde(rename = "FIAT_ON_RAMP_PAYMENT_METHOD_ACH_BANK_ACCOUNT")]
    AchBankAccount = 13,
}
impl FiatOnRampPaymentMethod {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "FIAT_ON_RAMP_PAYMENT_METHOD_UNSPECIFIED",
            Self::CreditDebitCard => "FIAT_ON_RAMP_PAYMENT_METHOD_CREDIT_DEBIT_CARD",
            Self::ApplePay => "FIAT_ON_RAMP_PAYMENT_METHOD_APPLE_PAY",
            Self::GbpBankTransfer => "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_BANK_TRANSFER",
            Self::GbpOpenBankingPayment => {
                "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_OPEN_BANKING_PAYMENT"
            }
            Self::GooglePay => "FIAT_ON_RAMP_PAYMENT_METHOD_GOOGLE_PAY",
            Self::SepaBankTransfer => "FIAT_ON_RAMP_PAYMENT_METHOD_SEPA_BANK_TRANSFER",
            Self::PixInstantPayment => "FIAT_ON_RAMP_PAYMENT_METHOD_PIX_INSTANT_PAYMENT",
            Self::Paypal => "FIAT_ON_RAMP_PAYMENT_METHOD_PAYPAL",
            Self::Venmo => "FIAT_ON_RAMP_PAYMENT_METHOD_VENMO",
            Self::MoonpayBalance => "FIAT_ON_RAMP_PAYMENT_METHOD_MOONPAY_BALANCE",
            Self::CryptoAccount => "FIAT_ON_RAMP_PAYMENT_METHOD_CRYPTO_ACCOUNT",
            Self::FiatWallet => "FIAT_ON_RAMP_PAYMENT_METHOD_FIAT_WALLET",
            Self::AchBankAccount => "FIAT_ON_RAMP_PAYMENT_METHOD_ACH_BANK_ACCOUNT",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FIAT_ON_RAMP_PAYMENT_METHOD_UNSPECIFIED" => Some(Self::Unspecified),
            "FIAT_ON_RAMP_PAYMENT_METHOD_CREDIT_DEBIT_CARD" => {
                Some(Self::CreditDebitCard)
            }
            "FIAT_ON_RAMP_PAYMENT_METHOD_APPLE_PAY" => Some(Self::ApplePay),
            "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_BANK_TRANSFER" => {
                Some(Self::GbpBankTransfer)
            }
            "FIAT_ON_RAMP_PAYMENT_METHOD_GBP_OPEN_BANKING_PAYMENT" => {
                Some(Self::GbpOpenBankingPayment)
            }
            "FIAT_ON_RAMP_PAYMENT_METHOD_GOOGLE_PAY" => Some(Self::GooglePay),
            "FIAT_ON_RAMP_PAYMENT_METHOD_SEPA_BANK_TRANSFER" => {
                Some(Self::SepaBankTransfer)
            }
            "FIAT_ON_RAMP_PAYMENT_METHOD_PIX_INSTANT_PAYMENT" => {
                Some(Self::PixInstantPayment)
            }
            "FIAT_ON_RAMP_PAYMENT_METHOD_PAYPAL" => Some(Self::Paypal),
            "FIAT_ON_RAMP_PAYMENT_METHOD_VENMO" => Some(Self::Venmo),
            "FIAT_ON_RAMP_PAYMENT_METHOD_MOONPAY_BALANCE" => Some(Self::MoonpayBalance),
            "FIAT_ON_RAMP_PAYMENT_METHOD_CRYPTO_ACCOUNT" => Some(Self::CryptoAccount),
            "FIAT_ON_RAMP_PAYMENT_METHOD_FIAT_WALLET" => Some(Self::FiatWallet),
            "FIAT_ON_RAMP_PAYMENT_METHOD_ACH_BANK_ACCOUNT" => Some(Self::AchBankAccount),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransactionType {
    #[serde(rename = "TRANSACTION_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    /// Unsigned Ethereum transaction, RLP-encoded and hex-encoded
    #[serde(rename = "TRANSACTION_TYPE_ETHEREUM")]
    Ethereum = 1,
    /// Unsigned Solana transaction in hex bytes
    #[serde(rename = "TRANSACTION_TYPE_SOLANA")]
    Solana = 2,
    /// Unsigned Tron transaction, protobuf encoded and hex encoded
    #[serde(rename = "TRANSACTION_TYPE_TRON")]
    Tron = 3,
    /// Unsigned Bitcoin transaction, hex encoded
    #[serde(rename = "TRANSACTION_TYPE_BITCOIN")]
    Bitcoin = 4,
}
impl TransactionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "TRANSACTION_TYPE_UNSPECIFIED",
            Self::Ethereum => "TRANSACTION_TYPE_ETHEREUM",
            Self::Solana => "TRANSACTION_TYPE_SOLANA",
            Self::Tron => "TRANSACTION_TYPE_TRON",
            Self::Bitcoin => "TRANSACTION_TYPE_BITCOIN",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TRANSACTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "TRANSACTION_TYPE_ETHEREUM" => Some(Self::Ethereum),
            "TRANSACTION_TYPE_SOLANA" => Some(Self::Solana),
            "TRANSACTION_TYPE_TRON" => Some(Self::Tron),
            "TRANSACTION_TYPE_BITCOIN" => Some(Self::Bitcoin),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SmartContractInterfaceType {
    #[serde(rename = "SMART_CONTRACT_INTERFACE_TYPE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "SMART_CONTRACT_INTERFACE_TYPE_ETHEREUM")]
    Ethereum = 1,
    #[serde(rename = "SMART_CONTRACT_INTERFACE_TYPE_SOLANA")]
    Solana = 2,
}
impl SmartContractInterfaceType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "SMART_CONTRACT_INTERFACE_TYPE_UNSPECIFIED",
            Self::Ethereum => "SMART_CONTRACT_INTERFACE_TYPE_ETHEREUM",
            Self::Solana => "SMART_CONTRACT_INTERFACE_TYPE_SOLANA",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SMART_CONTRACT_INTERFACE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "SMART_CONTRACT_INTERFACE_TYPE_ETHEREUM" => Some(Self::Ethereum),
            "SMART_CONTRACT_INTERFACE_TYPE_SOLANA" => Some(Self::Solana),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Outcome {
    #[serde(rename = "OUTCOME_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "OUTCOME_ALLOW")]
    Allow = 1,
    #[serde(rename = "OUTCOME_DENY_EXPLICIT")]
    DenyExplicit = 2,
    #[serde(rename = "OUTCOME_DENY_IMPLICIT")]
    DenyImplicit = 3,
    #[serde(rename = "OUTCOME_REQUIRES_CONSENSUS")]
    RequiresConsensus = 4,
    #[serde(rename = "OUTCOME_REJECTED")]
    Rejected = 5,
    #[serde(rename = "OUTCOME_ERROR")]
    Error = 6,
}
impl Outcome {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "OUTCOME_UNSPECIFIED",
            Self::Allow => "OUTCOME_ALLOW",
            Self::DenyExplicit => "OUTCOME_DENY_EXPLICIT",
            Self::DenyImplicit => "OUTCOME_DENY_IMPLICIT",
            Self::RequiresConsensus => "OUTCOME_REQUIRES_CONSENSUS",
            Self::Rejected => "OUTCOME_REJECTED",
            Self::Error => "OUTCOME_ERROR",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OUTCOME_UNSPECIFIED" => Some(Self::Unspecified),
            "OUTCOME_ALLOW" => Some(Self::Allow),
            "OUTCOME_DENY_EXPLICIT" => Some(Self::DenyExplicit),
            "OUTCOME_DENY_IMPLICIT" => Some(Self::DenyImplicit),
            "OUTCOME_REQUIRES_CONSENSUS" => Some(Self::RequiresConsensus),
            "OUTCOME_REJECTED" => Some(Self::Rejected),
            "OUTCOME_ERROR" => Some(Self::Error),
            _ => None,
        }
    }
}
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operator {
    #[serde(rename = "OPERATOR_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "OPERATOR_EQUAL")]
    Equal = 1,
    #[serde(rename = "OPERATOR_MORE_THAN")]
    MoreThan = 2,
    #[serde(rename = "OPERATOR_MORE_THAN_OR_EQUAL")]
    MoreThanOrEqual = 3,
    #[serde(rename = "OPERATOR_LESS_THAN")]
    LessThan = 4,
    #[serde(rename = "OPERATOR_LESS_THAN_OR_EQUAL")]
    LessThanOrEqual = 5,
    #[serde(rename = "OPERATOR_CONTAINS")]
    Contains = 6,
    #[serde(rename = "OPERATOR_NOT_EQUAL")]
    NotEqual = 7,
    #[serde(rename = "OPERATOR_IN")]
    In = 8,
    #[serde(rename = "OPERATOR_NOT_IN")]
    NotIn = 9,
    #[serde(rename = "OPERATOR_CONTAINS_ONE")]
    ContainsOne = 10,
    #[serde(rename = "OPERATOR_CONTAINS_ALL")]
    ContainsAll = 11,
}
impl Operator {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "OPERATOR_UNSPECIFIED",
            Self::Equal => "OPERATOR_EQUAL",
            Self::MoreThan => "OPERATOR_MORE_THAN",
            Self::MoreThanOrEqual => "OPERATOR_MORE_THAN_OR_EQUAL",
            Self::LessThan => "OPERATOR_LESS_THAN",
            Self::LessThanOrEqual => "OPERATOR_LESS_THAN_OR_EQUAL",
            Self::Contains => "OPERATOR_CONTAINS",
            Self::NotEqual => "OPERATOR_NOT_EQUAL",
            Self::In => "OPERATOR_IN",
            Self::NotIn => "OPERATOR_NOT_IN",
            Self::ContainsOne => "OPERATOR_CONTAINS_ONE",
            Self::ContainsAll => "OPERATOR_CONTAINS_ALL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OPERATOR_UNSPECIFIED" => Some(Self::Unspecified),
            "OPERATOR_EQUAL" => Some(Self::Equal),
            "OPERATOR_MORE_THAN" => Some(Self::MoreThan),
            "OPERATOR_MORE_THAN_OR_EQUAL" => Some(Self::MoreThanOrEqual),
            "OPERATOR_LESS_THAN" => Some(Self::LessThan),
            "OPERATOR_LESS_THAN_OR_EQUAL" => Some(Self::LessThanOrEqual),
            "OPERATOR_CONTAINS" => Some(Self::Contains),
            "OPERATOR_NOT_EQUAL" => Some(Self::NotEqual),
            "OPERATOR_IN" => Some(Self::In),
            "OPERATOR_NOT_IN" => Some(Self::NotIn),
            "OPERATOR_CONTAINS_ONE" => Some(Self::ContainsOne),
            "OPERATOR_CONTAINS_ALL" => Some(Self::ContainsAll),
            _ => None,
        }
    }
}
/// A list of OAuth 2.0 providers that are supported
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Oauth2Provider {
    #[serde(rename = "OAUTH2_PROVIDER_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "OAUTH2_PROVIDER_X")]
    X = 1,
    #[serde(rename = "OAUTH2_PROVIDER_DISCORD")]
    Discord = 2,
}
impl Oauth2Provider {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "OAUTH2_PROVIDER_UNSPECIFIED",
            Self::X => "OAUTH2_PROVIDER_X",
            Self::Discord => "OAUTH2_PROVIDER_DISCORD",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "OAUTH2_PROVIDER_UNSPECIFIED" => Some(Self::Unspecified),
            "OAUTH2_PROVIDER_X" => Some(Self::X),
            "OAUTH2_PROVIDER_DISCORD" => Some(Self::Discord),
            _ => None,
        }
    }
}
/// The current stage of a TVC deployment
/// (note: leaving some space in the numbering to account for potential future stages)
#[derive(::serde::Serialize, ::serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TvcDeploymentStage {
    #[serde(rename = "TVC_DEPLOYMENT_STAGE_UNSPECIFIED")]
    Unspecified = 0,
    #[serde(rename = "TVC_DEPLOYMENT_STAGE_APPROVE")]
    Approve = 10,
    #[serde(rename = "TVC_DEPLOYMENT_STAGE_PROVISION")]
    Provision = 20,
    #[serde(rename = "TVC_DEPLOYMENT_STAGE_LIVE")]
    Live = 30,
    #[serde(rename = "TVC_DEPLOYMENT_STAGE_DELETE")]
    Delete = 40,
}
impl TvcDeploymentStage {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Unspecified => "TVC_DEPLOYMENT_STAGE_UNSPECIFIED",
            Self::Approve => "TVC_DEPLOYMENT_STAGE_APPROVE",
            Self::Provision => "TVC_DEPLOYMENT_STAGE_PROVISION",
            Self::Live => "TVC_DEPLOYMENT_STAGE_LIVE",
            Self::Delete => "TVC_DEPLOYMENT_STAGE_DELETE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TVC_DEPLOYMENT_STAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "TVC_DEPLOYMENT_STAGE_APPROVE" => Some(Self::Approve),
            "TVC_DEPLOYMENT_STAGE_PROVISION" => Some(Self::Provision),
            "TVC_DEPLOYMENT_STAGE_LIVE" => Some(Self::Live),
            "TVC_DEPLOYMENT_STAGE_DELETE" => Some(Self::Delete),
            _ => None,
        }
    }
}
