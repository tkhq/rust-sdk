impl serde::Serialize for AuthenticatorType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "AUTHENTICATOR_TYPE_UNKNOWN",
            Self::CrossPlatform => "CROSS_PLATFORM",
            Self::Platform => "PLATFORM",
            Self::Unspecified => "UNSPECIFIED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AUTHENTICATOR_TYPE_UNKNOWN",
            "CROSS_PLATFORM",
            "PLATFORM",
            "UNSPECIFIED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorType;

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
                    "AUTHENTICATOR_TYPE_UNKNOWN" => Ok(AuthenticatorType::Unknown),
                    "CROSS_PLATFORM" => Ok(AuthenticatorType::CrossPlatform),
                    "PLATFORM" => Ok(AuthenticatorType::Platform),
                    "UNSPECIFIED" => Ok(AuthenticatorType::Unspecified),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for WebAuthnStamp {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("external.webauthn.v1.WebAuthnStamp", len)?;
        if true {
            struct_ser.serialize_field("credentialId", &self.credential_id)?;
        }
        if true {
            struct_ser.serialize_field("clientDataJson", &self.client_data_json)?;
        }
        if true {
            struct_ser.serialize_field("authenticatorData", &self.authenticator_data)?;
        }
        if true {
            struct_ser.serialize_field("signature", &self.signature)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WebAuthnStamp {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credential_id",
            "credentialId",
            "client_data_json",
            "clientDataJson",
            "authenticator_data",
            "authenticatorData",
            "signature",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CredentialId,
            ClientDataJson,
            AuthenticatorData,
            Signature,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "credentialId" | "credential_id" => Ok(GeneratedField::CredentialId),
                            "clientDataJson" | "client_data_json" => Ok(GeneratedField::ClientDataJson),
                            "authenticatorData" | "authenticator_data" => Ok(GeneratedField::AuthenticatorData),
                            "signature" => Ok(GeneratedField::Signature),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WebAuthnStamp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.webauthn.v1.WebAuthnStamp")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WebAuthnStamp, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut credential_id__ = None;
                let mut client_data_json__ = None;
                let mut authenticator_data__ = None;
                let mut signature__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CredentialId => {
                            if credential_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialId"));
                            }
                            credential_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ClientDataJson => {
                            if client_data_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientDataJson"));
                            }
                            client_data_json__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthenticatorData => {
                            if authenticator_data__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorData"));
                            }
                            authenticator_data__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Signature => {
                            if signature__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signature"));
                            }
                            signature__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(WebAuthnStamp {
                    credential_id: credential_id__.unwrap_or_default(),
                    client_data_json: client_data_json__.unwrap_or_default(),
                    authenticator_data: authenticator_data__.unwrap_or_default(),
                    signature: signature__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.webauthn.v1.WebAuthnStamp", FIELDS, GeneratedVisitor)
    }
}
