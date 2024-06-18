impl serde::Serialize for Contact {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Contact", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("url", &self.url)?;
        }
        if true {
            struct_ser.serialize_field("email", &self.email)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Contact {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "url",
            "email",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Url,
            Email,
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
                            "name" => Ok(GeneratedField::Name),
                            "url" => Ok(GeneratedField::Url),
                            "email" => Ok(GeneratedField::Email),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Contact;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Contact")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Contact, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut url__ = None;
                let mut email__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Contact {
                    name: name__.unwrap_or_default(),
                    url: url__.unwrap_or_default(),
                    email: email__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Contact", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ExternalDocumentation {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.ExternalDocumentation", len)?;
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            struct_ser.serialize_field("url", &self.url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ExternalDocumentation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "url",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            Url,
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
                            "description" => Ok(GeneratedField::Description),
                            "url" => Ok(GeneratedField::Url),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ExternalDocumentation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.ExternalDocumentation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ExternalDocumentation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ExternalDocumentation {
                    description: description__.unwrap_or_default(),
                    url: url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.ExternalDocumentation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Header {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Header", len)?;
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if true {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if true {
            struct_ser.serialize_field("default", &self.default)?;
        }
        if true {
            struct_ser.serialize_field("pattern", &self.pattern)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Header {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "type",
            "format",
            "default",
            "pattern",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            Type,
            Format,
            Default,
            Pattern,
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
                            "description" => Ok(GeneratedField::Description),
                            "type" => Ok(GeneratedField::Type),
                            "format" => Ok(GeneratedField::Format),
                            "default" => Ok(GeneratedField::Default),
                            "pattern" => Ok(GeneratedField::Pattern),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Header;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Header")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Header, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut r#type__ = None;
                let mut format__ = None;
                let mut default__ = None;
                let mut pattern__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Default => {
                            if default__.is_some() {
                                return Err(serde::de::Error::duplicate_field("default"));
                            }
                            default__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Pattern => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pattern"));
                            }
                            pattern__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Header {
                    description: description__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    default: default__.unwrap_or_default(),
                    pattern: pattern__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Header", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HeaderParameter {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.HeaderParameter", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            let v = header_parameter::Type::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if true {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if true {
            struct_ser.serialize_field("required", &self.required)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HeaderParameter {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "description",
            "type",
            "format",
            "required",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Description,
            Type,
            Format,
            Required,
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
                            "name" => Ok(GeneratedField::Name),
                            "description" => Ok(GeneratedField::Description),
                            "type" => Ok(GeneratedField::Type),
                            "format" => Ok(GeneratedField::Format),
                            "required" => Ok(GeneratedField::Required),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HeaderParameter;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.HeaderParameter")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HeaderParameter, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut description__ = None;
                let mut r#type__ = None;
                let mut format__ = None;
                let mut required__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<header_parameter::Type>()? as i32);
                        }
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Required => {
                            if required__.is_some() {
                                return Err(serde::de::Error::duplicate_field("required"));
                            }
                            required__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(HeaderParameter {
                    name: name__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    required: required__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.HeaderParameter", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for header_parameter::Type {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "UNKNOWN",
            Self::String => "STRING",
            Self::Number => "NUMBER",
            Self::Integer => "INTEGER",
            Self::Boolean => "BOOLEAN",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for header_parameter::Type {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNKNOWN",
            "STRING",
            "NUMBER",
            "INTEGER",
            "BOOLEAN",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = header_parameter::Type;

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
                    "UNKNOWN" => Ok(header_parameter::Type::Unknown),
                    "STRING" => Ok(header_parameter::Type::String),
                    "NUMBER" => Ok(header_parameter::Type::Number),
                    "INTEGER" => Ok(header_parameter::Type::Integer),
                    "BOOLEAN" => Ok(header_parameter::Type::Boolean),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Info {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Info", len)?;
        if true {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            struct_ser.serialize_field("termsOfService", &self.terms_of_service)?;
        }
        if let Some(v) = self.contact.as_ref() {
            struct_ser.serialize_field("contact", v)?;
        }
        if let Some(v) = self.license.as_ref() {
            struct_ser.serialize_field("license", v)?;
        }
        if true {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Info {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "title",
            "description",
            "terms_of_service",
            "termsOfService",
            "contact",
            "license",
            "version",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Title,
            Description,
            TermsOfService,
            Contact,
            License,
            Version,
            Extensions,
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
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "termsOfService" | "terms_of_service" => Ok(GeneratedField::TermsOfService),
                            "contact" => Ok(GeneratedField::Contact),
                            "license" => Ok(GeneratedField::License),
                            "version" => Ok(GeneratedField::Version),
                            "extensions" => Ok(GeneratedField::Extensions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Info;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Info")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Info, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut title__ = None;
                let mut description__ = None;
                let mut terms_of_service__ = None;
                let mut contact__ = None;
                let mut license__ = None;
                let mut version__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TermsOfService => {
                            if terms_of_service__.is_some() {
                                return Err(serde::de::Error::duplicate_field("termsOfService"));
                            }
                            terms_of_service__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Contact => {
                            if contact__.is_some() {
                                return Err(serde::de::Error::duplicate_field("contact"));
                            }
                            contact__ = map_.next_value()?;
                        }
                        GeneratedField::License => {
                            if license__.is_some() {
                                return Err(serde::de::Error::duplicate_field("license"));
                            }
                            license__ = map_.next_value()?;
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Info {
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    terms_of_service: terms_of_service__.unwrap_or_default(),
                    contact: contact__,
                    license: license__,
                    version: version__.unwrap_or_default(),
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Info", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for JsonSchema {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.JSONSchema", len)?;
        if true {
            struct_ser.serialize_field("ref", &self.r#ref)?;
        }
        if true {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            struct_ser.serialize_field("default", &self.default)?;
        }
        if true {
            struct_ser.serialize_field("readOnly", &self.read_only)?;
        }
        if true {
            struct_ser.serialize_field("example", &self.example)?;
        }
        if true {
            struct_ser.serialize_field("multipleOf", &self.multiple_of)?;
        }
        if true {
            struct_ser.serialize_field("maximum", &self.maximum)?;
        }
        if true {
            struct_ser.serialize_field("exclusiveMaximum", &self.exclusive_maximum)?;
        }
        if true {
            struct_ser.serialize_field("minimum", &self.minimum)?;
        }
        if true {
            struct_ser.serialize_field("exclusiveMinimum", &self.exclusive_minimum)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("maxLength", ToString::to_string(&self.max_length).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("minLength", ToString::to_string(&self.min_length).as_str())?;
        }
        if true {
            struct_ser.serialize_field("pattern", &self.pattern)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("maxItems", ToString::to_string(&self.max_items).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("minItems", ToString::to_string(&self.min_items).as_str())?;
        }
        if true {
            struct_ser.serialize_field("uniqueItems", &self.unique_items)?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("maxProperties", ToString::to_string(&self.max_properties).as_str())?;
        }
        if true {
            #[allow(clippy::needless_borrow)]
            struct_ser.serialize_field("minProperties", ToString::to_string(&self.min_properties).as_str())?;
        }
        if true {
            struct_ser.serialize_field("required", &self.required)?;
        }
        if true {
            struct_ser.serialize_field("array", &self.array)?;
        }
        if true {
            let v = self.r#type.iter().cloned().map(|v| {
                json_schema::JsonSchemaSimpleTypes::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("type", &v)?;
        }
        if true {
            struct_ser.serialize_field("format", &self.format)?;
        }
        if true {
            struct_ser.serialize_field("enum", &self.r#enum)?;
        }
        if let Some(v) = self.field_configuration.as_ref() {
            struct_ser.serialize_field("fieldConfiguration", v)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for JsonSchema {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ref",
            "title",
            "description",
            "default",
            "read_only",
            "readOnly",
            "example",
            "multiple_of",
            "multipleOf",
            "maximum",
            "exclusive_maximum",
            "exclusiveMaximum",
            "minimum",
            "exclusive_minimum",
            "exclusiveMinimum",
            "max_length",
            "maxLength",
            "min_length",
            "minLength",
            "pattern",
            "max_items",
            "maxItems",
            "min_items",
            "minItems",
            "unique_items",
            "uniqueItems",
            "max_properties",
            "maxProperties",
            "min_properties",
            "minProperties",
            "required",
            "array",
            "type",
            "format",
            "enum",
            "field_configuration",
            "fieldConfiguration",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Ref,
            Title,
            Description,
            Default,
            ReadOnly,
            Example,
            MultipleOf,
            Maximum,
            ExclusiveMaximum,
            Minimum,
            ExclusiveMinimum,
            MaxLength,
            MinLength,
            Pattern,
            MaxItems,
            MinItems,
            UniqueItems,
            MaxProperties,
            MinProperties,
            Required,
            Array,
            Type,
            Format,
            Enum,
            FieldConfiguration,
            Extensions,
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
                            "ref" => Ok(GeneratedField::Ref),
                            "title" => Ok(GeneratedField::Title),
                            "description" => Ok(GeneratedField::Description),
                            "default" => Ok(GeneratedField::Default),
                            "readOnly" | "read_only" => Ok(GeneratedField::ReadOnly),
                            "example" => Ok(GeneratedField::Example),
                            "multipleOf" | "multiple_of" => Ok(GeneratedField::MultipleOf),
                            "maximum" => Ok(GeneratedField::Maximum),
                            "exclusiveMaximum" | "exclusive_maximum" => Ok(GeneratedField::ExclusiveMaximum),
                            "minimum" => Ok(GeneratedField::Minimum),
                            "exclusiveMinimum" | "exclusive_minimum" => Ok(GeneratedField::ExclusiveMinimum),
                            "maxLength" | "max_length" => Ok(GeneratedField::MaxLength),
                            "minLength" | "min_length" => Ok(GeneratedField::MinLength),
                            "pattern" => Ok(GeneratedField::Pattern),
                            "maxItems" | "max_items" => Ok(GeneratedField::MaxItems),
                            "minItems" | "min_items" => Ok(GeneratedField::MinItems),
                            "uniqueItems" | "unique_items" => Ok(GeneratedField::UniqueItems),
                            "maxProperties" | "max_properties" => Ok(GeneratedField::MaxProperties),
                            "minProperties" | "min_properties" => Ok(GeneratedField::MinProperties),
                            "required" => Ok(GeneratedField::Required),
                            "array" => Ok(GeneratedField::Array),
                            "type" => Ok(GeneratedField::Type),
                            "format" => Ok(GeneratedField::Format),
                            "enum" => Ok(GeneratedField::Enum),
                            "fieldConfiguration" | "field_configuration" => Ok(GeneratedField::FieldConfiguration),
                            "extensions" => Ok(GeneratedField::Extensions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = JsonSchema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.JSONSchema")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<JsonSchema, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#ref__ = None;
                let mut title__ = None;
                let mut description__ = None;
                let mut default__ = None;
                let mut read_only__ = None;
                let mut example__ = None;
                let mut multiple_of__ = None;
                let mut maximum__ = None;
                let mut exclusive_maximum__ = None;
                let mut minimum__ = None;
                let mut exclusive_minimum__ = None;
                let mut max_length__ = None;
                let mut min_length__ = None;
                let mut pattern__ = None;
                let mut max_items__ = None;
                let mut min_items__ = None;
                let mut unique_items__ = None;
                let mut max_properties__ = None;
                let mut min_properties__ = None;
                let mut required__ = None;
                let mut array__ = None;
                let mut r#type__ = None;
                let mut format__ = None;
                let mut r#enum__ = None;
                let mut field_configuration__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Ref => {
                            if r#ref__.is_some() {
                                return Err(serde::de::Error::duplicate_field("ref"));
                            }
                            r#ref__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Default => {
                            if default__.is_some() {
                                return Err(serde::de::Error::duplicate_field("default"));
                            }
                            default__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReadOnly => {
                            if read_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readOnly"));
                            }
                            read_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Example => {
                            if example__.is_some() {
                                return Err(serde::de::Error::duplicate_field("example"));
                            }
                            example__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MultipleOf => {
                            if multiple_of__.is_some() {
                                return Err(serde::de::Error::duplicate_field("multipleOf"));
                            }
                            multiple_of__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Maximum => {
                            if maximum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maximum"));
                            }
                            maximum__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ExclusiveMaximum => {
                            if exclusive_maximum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exclusiveMaximum"));
                            }
                            exclusive_maximum__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Minimum => {
                            if minimum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minimum"));
                            }
                            minimum__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::ExclusiveMinimum => {
                            if exclusive_minimum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exclusiveMinimum"));
                            }
                            exclusive_minimum__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxLength => {
                            if max_length__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxLength"));
                            }
                            max_length__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MinLength => {
                            if min_length__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minLength"));
                            }
                            min_length__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pattern => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pattern"));
                            }
                            pattern__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxItems => {
                            if max_items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxItems"));
                            }
                            max_items__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MinItems => {
                            if min_items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minItems"));
                            }
                            min_items__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::UniqueItems => {
                            if unique_items__.is_some() {
                                return Err(serde::de::Error::duplicate_field("uniqueItems"));
                            }
                            unique_items__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxProperties => {
                            if max_properties__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxProperties"));
                            }
                            max_properties__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MinProperties => {
                            if min_properties__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minProperties"));
                            }
                            min_properties__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Required => {
                            if required__.is_some() {
                                return Err(serde::de::Error::duplicate_field("required"));
                            }
                            required__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Array => {
                            if array__.is_some() {
                                return Err(serde::de::Error::duplicate_field("array"));
                            }
                            array__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<Vec<json_schema::JsonSchemaSimpleTypes>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::Format => {
                            if format__.is_some() {
                                return Err(serde::de::Error::duplicate_field("format"));
                            }
                            format__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Enum => {
                            if r#enum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("enum"));
                            }
                            r#enum__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FieldConfiguration => {
                            if field_configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fieldConfiguration"));
                            }
                            field_configuration__ = map_.next_value()?;
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(JsonSchema {
                    r#ref: r#ref__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    default: default__.unwrap_or_default(),
                    read_only: read_only__.unwrap_or_default(),
                    example: example__.unwrap_or_default(),
                    multiple_of: multiple_of__.unwrap_or_default(),
                    maximum: maximum__.unwrap_or_default(),
                    exclusive_maximum: exclusive_maximum__.unwrap_or_default(),
                    minimum: minimum__.unwrap_or_default(),
                    exclusive_minimum: exclusive_minimum__.unwrap_or_default(),
                    max_length: max_length__.unwrap_or_default(),
                    min_length: min_length__.unwrap_or_default(),
                    pattern: pattern__.unwrap_or_default(),
                    max_items: max_items__.unwrap_or_default(),
                    min_items: min_items__.unwrap_or_default(),
                    unique_items: unique_items__.unwrap_or_default(),
                    max_properties: max_properties__.unwrap_or_default(),
                    min_properties: min_properties__.unwrap_or_default(),
                    required: required__.unwrap_or_default(),
                    array: array__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    format: format__.unwrap_or_default(),
                    r#enum: r#enum__.unwrap_or_default(),
                    field_configuration: field_configuration__,
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.JSONSchema", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for json_schema::FieldConfiguration {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.JSONSchema.FieldConfiguration", len)?;
        if true {
            struct_ser.serialize_field("pathParamName", &self.path_param_name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for json_schema::FieldConfiguration {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "path_param_name",
            "pathParamName",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PathParamName,
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
                            "pathParamName" | "path_param_name" => Ok(GeneratedField::PathParamName),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = json_schema::FieldConfiguration;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.JSONSchema.FieldConfiguration")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<json_schema::FieldConfiguration, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut path_param_name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::PathParamName => {
                            if path_param_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pathParamName"));
                            }
                            path_param_name__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(json_schema::FieldConfiguration {
                    path_param_name: path_param_name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.JSONSchema.FieldConfiguration", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for json_schema::JsonSchemaSimpleTypes {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "UNKNOWN",
            Self::Array => "ARRAY",
            Self::Boolean => "BOOLEAN",
            Self::Integer => "INTEGER",
            Self::Null => "NULL",
            Self::Number => "NUMBER",
            Self::Object => "OBJECT",
            Self::String => "STRING",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for json_schema::JsonSchemaSimpleTypes {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNKNOWN",
            "ARRAY",
            "BOOLEAN",
            "INTEGER",
            "NULL",
            "NUMBER",
            "OBJECT",
            "STRING",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = json_schema::JsonSchemaSimpleTypes;

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
                    "UNKNOWN" => Ok(json_schema::JsonSchemaSimpleTypes::Unknown),
                    "ARRAY" => Ok(json_schema::JsonSchemaSimpleTypes::Array),
                    "BOOLEAN" => Ok(json_schema::JsonSchemaSimpleTypes::Boolean),
                    "INTEGER" => Ok(json_schema::JsonSchemaSimpleTypes::Integer),
                    "NULL" => Ok(json_schema::JsonSchemaSimpleTypes::Null),
                    "NUMBER" => Ok(json_schema::JsonSchemaSimpleTypes::Number),
                    "OBJECT" => Ok(json_schema::JsonSchemaSimpleTypes::Object),
                    "STRING" => Ok(json_schema::JsonSchemaSimpleTypes::String),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for License {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.License", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("url", &self.url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for License {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "url",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Url,
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
                            "name" => Ok(GeneratedField::Name),
                            "url" => Ok(GeneratedField::Url),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = License;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.License")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<License, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut url__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(License {
                    name: name__.unwrap_or_default(),
                    url: url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.License", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Operation {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Operation", len)?;
        if true {
            struct_ser.serialize_field("tags", &self.tags)?;
        }
        if true {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.external_docs.as_ref() {
            struct_ser.serialize_field("externalDocs", v)?;
        }
        if true {
            struct_ser.serialize_field("operationId", &self.operation_id)?;
        }
        if true {
            struct_ser.serialize_field("consumes", &self.consumes)?;
        }
        if true {
            struct_ser.serialize_field("produces", &self.produces)?;
        }
        if true {
            struct_ser.serialize_field("responses", &self.responses)?;
        }
        if true {
            let v = self.schemes.iter().cloned().map(|v| {
                Scheme::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("schemes", &v)?;
        }
        if true {
            struct_ser.serialize_field("deprecated", &self.deprecated)?;
        }
        if true {
            struct_ser.serialize_field("security", &self.security)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        if let Some(v) = self.parameters.as_ref() {
            struct_ser.serialize_field("parameters", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Operation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tags",
            "summary",
            "description",
            "external_docs",
            "externalDocs",
            "operation_id",
            "operationId",
            "consumes",
            "produces",
            "responses",
            "schemes",
            "deprecated",
            "security",
            "extensions",
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tags,
            Summary,
            Description,
            ExternalDocs,
            OperationId,
            Consumes,
            Produces,
            Responses,
            Schemes,
            Deprecated,
            Security,
            Extensions,
            Parameters,
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
                            "tags" => Ok(GeneratedField::Tags),
                            "summary" => Ok(GeneratedField::Summary),
                            "description" => Ok(GeneratedField::Description),
                            "externalDocs" | "external_docs" => Ok(GeneratedField::ExternalDocs),
                            "operationId" | "operation_id" => Ok(GeneratedField::OperationId),
                            "consumes" => Ok(GeneratedField::Consumes),
                            "produces" => Ok(GeneratedField::Produces),
                            "responses" => Ok(GeneratedField::Responses),
                            "schemes" => Ok(GeneratedField::Schemes),
                            "deprecated" => Ok(GeneratedField::Deprecated),
                            "security" => Ok(GeneratedField::Security),
                            "extensions" => Ok(GeneratedField::Extensions),
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Operation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Operation")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Operation, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tags__ = None;
                let mut summary__ = None;
                let mut description__ = None;
                let mut external_docs__ = None;
                let mut operation_id__ = None;
                let mut consumes__ = None;
                let mut produces__ = None;
                let mut responses__ = None;
                let mut schemes__ = None;
                let mut deprecated__ = None;
                let mut security__ = None;
                let mut extensions__ = None;
                let mut parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Tags => {
                            if tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExternalDocs => {
                            if external_docs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalDocs"));
                            }
                            external_docs__ = map_.next_value()?;
                        }
                        GeneratedField::OperationId => {
                            if operation_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operationId"));
                            }
                            operation_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Consumes => {
                            if consumes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consumes"));
                            }
                            consumes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Produces => {
                            if produces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("produces"));
                            }
                            produces__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Responses => {
                            if responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responses"));
                            }
                            responses__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Schemes => {
                            if schemes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemes"));
                            }
                            schemes__ = Some(map_.next_value::<Vec<Scheme>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::Deprecated => {
                            if deprecated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deprecated"));
                            }
                            deprecated__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Security => {
                            if security__.is_some() {
                                return Err(serde::de::Error::duplicate_field("security"));
                            }
                            security__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = map_.next_value()?;
                        }
                    }
                }
                Ok(Operation {
                    tags: tags__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    external_docs: external_docs__,
                    operation_id: operation_id__.unwrap_or_default(),
                    consumes: consumes__.unwrap_or_default(),
                    produces: produces__.unwrap_or_default(),
                    responses: responses__.unwrap_or_default(),
                    schemes: schemes__.unwrap_or_default(),
                    deprecated: deprecated__.unwrap_or_default(),
                    security: security__.unwrap_or_default(),
                    extensions: extensions__.unwrap_or_default(),
                    parameters: parameters__,
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Operation", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Parameters {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Parameters", len)?;
        if true {
            struct_ser.serialize_field("headers", &self.headers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Parameters {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "headers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Headers,
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
                            "headers" => Ok(GeneratedField::Headers),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Parameters;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Parameters")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Parameters, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut headers__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Headers => {
                            if headers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("headers"));
                            }
                            headers__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Parameters {
                    headers: headers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Parameters", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Response {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Response", len)?;
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.schema.as_ref() {
            struct_ser.serialize_field("schema", v)?;
        }
        if true {
            struct_ser.serialize_field("headers", &self.headers)?;
        }
        if true {
            struct_ser.serialize_field("examples", &self.examples)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Response {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "description",
            "schema",
            "headers",
            "examples",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Description,
            Schema,
            Headers,
            Examples,
            Extensions,
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
                            "description" => Ok(GeneratedField::Description),
                            "schema" => Ok(GeneratedField::Schema),
                            "headers" => Ok(GeneratedField::Headers),
                            "examples" => Ok(GeneratedField::Examples),
                            "extensions" => Ok(GeneratedField::Extensions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Response;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Response")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Response, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut description__ = None;
                let mut schema__ = None;
                let mut headers__ = None;
                let mut examples__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Schema => {
                            if schema__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schema"));
                            }
                            schema__ = map_.next_value()?;
                        }
                        GeneratedField::Headers => {
                            if headers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("headers"));
                            }
                            headers__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Examples => {
                            if examples__.is_some() {
                                return Err(serde::de::Error::duplicate_field("examples"));
                            }
                            examples__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Response {
                    description: description__.unwrap_or_default(),
                    schema: schema__,
                    headers: headers__.unwrap_or_default(),
                    examples: examples__.unwrap_or_default(),
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Response", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Schema {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Schema", len)?;
        if let Some(v) = self.json_schema.as_ref() {
            struct_ser.serialize_field("jsonSchema", v)?;
        }
        if true {
            struct_ser.serialize_field("discriminator", &self.discriminator)?;
        }
        if true {
            struct_ser.serialize_field("readOnly", &self.read_only)?;
        }
        if let Some(v) = self.external_docs.as_ref() {
            struct_ser.serialize_field("externalDocs", v)?;
        }
        if true {
            struct_ser.serialize_field("example", &self.example)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Schema {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "json_schema",
            "jsonSchema",
            "discriminator",
            "read_only",
            "readOnly",
            "external_docs",
            "externalDocs",
            "example",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            JsonSchema,
            Discriminator,
            ReadOnly,
            ExternalDocs,
            Example,
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
                            "jsonSchema" | "json_schema" => Ok(GeneratedField::JsonSchema),
                            "discriminator" => Ok(GeneratedField::Discriminator),
                            "readOnly" | "read_only" => Ok(GeneratedField::ReadOnly),
                            "externalDocs" | "external_docs" => Ok(GeneratedField::ExternalDocs),
                            "example" => Ok(GeneratedField::Example),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Schema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Schema")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Schema, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut json_schema__ = None;
                let mut discriminator__ = None;
                let mut read_only__ = None;
                let mut external_docs__ = None;
                let mut example__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::JsonSchema => {
                            if json_schema__.is_some() {
                                return Err(serde::de::Error::duplicate_field("jsonSchema"));
                            }
                            json_schema__ = map_.next_value()?;
                        }
                        GeneratedField::Discriminator => {
                            if discriminator__.is_some() {
                                return Err(serde::de::Error::duplicate_field("discriminator"));
                            }
                            discriminator__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ReadOnly => {
                            if read_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readOnly"));
                            }
                            read_only__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExternalDocs => {
                            if external_docs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalDocs"));
                            }
                            external_docs__ = map_.next_value()?;
                        }
                        GeneratedField::Example => {
                            if example__.is_some() {
                                return Err(serde::de::Error::duplicate_field("example"));
                            }
                            example__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Schema {
                    json_schema: json_schema__,
                    discriminator: discriminator__.unwrap_or_default(),
                    read_only: read_only__.unwrap_or_default(),
                    external_docs: external_docs__,
                    example: example__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Schema", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Scheme {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "UNKNOWN",
            Self::Http => "HTTP",
            Self::Https => "HTTPS",
            Self::Ws => "WS",
            Self::Wss => "WSS",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Scheme {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNKNOWN",
            "HTTP",
            "HTTPS",
            "WS",
            "WSS",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Scheme;

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
                    "UNKNOWN" => Ok(Scheme::Unknown),
                    "HTTP" => Ok(Scheme::Http),
                    "HTTPS" => Ok(Scheme::Https),
                    "WS" => Ok(Scheme::Ws),
                    "WSS" => Ok(Scheme::Wss),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Scopes {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Scopes", len)?;
        if true {
            struct_ser.serialize_field("scope", &self.scope)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Scopes {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scope",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scope,
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
                            "scope" => Ok(GeneratedField::Scope),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Scopes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Scopes")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Scopes, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scope__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scope => {
                            if scope__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scope"));
                            }
                            scope__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Scopes {
                    scope: scope__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Scopes", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SecurityDefinitions {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityDefinitions", len)?;
        if true {
            struct_ser.serialize_field("security", &self.security)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SecurityDefinitions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "security",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Security,
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
                            "security" => Ok(GeneratedField::Security),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SecurityDefinitions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.SecurityDefinitions")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SecurityDefinitions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut security__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Security => {
                            if security__.is_some() {
                                return Err(serde::de::Error::duplicate_field("security"));
                            }
                            security__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(SecurityDefinitions {
                    security: security__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityDefinitions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SecurityRequirement {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement", len)?;
        if true {
            struct_ser.serialize_field("securityRequirement", &self.security_requirement)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SecurityRequirement {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "security_requirement",
            "securityRequirement",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SecurityRequirement,
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
                            "securityRequirement" | "security_requirement" => Ok(GeneratedField::SecurityRequirement),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SecurityRequirement;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SecurityRequirement, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut security_requirement__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SecurityRequirement => {
                            if security_requirement__.is_some() {
                                return Err(serde::de::Error::duplicate_field("securityRequirement"));
                            }
                            security_requirement__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(SecurityRequirement {
                    security_requirement: security_requirement__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for security_requirement::SecurityRequirementValue {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement.SecurityRequirementValue", len)?;
        if true {
            struct_ser.serialize_field("scope", &self.scope)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for security_requirement::SecurityRequirementValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "scope",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Scope,
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
                            "scope" => Ok(GeneratedField::Scope),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = security_requirement::SecurityRequirementValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement.SecurityRequirementValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<security_requirement::SecurityRequirementValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut scope__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Scope => {
                            if scope__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scope"));
                            }
                            scope__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(security_requirement::SecurityRequirementValue {
                    scope: scope__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityRequirement.SecurityRequirementValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SecurityScheme {
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityScheme", len)?;
        if true {
            let v = security_scheme::Type::try_from(self.r#type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#type)))?;
            struct_ser.serialize_field("type", &v)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            let v = security_scheme::In::try_from(self.r#in)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.r#in)))?;
            struct_ser.serialize_field("in", &v)?;
        }
        if true {
            let v = security_scheme::Flow::try_from(self.flow)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.flow)))?;
            struct_ser.serialize_field("flow", &v)?;
        }
        if true {
            struct_ser.serialize_field("authorizationUrl", &self.authorization_url)?;
        }
        if true {
            struct_ser.serialize_field("tokenUrl", &self.token_url)?;
        }
        if let Some(v) = self.scopes.as_ref() {
            struct_ser.serialize_field("scopes", v)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SecurityScheme {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "type",
            "description",
            "name",
            "in",
            "flow",
            "authorization_url",
            "authorizationUrl",
            "token_url",
            "tokenUrl",
            "scopes",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Type,
            Description,
            Name,
            In,
            Flow,
            AuthorizationUrl,
            TokenUrl,
            Scopes,
            Extensions,
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
                            "description" => Ok(GeneratedField::Description),
                            "name" => Ok(GeneratedField::Name),
                            "in" => Ok(GeneratedField::In),
                            "flow" => Ok(GeneratedField::Flow),
                            "authorizationUrl" | "authorization_url" => Ok(GeneratedField::AuthorizationUrl),
                            "tokenUrl" | "token_url" => Ok(GeneratedField::TokenUrl),
                            "scopes" => Ok(GeneratedField::Scopes),
                            "extensions" => Ok(GeneratedField::Extensions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SecurityScheme;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.SecurityScheme")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<SecurityScheme, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut r#type__ = None;
                let mut description__ = None;
                let mut name__ = None;
                let mut r#in__ = None;
                let mut flow__ = None;
                let mut authorization_url__ = None;
                let mut token_url__ = None;
                let mut scopes__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value::<security_scheme::Type>()? as i32);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::In => {
                            if r#in__.is_some() {
                                return Err(serde::de::Error::duplicate_field("in"));
                            }
                            r#in__ = Some(map_.next_value::<security_scheme::In>()? as i32);
                        }
                        GeneratedField::Flow => {
                            if flow__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flow"));
                            }
                            flow__ = Some(map_.next_value::<security_scheme::Flow>()? as i32);
                        }
                        GeneratedField::AuthorizationUrl => {
                            if authorization_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authorizationUrl"));
                            }
                            authorization_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TokenUrl => {
                            if token_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tokenUrl"));
                            }
                            token_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Scopes => {
                            if scopes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("scopes"));
                            }
                            scopes__ = map_.next_value()?;
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(SecurityScheme {
                    r#type: r#type__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    r#in: r#in__.unwrap_or_default(),
                    flow: flow__.unwrap_or_default(),
                    authorization_url: authorization_url__.unwrap_or_default(),
                    token_url: token_url__.unwrap_or_default(),
                    scopes: scopes__,
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.SecurityScheme", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for security_scheme::Flow {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "FLOW_INVALID",
            Self::Implicit => "FLOW_IMPLICIT",
            Self::Password => "FLOW_PASSWORD",
            Self::Application => "FLOW_APPLICATION",
            Self::AccessCode => "FLOW_ACCESS_CODE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for security_scheme::Flow {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FLOW_INVALID",
            "FLOW_IMPLICIT",
            "FLOW_PASSWORD",
            "FLOW_APPLICATION",
            "FLOW_ACCESS_CODE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = security_scheme::Flow;

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
                    "FLOW_INVALID" => Ok(security_scheme::Flow::Invalid),
                    "FLOW_IMPLICIT" => Ok(security_scheme::Flow::Implicit),
                    "FLOW_PASSWORD" => Ok(security_scheme::Flow::Password),
                    "FLOW_APPLICATION" => Ok(security_scheme::Flow::Application),
                    "FLOW_ACCESS_CODE" => Ok(security_scheme::Flow::AccessCode),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for security_scheme::In {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "IN_INVALID",
            Self::Query => "IN_QUERY",
            Self::Header => "IN_HEADER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for security_scheme::In {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "IN_INVALID",
            "IN_QUERY",
            "IN_HEADER",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = security_scheme::In;

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
                    "IN_INVALID" => Ok(security_scheme::In::Invalid),
                    "IN_QUERY" => Ok(security_scheme::In::Query),
                    "IN_HEADER" => Ok(security_scheme::In::Header),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for security_scheme::Type {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Invalid => "TYPE_INVALID",
            Self::Basic => "TYPE_BASIC",
            Self::ApiKey => "TYPE_API_KEY",
            Self::Oauth2 => "TYPE_OAUTH2",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for security_scheme::Type {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "TYPE_INVALID",
            "TYPE_BASIC",
            "TYPE_API_KEY",
            "TYPE_OAUTH2",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = security_scheme::Type;

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
                    "TYPE_INVALID" => Ok(security_scheme::Type::Invalid),
                    "TYPE_BASIC" => Ok(security_scheme::Type::Basic),
                    "TYPE_API_KEY" => Ok(security_scheme::Type::ApiKey),
                    "TYPE_OAUTH2" => Ok(security_scheme::Type::Oauth2),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Swagger {
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
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        if true {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Swagger", len)?;
        if true {
            struct_ser.serialize_field("swagger", &self.swagger)?;
        }
        if let Some(v) = self.info.as_ref() {
            struct_ser.serialize_field("info", v)?;
        }
        if true {
            struct_ser.serialize_field("host", &self.host)?;
        }
        if true {
            struct_ser.serialize_field("basePath", &self.base_path)?;
        }
        if true {
            let v = self.schemes.iter().cloned().map(|v| {
                Scheme::try_from(v)
                    .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", v)))
                }).collect::<std::result::Result<Vec<_>, _>>()?;
            struct_ser.serialize_field("schemes", &v)?;
        }
        if true {
            struct_ser.serialize_field("consumes", &self.consumes)?;
        }
        if true {
            struct_ser.serialize_field("produces", &self.produces)?;
        }
        if true {
            struct_ser.serialize_field("responses", &self.responses)?;
        }
        if let Some(v) = self.security_definitions.as_ref() {
            struct_ser.serialize_field("securityDefinitions", v)?;
        }
        if true {
            struct_ser.serialize_field("security", &self.security)?;
        }
        if true {
            struct_ser.serialize_field("tags", &self.tags)?;
        }
        if let Some(v) = self.external_docs.as_ref() {
            struct_ser.serialize_field("externalDocs", v)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Swagger {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "swagger",
            "info",
            "host",
            "base_path",
            "basePath",
            "schemes",
            "consumes",
            "produces",
            "responses",
            "security_definitions",
            "securityDefinitions",
            "security",
            "tags",
            "external_docs",
            "externalDocs",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Swagger,
            Info,
            Host,
            BasePath,
            Schemes,
            Consumes,
            Produces,
            Responses,
            SecurityDefinitions,
            Security,
            Tags,
            ExternalDocs,
            Extensions,
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
                            "swagger" => Ok(GeneratedField::Swagger),
                            "info" => Ok(GeneratedField::Info),
                            "host" => Ok(GeneratedField::Host),
                            "basePath" | "base_path" => Ok(GeneratedField::BasePath),
                            "schemes" => Ok(GeneratedField::Schemes),
                            "consumes" => Ok(GeneratedField::Consumes),
                            "produces" => Ok(GeneratedField::Produces),
                            "responses" => Ok(GeneratedField::Responses),
                            "securityDefinitions" | "security_definitions" => Ok(GeneratedField::SecurityDefinitions),
                            "security" => Ok(GeneratedField::Security),
                            "tags" => Ok(GeneratedField::Tags),
                            "externalDocs" | "external_docs" => Ok(GeneratedField::ExternalDocs),
                            "extensions" => Ok(GeneratedField::Extensions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Swagger;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Swagger")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Swagger, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut swagger__ = None;
                let mut info__ = None;
                let mut host__ = None;
                let mut base_path__ = None;
                let mut schemes__ = None;
                let mut consumes__ = None;
                let mut produces__ = None;
                let mut responses__ = None;
                let mut security_definitions__ = None;
                let mut security__ = None;
                let mut tags__ = None;
                let mut external_docs__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Swagger => {
                            if swagger__.is_some() {
                                return Err(serde::de::Error::duplicate_field("swagger"));
                            }
                            swagger__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = map_.next_value()?;
                        }
                        GeneratedField::Host => {
                            if host__.is_some() {
                                return Err(serde::de::Error::duplicate_field("host"));
                            }
                            host__ = Some(map_.next_value()?);
                        }
                        GeneratedField::BasePath => {
                            if base_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("basePath"));
                            }
                            base_path__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Schemes => {
                            if schemes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemes"));
                            }
                            schemes__ = Some(map_.next_value::<Vec<Scheme>>()?.into_iter().map(|x| x as i32).collect());
                        }
                        GeneratedField::Consumes => {
                            if consumes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("consumes"));
                            }
                            consumes__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Produces => {
                            if produces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("produces"));
                            }
                            produces__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Responses => {
                            if responses__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responses"));
                            }
                            responses__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::SecurityDefinitions => {
                            if security_definitions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("securityDefinitions"));
                            }
                            security_definitions__ = map_.next_value()?;
                        }
                        GeneratedField::Security => {
                            if security__.is_some() {
                                return Err(serde::de::Error::duplicate_field("security"));
                            }
                            security__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Tags => {
                            if tags__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tags"));
                            }
                            tags__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExternalDocs => {
                            if external_docs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalDocs"));
                            }
                            external_docs__ = map_.next_value()?;
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Swagger {
                    swagger: swagger__.unwrap_or_default(),
                    info: info__,
                    host: host__.unwrap_or_default(),
                    base_path: base_path__.unwrap_or_default(),
                    schemes: schemes__.unwrap_or_default(),
                    consumes: consumes__.unwrap_or_default(),
                    produces: produces__.unwrap_or_default(),
                    responses: responses__.unwrap_or_default(),
                    security_definitions: security_definitions__,
                    security: security__.unwrap_or_default(),
                    tags: tags__.unwrap_or_default(),
                    external_docs: external_docs__,
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Swagger", FIELDS, GeneratedVisitor)
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
        let mut struct_ser = serializer.serialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Tag", len)?;
        if true {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if true {
            struct_ser.serialize_field("description", &self.description)?;
        }
        if let Some(v) = self.external_docs.as_ref() {
            struct_ser.serialize_field("externalDocs", v)?;
        }
        if true {
            struct_ser.serialize_field("extensions", &self.extensions)?;
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
            "name",
            "description",
            "external_docs",
            "externalDocs",
            "extensions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Description,
            ExternalDocs,
            Extensions,
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
                            "name" => Ok(GeneratedField::Name),
                            "description" => Ok(GeneratedField::Description),
                            "externalDocs" | "external_docs" => Ok(GeneratedField::ExternalDocs),
                            "extensions" => Ok(GeneratedField::Extensions),
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
                formatter.write_str("struct grpc.gateway.protoc_gen_openapiv2.options.Tag")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Tag, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut description__ = None;
                let mut external_docs__ = None;
                let mut extensions__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExternalDocs => {
                            if external_docs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalDocs"));
                            }
                            external_docs__ = map_.next_value()?;
                        }
                        GeneratedField::Extensions => {
                            if extensions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("extensions"));
                            }
                            extensions__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(Tag {
                    name: name__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                    external_docs: external_docs__,
                    extensions: extensions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("grpc.gateway.protoc_gen_openapiv2.options.Tag", FIELDS, GeneratedVisitor)
    }
}
