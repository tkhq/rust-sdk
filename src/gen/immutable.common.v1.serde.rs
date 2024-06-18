impl serde::Serialize for AccessType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ACCESS_TYPE_UNSPECIFIED",
            Self::Web => "ACCESS_TYPE_WEB",
            Self::Api => "ACCESS_TYPE_API",
            Self::All => "ACCESS_TYPE_ALL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AccessType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ACCESS_TYPE_UNSPECIFIED",
            "ACCESS_TYPE_WEB",
            "ACCESS_TYPE_API",
            "ACCESS_TYPE_ALL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AccessType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ACCESS_TYPE_UNSPECIFIED" => Ok(AccessType::Unspecified),
                    "ACCESS_TYPE_WEB" => Ok(AccessType::Web),
                    "ACCESS_TYPE_API" => Ok(AccessType::Api),
                    "ACCESS_TYPE_ALL" => Ok(AccessType::All),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for AddressFormat {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ADDRESS_FORMAT_UNSPECIFIED",
            Self::Uncompressed => "ADDRESS_FORMAT_UNCOMPRESSED",
            Self::Compressed => "ADDRESS_FORMAT_COMPRESSED",
            Self::Ethereum => "ADDRESS_FORMAT_ETHEREUM",
            Self::Solana => "ADDRESS_FORMAT_SOLANA",
            Self::Cosmos => "ADDRESS_FORMAT_COSMOS",
            Self::Tron => "ADDRESS_FORMAT_TRON",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AddressFormat {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ADDRESS_FORMAT_UNSPECIFIED",
            "ADDRESS_FORMAT_UNCOMPRESSED",
            "ADDRESS_FORMAT_COMPRESSED",
            "ADDRESS_FORMAT_ETHEREUM",
            "ADDRESS_FORMAT_SOLANA",
            "ADDRESS_FORMAT_COSMOS",
            "ADDRESS_FORMAT_TRON",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AddressFormat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ADDRESS_FORMAT_UNSPECIFIED" => Ok(AddressFormat::Unspecified),
                    "ADDRESS_FORMAT_UNCOMPRESSED" => Ok(AddressFormat::Uncompressed),
                    "ADDRESS_FORMAT_COMPRESSED" => Ok(AddressFormat::Compressed),
                    "ADDRESS_FORMAT_ETHEREUM" => Ok(AddressFormat::Ethereum),
                    "ADDRESS_FORMAT_SOLANA" => Ok(AddressFormat::Solana),
                    "ADDRESS_FORMAT_COSMOS" => Ok(AddressFormat::Cosmos),
                    "ADDRESS_FORMAT_TRON" => Ok(AddressFormat::Tron),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CredentialType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CREDENTIAL_TYPE_UNSPECIFIED",
            Self::WebauthnAuthenticator => "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR",
            Self::ApiKeyP256 => "CREDENTIAL_TYPE_API_KEY_P256",
            Self::RecoverUserKeyP256 => "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for CredentialType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CREDENTIAL_TYPE_UNSPECIFIED",
            "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR",
            "CREDENTIAL_TYPE_API_KEY_P256",
            "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CredentialType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "CREDENTIAL_TYPE_UNSPECIFIED" => Ok(CredentialType::Unspecified),
                    "CREDENTIAL_TYPE_WEBAUTHN_AUTHENTICATOR" => Ok(CredentialType::WebauthnAuthenticator),
                    "CREDENTIAL_TYPE_API_KEY_P256" => Ok(CredentialType::ApiKeyP256),
                    "CREDENTIAL_TYPE_RECOVER_USER_KEY_P256" => Ok(CredentialType::RecoverUserKeyP256),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Curve {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "CURVE_UNSPECIFIED",
            Self::Secp256k1 => "CURVE_SECP256K1",
            Self::Ed25519 => "CURVE_ED25519",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Curve {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "CURVE_UNSPECIFIED",
            "CURVE_SECP256K1",
            "CURVE_ED25519",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Curve;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "CURVE_UNSPECIFIED" => Ok(Curve::Unspecified),
                    "CURVE_SECP256K1" => Ok(Curve::Secp256k1),
                    "CURVE_ED25519" => Ok(Curve::Ed25519),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Effect {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "EFFECT_UNSPECIFIED",
            Self::Allow => "EFFECT_ALLOW",
            Self::Deny => "EFFECT_DENY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Effect {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EFFECT_UNSPECIFIED",
            "EFFECT_ALLOW",
            "EFFECT_DENY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Effect;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EFFECT_UNSPECIFIED" => Ok(Effect::Unspecified),
                    "EFFECT_ALLOW" => Ok(Effect::Allow),
                    "EFFECT_DENY" => Ok(Effect::Deny),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for FeatureName {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "FEATURE_NAME_UNSPECIFIED",
            Self::RootUserEmailRecovery => "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY",
            Self::WebauthnOrigins => "FEATURE_NAME_WEBAUTHN_ORIGINS",
            Self::EmailAuth => "FEATURE_NAME_EMAIL_AUTH",
            Self::EmailRecovery => "FEATURE_NAME_EMAIL_RECOVERY",
            Self::Webhook => "FEATURE_NAME_WEBHOOK",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for FeatureName {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FEATURE_NAME_UNSPECIFIED",
            "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY",
            "FEATURE_NAME_WEBAUTHN_ORIGINS",
            "FEATURE_NAME_EMAIL_AUTH",
            "FEATURE_NAME_EMAIL_RECOVERY",
            "FEATURE_NAME_WEBHOOK",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FeatureName;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "FEATURE_NAME_UNSPECIFIED" => Ok(FeatureName::Unspecified),
                    "FEATURE_NAME_ROOT_USER_EMAIL_RECOVERY" => Ok(FeatureName::RootUserEmailRecovery),
                    "FEATURE_NAME_WEBAUTHN_ORIGINS" => Ok(FeatureName::WebauthnOrigins),
                    "FEATURE_NAME_EMAIL_AUTH" => Ok(FeatureName::EmailAuth),
                    "FEATURE_NAME_EMAIL_RECOVERY" => Ok(FeatureName::EmailRecovery),
                    "FEATURE_NAME_WEBHOOK" => Ok(FeatureName::Webhook),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for HashFunction {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "HASH_FUNCTION_UNSPECIFIED",
            Self::NoOp => "HASH_FUNCTION_NO_OP",
            Self::Sha256 => "HASH_FUNCTION_SHA256",
            Self::Keccak256 => "HASH_FUNCTION_KECCAK256",
            Self::NotApplicable => "HASH_FUNCTION_NOT_APPLICABLE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for HashFunction {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "HASH_FUNCTION_UNSPECIFIED",
            "HASH_FUNCTION_NO_OP",
            "HASH_FUNCTION_SHA256",
            "HASH_FUNCTION_KECCAK256",
            "HASH_FUNCTION_NOT_APPLICABLE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HashFunction;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "HASH_FUNCTION_UNSPECIFIED" => Ok(HashFunction::Unspecified),
                    "HASH_FUNCTION_NO_OP" => Ok(HashFunction::NoOp),
                    "HASH_FUNCTION_SHA256" => Ok(HashFunction::Sha256),
                    "HASH_FUNCTION_KECCAK256" => Ok(HashFunction::Keccak256),
                    "HASH_FUNCTION_NOT_APPLICABLE" => Ok(HashFunction::NotApplicable),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for MnemonicLanguage {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
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
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for MnemonicLanguage {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "MNEMONIC_LANGUAGE_UNSPECIFIED",
            "MNEMONIC_LANGUAGE_ENGLISH",
            "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE",
            "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE",
            "MNEMONIC_LANGUAGE_CZECH",
            "MNEMONIC_LANGUAGE_FRENCH",
            "MNEMONIC_LANGUAGE_ITALIAN",
            "MNEMONIC_LANGUAGE_JAPANESE",
            "MNEMONIC_LANGUAGE_KOREAN",
            "MNEMONIC_LANGUAGE_SPANISH",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MnemonicLanguage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "MNEMONIC_LANGUAGE_UNSPECIFIED" => Ok(MnemonicLanguage::Unspecified),
                    "MNEMONIC_LANGUAGE_ENGLISH" => Ok(MnemonicLanguage::English),
                    "MNEMONIC_LANGUAGE_SIMPLIFIED_CHINESE" => Ok(MnemonicLanguage::SimplifiedChinese),
                    "MNEMONIC_LANGUAGE_TRADITIONAL_CHINESE" => Ok(MnemonicLanguage::TraditionalChinese),
                    "MNEMONIC_LANGUAGE_CZECH" => Ok(MnemonicLanguage::Czech),
                    "MNEMONIC_LANGUAGE_FRENCH" => Ok(MnemonicLanguage::French),
                    "MNEMONIC_LANGUAGE_ITALIAN" => Ok(MnemonicLanguage::Italian),
                    "MNEMONIC_LANGUAGE_JAPANESE" => Ok(MnemonicLanguage::Japanese),
                    "MNEMONIC_LANGUAGE_KOREAN" => Ok(MnemonicLanguage::Korean),
                    "MNEMONIC_LANGUAGE_SPANISH" => Ok(MnemonicLanguage::Spanish),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Operator {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
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
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Operator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "OPERATOR_UNSPECIFIED",
            "OPERATOR_EQUAL",
            "OPERATOR_MORE_THAN",
            "OPERATOR_MORE_THAN_OR_EQUAL",
            "OPERATOR_LESS_THAN",
            "OPERATOR_LESS_THAN_OR_EQUAL",
            "OPERATOR_CONTAINS",
            "OPERATOR_NOT_EQUAL",
            "OPERATOR_IN",
            "OPERATOR_NOT_IN",
            "OPERATOR_CONTAINS_ONE",
            "OPERATOR_CONTAINS_ALL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Operator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "OPERATOR_UNSPECIFIED" => Ok(Operator::Unspecified),
                    "OPERATOR_EQUAL" => Ok(Operator::Equal),
                    "OPERATOR_MORE_THAN" => Ok(Operator::MoreThan),
                    "OPERATOR_MORE_THAN_OR_EQUAL" => Ok(Operator::MoreThanOrEqual),
                    "OPERATOR_LESS_THAN" => Ok(Operator::LessThan),
                    "OPERATOR_LESS_THAN_OR_EQUAL" => Ok(Operator::LessThanOrEqual),
                    "OPERATOR_CONTAINS" => Ok(Operator::Contains),
                    "OPERATOR_NOT_EQUAL" => Ok(Operator::NotEqual),
                    "OPERATOR_IN" => Ok(Operator::In),
                    "OPERATOR_NOT_IN" => Ok(Operator::NotIn),
                    "OPERATOR_CONTAINS_ONE" => Ok(Operator::ContainsOne),
                    "OPERATOR_CONTAINS_ALL" => Ok(Operator::ContainsAll),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Outcome {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "OUTCOME_UNSPECIFIED",
            Self::Allow => "OUTCOME_ALLOW",
            Self::DenyExplicit => "OUTCOME_DENY_EXPLICIT",
            Self::DenyImplicit => "OUTCOME_DENY_IMPLICIT",
            Self::RequiresConsensus => "OUTCOME_REQUIRES_CONSENSUS",
            Self::Rejected => "OUTCOME_REJECTED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Outcome {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "OUTCOME_UNSPECIFIED",
            "OUTCOME_ALLOW",
            "OUTCOME_DENY_EXPLICIT",
            "OUTCOME_DENY_IMPLICIT",
            "OUTCOME_REQUIRES_CONSENSUS",
            "OUTCOME_REJECTED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Outcome;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "OUTCOME_UNSPECIFIED" => Ok(Outcome::Unspecified),
                    "OUTCOME_ALLOW" => Ok(Outcome::Allow),
                    "OUTCOME_DENY_EXPLICIT" => Ok(Outcome::DenyExplicit),
                    "OUTCOME_DENY_IMPLICIT" => Ok(Outcome::DenyImplicit),
                    "OUTCOME_REQUIRES_CONSENSUS" => Ok(Outcome::RequiresConsensus),
                    "OUTCOME_REJECTED" => Ok(Outcome::Rejected),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for PathFormat {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PATH_FORMAT_UNSPECIFIED",
            Self::Bip32 => "PATH_FORMAT_BIP32",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for PathFormat {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PATH_FORMAT_UNSPECIFIED",
            "PATH_FORMAT_BIP32",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PathFormat;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "PATH_FORMAT_UNSPECIFIED" => Ok(PathFormat::Unspecified),
                    "PATH_FORMAT_BIP32" => Ok(PathFormat::Bip32),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for PayloadEncoding {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PAYLOAD_ENCODING_UNSPECIFIED",
            Self::Hexadecimal => "PAYLOAD_ENCODING_HEXADECIMAL",
            Self::TextUtf8 => "PAYLOAD_ENCODING_TEXT_UTF8",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for PayloadEncoding {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PAYLOAD_ENCODING_UNSPECIFIED",
            "PAYLOAD_ENCODING_HEXADECIMAL",
            "PAYLOAD_ENCODING_TEXT_UTF8",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PayloadEncoding;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "PAYLOAD_ENCODING_UNSPECIFIED" => Ok(PayloadEncoding::Unspecified),
                    "PAYLOAD_ENCODING_HEXADECIMAL" => Ok(PayloadEncoding::Hexadecimal),
                    "PAYLOAD_ENCODING_TEXT_UTF8" => Ok(PayloadEncoding::TextUtf8),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for TransactionType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TRANSACTION_TYPE_UNSPECIFIED",
            Self::Ethereum => "TRANSACTION_TYPE_ETHEREUM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for TransactionType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TRANSACTION_TYPE_UNSPECIFIED",
            "TRANSACTION_TYPE_ETHEREUM",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TransactionType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "TRANSACTION_TYPE_UNSPECIFIED" => Ok(TransactionType::Unspecified),
                    "TRANSACTION_TYPE_ETHEREUM" => Ok(TransactionType::Ethereum),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
