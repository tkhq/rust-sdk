impl serde::Serialize for AuthenticatorAssertionResponse {
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
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.AuthenticatorAssertionResponse", len)?;
        if true {
            struct_ser.serialize_field("clientDataJSON", &self.client_data_json)?;
        }
        if true {
            struct_ser.serialize_field("authenticatorData", &self.authenticator_data)?;
        }
        if true {
            struct_ser.serialize_field("signature", &self.signature)?;
        }
        if let Some(v) = self.user_handle.as_ref() {
            struct_ser.serialize_field("userHandle", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorAssertionResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_data_JSON",
            "clientDataJSON",
            "authenticator_data",
            "authenticatorData",
            "signature",
            "user_handle",
            "userHandle",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientDataJson,
            AuthenticatorData,
            Signature,
            UserHandle,
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
                            "clientDataJSON" | "client_data_JSON" => Ok(GeneratedField::ClientDataJson),
                            "authenticatorData" | "authenticator_data" => Ok(GeneratedField::AuthenticatorData),
                            "signature" => Ok(GeneratedField::Signature),
                            "userHandle" | "user_handle" => Ok(GeneratedField::UserHandle),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorAssertionResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.AuthenticatorAssertionResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthenticatorAssertionResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_data_json__ = None;
                let mut authenticator_data__ = None;
                let mut signature__ = None;
                let mut user_handle__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientDataJson => {
                            if client_data_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientDataJSON"));
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
                        GeneratedField::UserHandle => {
                            if user_handle__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userHandle"));
                            }
                            user_handle__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AuthenticatorAssertionResponse {
                    client_data_json: client_data_json__.unwrap_or_default(),
                    authenticator_data: authenticator_data__.unwrap_or_default(),
                    signature: signature__.unwrap_or_default(),
                    user_handle: user_handle__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.AuthenticatorAssertionResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthenticatorAttestationResponse {
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
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.AuthenticatorAttestationResponse", len)?;
        if true {
            struct_ser.serialize_field("clientDataJson", &self.client_data_json)?;
        }
        if true {
            struct_ser.serialize_field("attestationObject", &self.attestation_object)?;
        }
        if true {
            let v = self.transports.iter().cloned().map(|v| {
                AuthenticatorTransport::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("transports", &v)?;
        }
        if let Some(v) = self.authenticator_attachment.as_ref() {
            struct_ser.serialize_field("authenticatorAttachment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorAttestationResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "client_data_json",
            "clientDataJson",
            "attestation_object",
            "attestationObject",
            "transports",
            "authenticator_attachment",
            "authenticatorAttachment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ClientDataJson,
            AttestationObject,
            Transports,
            AuthenticatorAttachment,
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
                            "clientDataJson" | "client_data_json" => Ok(GeneratedField::ClientDataJson),
                            "attestationObject" | "attestation_object" => Ok(GeneratedField::AttestationObject),
                            "transports" => Ok(GeneratedField::Transports),
                            "authenticatorAttachment" | "authenticator_attachment" => Ok(GeneratedField::AuthenticatorAttachment),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorAttestationResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.AuthenticatorAttestationResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AuthenticatorAttestationResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut client_data_json__ = None;
                let mut attestation_object__ = None;
                let mut transports__ = None;
                let mut authenticator_attachment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ClientDataJson => {
                            if client_data_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientDataJson"));
                            }
                            client_data_json__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AttestationObject => {
                            if attestation_object__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attestationObject"));
                            }
                            attestation_object__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Transports => {
                            if transports__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transports"));
                            }
                            transports__ = Some(map_.next_value::<Vec<AuthenticatorTransport>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::AuthenticatorAttachment => {
                            if authenticator_attachment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorAttachment"));
                            }
                            authenticator_attachment__ = map_.next_value()?;
                        }
                    }
                }
                Ok(AuthenticatorAttestationResponse {
                    client_data_json: client_data_json__.unwrap_or_default(),
                    attestation_object: attestation_object__.unwrap_or_default(),
                    transports: transports__.unwrap_or_default(),
                    authenticator_attachment: authenticator_attachment__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.AuthenticatorAttestationResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthenticatorTransport {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "AUTHENTICATOR_TRANSPORT_UNSPECIFIED",
            Self::Ble => "AUTHENTICATOR_TRANSPORT_BLE",
            Self::Internal => "AUTHENTICATOR_TRANSPORT_INTERNAL",
            Self::Nfc => "AUTHENTICATOR_TRANSPORT_NFC",
            Self::Usb => "AUTHENTICATOR_TRANSPORT_USB",
            Self::Hybrid => "AUTHENTICATOR_TRANSPORT_HYBRID",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AuthenticatorTransport {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AUTHENTICATOR_TRANSPORT_UNSPECIFIED",
            "AUTHENTICATOR_TRANSPORT_BLE",
            "AUTHENTICATOR_TRANSPORT_INTERNAL",
            "AUTHENTICATOR_TRANSPORT_NFC",
            "AUTHENTICATOR_TRANSPORT_USB",
            "AUTHENTICATOR_TRANSPORT_HYBRID",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthenticatorTransport;

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
                    "AUTHENTICATOR_TRANSPORT_UNSPECIFIED" => Ok(AuthenticatorTransport::Unspecified),
                    "AUTHENTICATOR_TRANSPORT_BLE" => Ok(AuthenticatorTransport::Ble),
                    "AUTHENTICATOR_TRANSPORT_INTERNAL" => Ok(AuthenticatorTransport::Internal),
                    "AUTHENTICATOR_TRANSPORT_NFC" => Ok(AuthenticatorTransport::Nfc),
                    "AUTHENTICATOR_TRANSPORT_USB" => Ok(AuthenticatorTransport::Usb),
                    "AUTHENTICATOR_TRANSPORT_HYBRID" => Ok(AuthenticatorTransport::Hybrid),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CredPropsAuthenticationExtensionsClientOutputs {
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
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.CredPropsAuthenticationExtensionsClientOutputs", len)?;
        if true {
            struct_ser.serialize_field("rk", &self.rk)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CredPropsAuthenticationExtensionsClientOutputs {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rk",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rk,
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
                            "rk" => Ok(GeneratedField::Rk),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CredPropsAuthenticationExtensionsClientOutputs;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.CredPropsAuthenticationExtensionsClientOutputs")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CredPropsAuthenticationExtensionsClientOutputs, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rk__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Rk => {
                            if rk__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rk"));
                            }
                            rk__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CredPropsAuthenticationExtensionsClientOutputs {
                    rk: rk__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.CredPropsAuthenticationExtensionsClientOutputs", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PublicKeyCredentialDescriptor {
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
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.PublicKeyCredentialDescriptor", len)?;
        if true {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if true {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if true {
            let v = self.transports.iter().cloned().map(|v| {
                AuthenticatorTransport::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("transports", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PublicKeyCredentialDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "id",
            "transports",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Id,
            Transports,
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
                            "type" => Ok(GeneratedField::Type),
                            "id" => Ok(GeneratedField::Id),
                            "transports" => Ok(GeneratedField::Transports),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PublicKeyCredentialDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.PublicKeyCredentialDescriptor")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PublicKeyCredentialDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut id__ = None;
                let mut transports__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Transports => {
                            if transports__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transports"));
                            }
                            transports__ = Some(map_.next_value::<Vec<AuthenticatorTransport>>()?.into_iter().map(|x| x as i32).collect());
                        }
                    }
                }
                Ok(PublicKeyCredentialDescriptor {
                    r#type: r#type__.unwrap_or_default(),
                    id: id__.unwrap_or_default(),
                    transports: transports__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.PublicKeyCredentialDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PublicKeyCredentialWithAssertion {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.PublicKeyCredentialWithAssertion", len)?;
        if true {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if true {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if true {
            struct_ser.serialize_field("rawId", &self.raw_id)?;
        }
        if let Some(v) = self.authenticator_attachment.as_ref() {
            struct_ser.serialize_field("authenticatorAttachment", v)?;
        }
        if let Some(v) = self.response.as_ref() {
            struct_ser.serialize_field("response", v)?;
        }
        if let Some(v) = self.client_extension_results.as_ref() {
            struct_ser.serialize_field("clientExtensionResults", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PublicKeyCredentialWithAssertion {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "type",
            "raw_id",
            "rawId",
            "authenticator_attachment",
            "authenticatorAttachment",
            "response",
            "client_extension_results",
            "clientExtensionResults",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Type,
            RawId,
            AuthenticatorAttachment,
            Response,
            ClientExtensionResults,
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
                            "id" => Ok(GeneratedField::Id),
                            "type" => Ok(GeneratedField::Type),
                            "rawId" | "raw_id" => Ok(GeneratedField::RawId),
                            "authenticatorAttachment" | "authenticator_attachment" => Ok(GeneratedField::AuthenticatorAttachment),
                            "response" => Ok(GeneratedField::Response),
                            "clientExtensionResults" | "client_extension_results" => Ok(GeneratedField::ClientExtensionResults),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PublicKeyCredentialWithAssertion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.PublicKeyCredentialWithAssertion")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PublicKeyCredentialWithAssertion, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut r#type__ = None;
                let mut raw_id__ = None;
                let mut authenticator_attachment__ = None;
                let mut response__ = None;
                let mut client_extension_results__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RawId => {
                            if raw_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawId"));
                            }
                            raw_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthenticatorAttachment => {
                            if authenticator_attachment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorAttachment"));
                            }
                            authenticator_attachment__ = map_.next_value()?;
                        }
                        GeneratedField::Response => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("response"));
                            }
                            response__ = map_.next_value()?;
                        }
                        GeneratedField::ClientExtensionResults => {
                            if client_extension_results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientExtensionResults"));
                            }
                            client_extension_results__ = map_.next_value()?;
                        }
                    }
                }
                Ok(PublicKeyCredentialWithAssertion {
                    id: id__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    raw_id: raw_id__.unwrap_or_default(),
                    authenticator_attachment: authenticator_attachment__,
                    response: response__,
                    client_extension_results: client_extension_results__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.PublicKeyCredentialWithAssertion", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PublicKeyCredentialWithAttestation {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.PublicKeyCredentialWithAttestation", len)?;
        if true {
            struct_ser.serialize_field("id", &self.id)?;
        }
        if true {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if true {
            struct_ser.serialize_field("rawId", &self.raw_id)?;
        }
        if let Some(v) = self.authenticator_attachment.as_ref() {
            struct_ser.serialize_field("authenticatorAttachment", v)?;
        }
        if let Some(v) = self.response.as_ref() {
            struct_ser.serialize_field("response", v)?;
        }
        if let Some(v) = self.client_extension_results.as_ref() {
            struct_ser.serialize_field("clientExtensionResults", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PublicKeyCredentialWithAttestation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "type",
            "raw_id",
            "rawId",
            "authenticator_attachment",
            "authenticatorAttachment",
            "response",
            "client_extension_results",
            "clientExtensionResults",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Type,
            RawId,
            AuthenticatorAttachment,
            Response,
            ClientExtensionResults,
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
                            "id" => Ok(GeneratedField::Id),
                            "type" => Ok(GeneratedField::Type),
                            "rawId" | "raw_id" => Ok(GeneratedField::RawId),
                            "authenticatorAttachment" | "authenticator_attachment" => Ok(GeneratedField::AuthenticatorAttachment),
                            "response" => Ok(GeneratedField::Response),
                            "clientExtensionResults" | "client_extension_results" => Ok(GeneratedField::ClientExtensionResults),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PublicKeyCredentialWithAttestation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.PublicKeyCredentialWithAttestation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PublicKeyCredentialWithAttestation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut r#type__ = None;
                let mut raw_id__ = None;
                let mut authenticator_attachment__ = None;
                let mut response__ = None;
                let mut client_extension_results__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RawId => {
                            if raw_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rawId"));
                            }
                            raw_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthenticatorAttachment => {
                            if authenticator_attachment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorAttachment"));
                            }
                            authenticator_attachment__ = map_.next_value()?;
                        }
                        GeneratedField::Response => {
                            if response__.is_some() {
                                return Err(serde::de::Error::duplicate_field("response"));
                            }
                            response__ = map_.next_value()?;
                        }
                        GeneratedField::ClientExtensionResults => {
                            if client_extension_results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientExtensionResults"));
                            }
                            client_extension_results__ = map_.next_value()?;
                        }
                    }
                }
                Ok(PublicKeyCredentialWithAttestation {
                    id: id__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    raw_id: raw_id__.unwrap_or_default(),
                    authenticator_attachment: authenticator_attachment__,
                    response: response__,
                    client_extension_results: client_extension_results__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.PublicKeyCredentialWithAttestation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SimpleClientExtensionResults {
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
        let mut struct_ser = serializer.serialize_struct("immutable.webauthn.v1.SimpleClientExtensionResults", len)?;
        if let Some(v) = self.appid.as_ref() {
            struct_ser.serialize_field("appid", v)?;
        }
        if let Some(v) = self.appid_exclude.as_ref() {
            struct_ser.serialize_field("appidExclude", v)?;
        }
        if let Some(v) = self.cred_props.as_ref() {
            struct_ser.serialize_field("credProps", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SimpleClientExtensionResults {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "appid",
            "appid_exclude",
            "appidExclude",
            "cred_props",
            "credProps",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Appid,
            AppidExclude,
            CredProps,
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
                            "appid" => Ok(GeneratedField::Appid),
                            "appidExclude" | "appid_exclude" => Ok(GeneratedField::AppidExclude),
                            "credProps" | "cred_props" => Ok(GeneratedField::CredProps),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SimpleClientExtensionResults;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct immutable.webauthn.v1.SimpleClientExtensionResults")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SimpleClientExtensionResults, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut appid__ = None;
                let mut appid_exclude__ = None;
                let mut cred_props__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Appid => {
                            if appid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appid"));
                            }
                            appid__ = map_.next_value()?;
                        }
                        GeneratedField::AppidExclude => {
                            if appid_exclude__.is_some() {
                                return Err(serde::de::Error::duplicate_field("appidExclude"));
                            }
                            appid_exclude__ = map_.next_value()?;
                        }
                        GeneratedField::CredProps => {
                            if cred_props__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credProps"));
                            }
                            cred_props__ = map_.next_value()?;
                        }
                    }
                }
                Ok(SimpleClientExtensionResults {
                    appid: appid__,
                    appid_exclude: appid_exclude__,
                    cred_props: cred_props__,
                })
            }
        }
        deserializer.deserialize_struct("immutable.webauthn.v1.SimpleClientExtensionResults", FIELDS, GeneratedVisitor)
    }
}
