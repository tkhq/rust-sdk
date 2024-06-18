impl serde::Serialize for Address {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Address", len)?;
        if true {
            let v = super::super::super::immutable::common::v1::AddressFormat::try_from(self.format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.format)))?;
            struct_ser.serialize_field("format", &v)?;
        }
        if true {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Address {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "format",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Format,
            Address,
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
                            "format" => Ok(GeneratedField::Format),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Address;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Address")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Address, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut format__ = None;
                let mut address__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value::<super::super::super::immutable::common::v1::AddressFormat>()? as i32);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Address {
                    format: format__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Address", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ApiKey {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.ApiKey", len)?;
        if let Some(v) = self.credential.as_ref() {
            struct_ser.serialize_field("credential", v)?;
        }
        if true {
            struct_ser.serialize_field("apiKeyId", &self.api_key_id)?;
        }
        if true {
            struct_ser.serialize_field("apiKeyName", &self.api_key_name)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        if let Some(v) = self.expiration_seconds.as_ref() {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("expirationSeconds", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ApiKey {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credential",
            "api_key_id",
            "apiKeyId",
            "api_key_name",
            "apiKeyName",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "expiration_seconds",
            "expirationSeconds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Credential,
            ApiKeyId,
            ApiKeyName,
            CreatedAt,
            UpdatedAt,
            ExpirationSeconds,
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
                            "credential" => Ok(GeneratedField::Credential),
                            "apiKeyId" | "api_key_id" => Ok(GeneratedField::ApiKeyId),
                            "apiKeyName" | "api_key_name" => Ok(GeneratedField::ApiKeyName),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "expirationSeconds" | "expiration_seconds" => Ok(GeneratedField::ExpirationSeconds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ApiKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.ApiKey")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ApiKey, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut credential__ = None;
                let mut api_key_id__ = None;
                let mut api_key_name__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut expiration_seconds__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Credential => {
                            if credential__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credential"));
                            }
                            credential__ = map_.next_value()?;
                        }
                        GeneratedField::ApiKeyId => {
                            if api_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyId"));
                            }
                            api_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeyName => {
                            if api_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeyName"));
                            }
                            api_key_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                        GeneratedField::ExpirationSeconds => {
                            if expiration_seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expirationSeconds"));
                            }
                            expiration_seconds__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                    }
                }
                Ok(ApiKey {
                    credential: credential__,
                    api_key_id: api_key_id__.unwrap_or_default(),
                    api_key_name: api_key_name__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                    expiration_seconds: expiration_seconds__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.ApiKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Authenticator {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Authenticator", len)?;
        if true {
            let v = self.transports.iter().cloned().map(|v| {
                super::super::super::immutable::webauthn::v1::AuthenticatorTransport::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("transports", &v)?;
        }
        if true {
            struct_ser.serialize_field("attestationType", &self.attestation_type)?;
        }
        if true {
            struct_ser.serialize_field("aaguid", &self.aaguid)?;
        }
        if true {
            struct_ser.serialize_field("credentialId", &self.credential_id)?;
        }
        if true {
            struct_ser.serialize_field("model", &self.model)?;
        }
        if let Some(v) = self.credential.as_ref() {
            struct_ser.serialize_field("credential", v)?;
        }
        if true {
            struct_ser.serialize_field("authenticatorId", &self.authenticator_id)?;
        }
        if true {
            struct_ser.serialize_field("authenticatorName", &self.authenticator_name)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Authenticator {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "transports",
            "attestation_type",
            "attestationType",
            "aaguid",
            "credential_id",
            "credentialId",
            "model",
            "credential",
            "authenticator_id",
            "authenticatorId",
            "authenticator_name",
            "authenticatorName",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Transports,
            AttestationType,
            Aaguid,
            CredentialId,
            Model,
            Credential,
            AuthenticatorId,
            AuthenticatorName,
            CreatedAt,
            UpdatedAt,
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
                            "transports" => Ok(GeneratedField::Transports),
                            "attestationType" | "attestation_type" => Ok(GeneratedField::AttestationType),
                            "aaguid" => Ok(GeneratedField::Aaguid),
                            "credentialId" | "credential_id" => Ok(GeneratedField::CredentialId),
                            "model" => Ok(GeneratedField::Model),
                            "credential" => Ok(GeneratedField::Credential),
                            "authenticatorId" | "authenticator_id" => Ok(GeneratedField::AuthenticatorId),
                            "authenticatorName" | "authenticator_name" => Ok(GeneratedField::AuthenticatorName),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Authenticator;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Authenticator")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Authenticator, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut transports__ = None;
                let mut attestation_type__ = None;
                let mut aaguid__ = None;
                let mut credential_id__ = None;
                let mut model__ = None;
                let mut credential__ = None;
                let mut authenticator_id__ = None;
                let mut authenticator_name__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Transports => {
                            if transports__.is_some() {
                                return Err(serde::de::Error::duplicate_field("transports"));
                            }
                            transports__ = Some(map_.next_value::<Vec<super::super::super::immutable::webauthn::v1::AuthenticatorTransport>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::AttestationType => {
                            if attestation_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("attestationType"));
                            }
                            attestation_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Aaguid => {
                            if aaguid__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aaguid"));
                            }
                            aaguid__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CredentialId => {
                            if credential_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialId"));
                            }
                            credential_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Model => {
                            if model__.is_some() {
                                return Err(serde::de::Error::duplicate_field("model"));
                            }
                            model__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Credential => {
                            if credential__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credential"));
                            }
                            credential__ = map_.next_value()?;
                        }
                        GeneratedField::AuthenticatorId => {
                            if authenticator_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorId"));
                            }
                            authenticator_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AuthenticatorName => {
                            if authenticator_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticatorName"));
                            }
                            authenticator_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Authenticator {
                    transports: transports__.unwrap_or_default(),
                    attestation_type: attestation_type__.unwrap_or_default(),
                    aaguid: aaguid__.unwrap_or_default(),
                    credential_id: credential_id__.unwrap_or_default(),
                    model: model__.unwrap_or_default(),
                    credential: credential__,
                    authenticator_id: authenticator_id__.unwrap_or_default(),
                    authenticator_name: authenticator_name__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Authenticator", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Credential {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Credential", len)?;
        if true {
            struct_ser.serialize_field("publicKey", &self.public_key)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::CredentialType::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Credential {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "public_key",
            "publicKey",
            "type",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PublicKey,
            Type,
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
                            "publicKey" | "public_key" => Ok(GeneratedField::PublicKey),
                            "type" => Ok(GeneratedField::Type),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Credential;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Credential")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Credential, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut public_key__ = None;
                let mut r#type__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PublicKey => {
                            if public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicKey"));
                            }
                            public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<super::super::super::immutable::common::v1::CredentialType>()? as i32);
                        }
                    }
                }
                Ok(Credential {
                    public_key: public_key__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Credential", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Invitation {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Invitation", len)?;
        if true {
            struct_ser.serialize_field("invitationId", &self.invitation_id)?;
        }
        if true {
            struct_ser.serialize_field("receiverUserName", &self.receiver_user_name)?;
        }
        if true {
            struct_ser.serialize_field("receiverEmail", &self.receiver_email)?;
        }
        if true {
            struct_ser.serialize_field("receiverUserTags", &self.receiver_user_tags)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::AccessType::try_from(self.access_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.access_type)))?;
            struct_ser.serialize_field("accessType", &v)?;
        }
        if true {
            let v = InvitationStatus::try_from(self.status)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        if true {
            struct_ser.serialize_field("senderUserId", &self.sender_user_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Invitation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "invitation_id",
            "invitationId",
            "receiver_user_name",
            "receiverUserName",
            "receiver_email",
            "receiverEmail",
            "receiver_user_tags",
            "receiverUserTags",
            "access_type",
            "accessType",
            "status",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "sender_user_id",
            "senderUserId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InvitationId,
            ReceiverUserName,
            ReceiverEmail,
            ReceiverUserTags,
            AccessType,
            Status,
            CreatedAt,
            UpdatedAt,
            SenderUserId,
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
                            "invitationId" | "invitation_id" => Ok(GeneratedField::InvitationId),
                            "receiverUserName" | "receiver_user_name" => Ok(GeneratedField::ReceiverUserName),
                            "receiverEmail" | "receiver_email" => Ok(GeneratedField::ReceiverEmail),
                            "receiverUserTags" | "receiver_user_tags" => Ok(GeneratedField::ReceiverUserTags),
                            "accessType" | "access_type" => Ok(GeneratedField::AccessType),
                            "status" => Ok(GeneratedField::Status),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "senderUserId" | "sender_user_id" => Ok(GeneratedField::SenderUserId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Invitation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Invitation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Invitation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut invitation_id__ = None;
                let mut receiver_user_name__ = None;
                let mut receiver_email__ = None;
                let mut receiver_user_tags__ = None;
                let mut access_type__ = None;
                let mut status__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut sender_user_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::InvitationId => {
                            if invitation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitationId"));
                            }
                            invitation_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReceiverUserName => {
                            if receiver_user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverUserName"));
                            }
                            receiver_user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReceiverEmail => {
                            if receiver_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverEmail"));
                            }
                            receiver_email__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReceiverUserTags => {
                            if receiver_user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("receiverUserTags"));
                            }
                            receiver_user_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AccessType => {
                            if access_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessType"));
                            }
                            access_type__ = Some(map_.next_value::<super::super::super::immutable::common::v1::AccessType>()? as i32);
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map_.next_value::<InvitationStatus>()? as i32);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                        GeneratedField::SenderUserId => {
                            if sender_user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("senderUserId"));
                            }
                            sender_user_id__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Invitation {
                    invitation_id: invitation_id__.unwrap_or_default(),
                    receiver_user_name: receiver_user_name__.unwrap_or_default(),
                    receiver_email: receiver_email__.unwrap_or_default(),
                    receiver_user_tags: receiver_user_tags__.unwrap_or_default(),
                    access_type: access_type__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                    sender_user_id: sender_user_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Invitation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InvitationStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "INVITATION_STATUS_UNSPECIFIED",
            Self::Created => "INVITATION_STATUS_CREATED",
            Self::Accepted => "INVITATION_STATUS_ACCEPTED",
            Self::Revoked => "INVITATION_STATUS_REVOKED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for InvitationStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "INVITATION_STATUS_UNSPECIFIED",
            "INVITATION_STATUS_CREATED",
            "INVITATION_STATUS_ACCEPTED",
            "INVITATION_STATUS_REVOKED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InvitationStatus;

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
                    "INVITATION_STATUS_UNSPECIFIED" => Ok(InvitationStatus::Unspecified),
                    "INVITATION_STATUS_CREATED" => Ok(InvitationStatus::Created),
                    "INVITATION_STATUS_ACCEPTED" => Ok(InvitationStatus::Accepted),
                    "INVITATION_STATUS_REVOKED" => Ok(InvitationStatus::Revoked),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for OrganizationData {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.OrganizationData", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("users", &self.users)?;
        }
        if true {
            struct_ser.serialize_field("policies", &self.policies)?;
        }
        if true {
            struct_ser.serialize_field("privateKeys", &self.private_keys)?;
        }
        if true {
            struct_ser.serialize_field("invitations", &self.invitations)?;
        }
        if true {
            struct_ser.serialize_field("tags", &self.tags)?;
        }
        if let Some(v) = self.root_quorum.as_ref() {
            struct_ser.serialize_field("rootQuorum", v)?;
        }
        if true {
            struct_ser.serialize_field("features", &self.features)?;
        }
        if true {
            struct_ser.serialize_field("wallets", &self.wallets)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for OrganizationData {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
            "name",
            "users",
            "policies",
            "private_keys",
            "privateKeys",
            "invitations",
            "tags",
            "root_quorum",
            "rootQuorum",
            "features",
            "wallets",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
            Name,
            Users,
            Policies,
            PrivateKeys,
            Invitations,
            Tags,
            RootQuorum,
            Features,
            Wallets,
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
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            "name" => Ok(GeneratedField::Name),
                            "users" => Ok(GeneratedField::Users),
                            "policies" => Ok(GeneratedField::Policies),
                            "privateKeys" | "private_keys" => Ok(GeneratedField::PrivateKeys),
                            "invitations" => Ok(GeneratedField::Invitations),
                            "tags" => Ok(GeneratedField::Tags),
                            "rootQuorum" | "root_quorum" => Ok(GeneratedField::RootQuorum),
                            "features" => Ok(GeneratedField::Features),
                            "wallets" => Ok(GeneratedField::Wallets),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = OrganizationData;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.OrganizationData")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<OrganizationData, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                let mut name__ = None;
                let mut users__ = None;
                let mut policies__ = None;
                let mut private_keys__ = None;
                let mut invitations__ = None;
                let mut tags__ = None;
                let mut root_quorum__ = None;
                let mut features__ = None;
                let mut wallets__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Users => {
                            if users__.is_some() {
                                return Err(serde::de::Error::duplicate_field("users"));
                            }
                            users__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Policies => {
                            if policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policies"));
                            }
                            policies__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeys => {
                            if private_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeys"));
                            }
                            private_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Invitations => {
                            if invitations__.is_some() {
                                return Err(serde::de::Error::duplicate_field("invitations"));
                            }
                            invitations__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Tags => {
                            if tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RootQuorum => {
                            if root_quorum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rootQuorum"));
                            }
                            root_quorum__ = map_.next_value()?;
                        }
                        GeneratedField::Features => {
                            if features__.is_some() {
                                return Err(serde::de::Error::duplicate_field("features"));
                            }
                            features__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Wallets => {
                            if wallets__.is_some() {
                                return Err(serde::de::Error::duplicate_field("wallets"));
                            }
                            wallets__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(OrganizationData {
                    organization_id: organization_id__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    users: users__.unwrap_or_default(),
                    policies: policies__.unwrap_or_default(),
                    private_keys: private_keys__.unwrap_or_default(),
                    invitations: invitations__.unwrap_or_default(),
                    tags: tags__.unwrap_or_default(),
                    root_quorum: root_quorum__,
                    features: features__.unwrap_or_default(),
                    wallets: wallets__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.OrganizationData", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Policy {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Policy", len)?;
        if true {
            struct_ser.serialize_field("policyId", &self.policy_id)?;
        }
        if true {
            struct_ser.serialize_field("policyName", &self.policy_name)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::Effect::try_from(self.effect)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.effect)))?;
            struct_ser.serialize_field("effect", &v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        if true {
            struct_ser.serialize_field("notes", &self.notes)?;
        }
        if let Some(v) = self.consensus.as_ref() {
            struct_ser.serialize_field("consensus", v)?;
        }
        if let Some(v) = self.condition.as_ref() {
            struct_ser.serialize_field("condition", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Policy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policy_id",
            "policyId",
            "policy_name",
            "policyName",
            "effect",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "notes",
            "consensus",
            "condition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PolicyId,
            PolicyName,
            Effect,
            CreatedAt,
            UpdatedAt,
            Notes,
            Consensus,
            Condition,
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
                            "policyId" | "policy_id" => Ok(GeneratedField::PolicyId),
                            "policyName" | "policy_name" => Ok(GeneratedField::PolicyName),
                            "effect" => Ok(GeneratedField::Effect),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "notes" => Ok(GeneratedField::Notes),
                            "consensus" => Ok(GeneratedField::Consensus),
                            "condition" => Ok(GeneratedField::Condition),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Policy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Policy")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Policy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policy_id__ = None;
                let mut policy_name__ = None;
                let mut effect__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut notes__ = None;
                let mut consensus__ = None;
                let mut condition__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PolicyId => {
                            if policy_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyId"));
                            }
                            policy_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PolicyName => {
                            if policy_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyName"));
                            }
                            policy_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Effect => {
                            if effect__.is_some() {
                                return Err(serde::de::Error::duplicate_field("effect"));
                            }
                            effect__ = Some(map_.next_value::<super::super::super::immutable::common::v1::Effect>()? as i32);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                        GeneratedField::Notes => {
                            if notes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("notes"));
                            }
                            notes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Consensus => {
                            if consensus__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consensus"));
                            }
                            consensus__ = map_.next_value()?;
                        }
                        GeneratedField::Condition => {
                            if condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("condition"));
                            }
                            condition__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Policy {
                    policy_id: policy_id__.unwrap_or_default(),
                    policy_name: policy_name__.unwrap_or_default(),
                    effect: effect__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                    notes: notes__.unwrap_or_default(),
                    consensus: consensus__,
                    condition: condition__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Policy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PrivateKey {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.PrivateKey", len)?;
        if true {
            struct_ser.serialize_field("privateKeyId", &self.private_key_id)?;
        }
        if true {
            struct_ser.serialize_field("publicKey", &self.public_key)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyName", &self.private_key_name)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::Curve::try_from(self.curve)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.curve)))?;
            struct_ser.serialize_field("curve", &v)?;
        }
        if true {
            struct_ser.serialize_field("addresses", &self.addresses)?;
        }
        if true {
            struct_ser.serialize_field("privateKeyTags", &self.private_key_tags)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        if true {
            struct_ser.serialize_field("exported", &self.exported)?;
        }
        if true {
            struct_ser.serialize_field("imported", &self.imported)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PrivateKey {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "private_key_id",
            "privateKeyId",
            "public_key",
            "publicKey",
            "private_key_name",
            "privateKeyName",
            "curve",
            "addresses",
            "private_key_tags",
            "privateKeyTags",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "exported",
            "imported",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PrivateKeyId,
            PublicKey,
            PrivateKeyName,
            Curve,
            Addresses,
            PrivateKeyTags,
            CreatedAt,
            UpdatedAt,
            Exported,
            Imported,
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
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            "publicKey" | "public_key" => Ok(GeneratedField::PublicKey),
                            "privateKeyName" | "private_key_name" => Ok(GeneratedField::PrivateKeyName),
                            "curve" => Ok(GeneratedField::Curve),
                            "addresses" => Ok(GeneratedField::Addresses),
                            "privateKeyTags" | "private_key_tags" => Ok(GeneratedField::PrivateKeyTags),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "exported" => Ok(GeneratedField::Exported),
                            "imported" => Ok(GeneratedField::Imported),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PrivateKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.PrivateKey")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PrivateKey, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut private_key_id__ = None;
                let mut public_key__ = None;
                let mut private_key_name__ = None;
                let mut curve__ = None;
                let mut addresses__ = None;
                let mut private_key_tags__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut exported__ = None;
                let mut imported__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PublicKey => {
                            if public_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("publicKey"));
                            }
                            public_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyName => {
                            if private_key_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyName"));
                            }
                            private_key_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Curve => {
                            if curve__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curve"));
                            }
                            curve__ = Some(map_.next_value::<super::super::super::immutable::common::v1::Curve>()? as i32);
                        }
                        GeneratedField::Addresses => {
                            if addresses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addresses"));
                            }
                            addresses__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PrivateKeyTags => {
                            if private_key_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyTags"));
                            }
                            private_key_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                        GeneratedField::Exported => {
                            if exported__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exported"));
                            }
                            exported__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Imported => {
                            if imported__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imported"));
                            }
                            imported__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(PrivateKey {
                    private_key_id: private_key_id__.unwrap_or_default(),
                    public_key: public_key__.unwrap_or_default(),
                    private_key_name: private_key_name__.unwrap_or_default(),
                    curve: curve__.unwrap_or_default(),
                    addresses: addresses__.unwrap_or_default(),
                    private_key_tags: private_key_tags__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                    exported: exported__.unwrap_or_default(),
                    imported: imported__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.PrivateKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Quorum {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Quorum", len)?;
        if true {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if true {
            struct_ser.serialize_field("userIds", &self.user_ids)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Quorum {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "threshold",
            "user_ids",
            "userIds",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Threshold,
            UserIds,
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
                            "threshold" => Ok(GeneratedField::Threshold),
                            "userIds" | "user_ids" => Ok(GeneratedField::UserIds),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Quorum;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Quorum")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Quorum, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut threshold__ = None;
                let mut user_ids__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UserIds => {
                            if user_ids__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userIds"));
                            }
                            user_ids__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Quorum {
                    threshold: threshold__.unwrap_or_default(),
                    user_ids: user_ids__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Quorum", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Tag {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Tag", len)?;
        if true {
            struct_ser.serialize_field("tagId", &self.tag_id)?;
        }
        if true {
            struct_ser.serialize_field("tagName", &self.tag_name)?;
        }
        if true {
            let v = TagType::try_from(self.tag_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.tag_type)))?;
            struct_ser.serialize_field("tagType", &v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Tag {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_id",
            "tagId",
            "tag_name",
            "tagName",
            "tag_type",
            "tagType",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagId,
            TagName,
            TagType,
            CreatedAt,
            UpdatedAt,
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
                            "tagId" | "tag_id" => Ok(GeneratedField::TagId),
                            "tagName" | "tag_name" => Ok(GeneratedField::TagName),
                            "tagType" | "tag_type" => Ok(GeneratedField::TagType),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Tag;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Tag")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Tag, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_id__ = None;
                let mut tag_name__ = None;
                let mut tag_type__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagId => {
                            if tag_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagId"));
                            }
                            tag_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagName => {
                            if tag_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagName"));
                            }
                            tag_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagType => {
                            if tag_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagType"));
                            }
                            tag_type__ = Some(map_.next_value::<TagType>()? as i32);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Tag {
                    tag_id: tag_id__.unwrap_or_default(),
                    tag_name: tag_name__.unwrap_or_default(),
                    tag_type: tag_type__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Tag", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TagType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "TAG_TYPE_UNSPECIFIED",
            Self::User => "TAG_TYPE_USER",
            Self::PrivateKey => "TAG_TYPE_PRIVATE_KEY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for TagType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TAG_TYPE_UNSPECIFIED",
            "TAG_TYPE_USER",
            "TAG_TYPE_PRIVATE_KEY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TagType;

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
                    "TAG_TYPE_UNSPECIFIED" => Ok(TagType::Unspecified),
                    "TAG_TYPE_USER" => Ok(TagType::User),
                    "TAG_TYPE_PRIVATE_KEY" => Ok(TagType::PrivateKey),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Timestamp {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Timestamp", len)?;
        if true {
            struct_ser.serialize_field("seconds", &self.seconds)?;
        }
        if true {
            struct_ser.serialize_field("nanos", &self.nanos)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Timestamp {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "seconds",
            "nanos",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Seconds,
            Nanos,
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
                            "seconds" => Ok(GeneratedField::Seconds),
                            "nanos" => Ok(GeneratedField::Nanos),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Timestamp")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Timestamp, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut seconds__ = None;
                let mut nanos__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Seconds => {
                            if seconds__.is_some() {
                                return Err(serde::de::Error::duplicate_field("seconds"));
                            }
                            seconds__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Nanos => {
                            if nanos__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nanos"));
                            }
                            nanos__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Timestamp {
                    seconds: seconds__.unwrap_or_default(),
                    nanos: nanos__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Timestamp", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for User {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("external.data.v1.User", len)?;
        if true {
            struct_ser.serialize_field("userId", &self.user_id)?;
        }
        if true {
            struct_ser.serialize_field("userName", &self.user_name)?;
        }
        if let Some(v) = self.user_email.as_ref() {
            struct_ser.serialize_field("userEmail", v)?;
        }
        if true {
            struct_ser.serialize_field("authenticators", &self.authenticators)?;
        }
        if true {
            struct_ser.serialize_field("apiKeys", &self.api_keys)?;
        }
        if true {
            struct_ser.serialize_field("userTags", &self.user_tags)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for User {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "user_id",
            "userId",
            "user_name",
            "userName",
            "user_email",
            "userEmail",
            "authenticators",
            "api_keys",
            "apiKeys",
            "user_tags",
            "userTags",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            UserId,
            UserName,
            UserEmail,
            Authenticators,
            ApiKeys,
            UserTags,
            CreatedAt,
            UpdatedAt,
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
                            "userId" | "user_id" => Ok(GeneratedField::UserId),
                            "userName" | "user_name" => Ok(GeneratedField::UserName),
                            "userEmail" | "user_email" => Ok(GeneratedField::UserEmail),
                            "authenticators" => Ok(GeneratedField::Authenticators),
                            "apiKeys" | "api_keys" => Ok(GeneratedField::ApiKeys),
                            "userTags" | "user_tags" => Ok(GeneratedField::UserTags),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = User;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.User")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<User, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut user_id__ = None;
                let mut user_name__ = None;
                let mut user_email__ = None;
                let mut authenticators__ = None;
                let mut api_keys__ = None;
                let mut user_tags__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::UserId => {
                            if user_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userId"));
                            }
                            user_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserName => {
                            if user_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userName"));
                            }
                            user_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserEmail => {
                            if user_email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userEmail"));
                            }
                            user_email__ = map_.next_value()?;
                        }
                        GeneratedField::Authenticators => {
                            if authenticators__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authenticators"));
                            }
                            authenticators__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApiKeys => {
                            if api_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("apiKeys"));
                            }
                            api_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UserTags => {
                            if user_tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("userTags"));
                            }
                            user_tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(User {
                    user_id: user_id__.unwrap_or_default(),
                    user_name: user_name__.unwrap_or_default(),
                    user_email: user_email__,
                    authenticators: authenticators__.unwrap_or_default(),
                    api_keys: api_keys__.unwrap_or_default(),
                    user_tags: user_tags__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.User", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Wallet {
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
        let mut struct_ser = serializer.serialize_struct("external.data.v1.Wallet", len)?;
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            struct_ser.serialize_field("walletName", &self.wallet_name)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        if true {
            struct_ser.serialize_field("exported", &self.exported)?;
        }
        if true {
            struct_ser.serialize_field("imported", &self.imported)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Wallet {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "wallet_id",
            "walletId",
            "wallet_name",
            "walletName",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "exported",
            "imported",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            WalletId,
            WalletName,
            CreatedAt,
            UpdatedAt,
            Exported,
            Imported,
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
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "walletName" | "wallet_name" => Ok(GeneratedField::WalletName),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "exported" => Ok(GeneratedField::Exported),
                            "imported" => Ok(GeneratedField::Imported),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Wallet;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.Wallet")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Wallet, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut wallet_id__ = None;
                let mut wallet_name__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut exported__ = None;
                let mut imported__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WalletName => {
                            if wallet_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletName"));
                            }
                            wallet_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                        GeneratedField::Exported => {
                            if exported__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exported"));
                            }
                            exported__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Imported => {
                            if imported__.is_some() {
                                return Err(serde::de::Error::duplicate_field("imported"));
                            }
                            imported__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Wallet {
                    wallet_id: wallet_id__.unwrap_or_default(),
                    wallet_name: wallet_name__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                    exported: exported__.unwrap_or_default(),
                    imported: imported__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.Wallet", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for WalletAccount {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("external.data.v1.WalletAccount", len)?;
        if true {
            struct_ser.serialize_field("organizationId", &self.organization_id)?;
        }
        if true {
            struct_ser.serialize_field("walletId", &self.wallet_id)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::Curve::try_from(self.curve)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.curve)))?;
            struct_ser.serialize_field("curve", &v)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::PathFormat::try_from(self.path_format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.path_format)))?;
            struct_ser.serialize_field("pathFormat", &v)?;
        }
        if true {
            struct_ser.serialize_field("path", &self.path)?;
        }
        if true {
            let v = super::super::super::immutable::common::v1::AddressFormat::try_from(self.address_format)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.address_format)))?;
            struct_ser.serialize_field("addressFormat", &v)?;
        }
        if true {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            struct_ser.serialize_field("updatedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for WalletAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "organization_id",
            "organizationId",
            "wallet_id",
            "walletId",
            "curve",
            "path_format",
            "pathFormat",
            "path",
            "address_format",
            "addressFormat",
            "address",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OrganizationId,
            WalletId,
            Curve,
            PathFormat,
            Path,
            AddressFormat,
            Address,
            CreatedAt,
            UpdatedAt,
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
                            "organizationId" | "organization_id" => Ok(GeneratedField::OrganizationId),
                            "walletId" | "wallet_id" => Ok(GeneratedField::WalletId),
                            "curve" => Ok(GeneratedField::Curve),
                            "pathFormat" | "path_format" => Ok(GeneratedField::PathFormat),
                            "path" => Ok(GeneratedField::Path),
                            "addressFormat" | "address_format" => Ok(GeneratedField::AddressFormat),
                            "address" => Ok(GeneratedField::Address),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = WalletAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.data.v1.WalletAccount")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<WalletAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut organization_id__ = None;
                let mut wallet_id__ = None;
                let mut curve__ = None;
                let mut path_format__ = None;
                let mut path__ = None;
                let mut address_format__ = None;
                let mut address__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OrganizationId => {
                            if organization_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("organizationId"));
                            }
                            organization_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WalletId => {
                            if wallet_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("walletId"));
                            }
                            wallet_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Curve => {
                            if curve__.is_some() {
                                return Err(serde::de::Error::duplicate_field("curve"));
                            }
                            curve__ = Some(map_.next_value::<super::super::super::immutable::common::v1::Curve>()? as i32);
                        }
                        GeneratedField::PathFormat => {
                            if path_format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pathFormat"));
                            }
                            path_format__ = Some(map_.next_value::<super::super::super::immutable::common::v1::PathFormat>()? as i32);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AddressFormat => {
                            if address_format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressFormat"));
                            }
                            address_format__ = Some(map_.next_value::<super::super::super::immutable::common::v1::AddressFormat>()? as i32);
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = map_.next_value()?;
                        }
                    }
                }
                Ok(WalletAccount {
                    organization_id: organization_id__.unwrap_or_default(),
                    wallet_id: wallet_id__.unwrap_or_default(),
                    curve: curve__.unwrap_or_default(),
                    path_format: path_format__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                    address_format: address_format__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                    created_at: created_at__,
                    updated_at: updated_at__,
                })
            }
        }
        deserializer.deserialize_struct("external.data.v1.WalletAccount", FIELDS, GeneratedVisitor)
    }
}
