impl serde::Serialize for CustomHttpPattern {
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
        let mut struct_ser = serializer.serialize_struct("google.api.CustomHttpPattern", len)?;
        if true {
            struct_ser.serialize_field("kind", &self.kind)?;
        }
        if true {
            struct_ser.serialize_field("path", &self.path)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CustomHttpPattern {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "kind",
            "path",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Kind,
            Path,
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
                            "kind" => Ok(GeneratedField::Kind),
                            "path" => Ok(GeneratedField::Path),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CustomHttpPattern;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.api.CustomHttpPattern")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CustomHttpPattern, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut kind__ = None;
                let mut path__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Kind => {
                            if kind__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kind"));
                            }
                            kind__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Path => {
                            if path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(CustomHttpPattern {
                    kind: kind__.unwrap_or_default(),
                    path: path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.api.CustomHttpPattern", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FieldBehavior {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "FIELD_BEHAVIOR_UNSPECIFIED",
            Self::Optional => "OPTIONAL",
            Self::Required => "REQUIRED",
            Self::OutputOnly => "OUTPUT_ONLY",
            Self::InputOnly => "INPUT_ONLY",
            Self::Immutable => "IMMUTABLE",
            Self::UnorderedList => "UNORDERED_LIST",
            Self::NonEmptyDefault => "NON_EMPTY_DEFAULT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for FieldBehavior {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FIELD_BEHAVIOR_UNSPECIFIED",
            "OPTIONAL",
            "REQUIRED",
            "OUTPUT_ONLY",
            "INPUT_ONLY",
            "IMMUTABLE",
            "UNORDERED_LIST",
            "NON_EMPTY_DEFAULT",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FieldBehavior;

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
                    "FIELD_BEHAVIOR_UNSPECIFIED" => Ok(FieldBehavior::Unspecified),
                    "OPTIONAL" => Ok(FieldBehavior::Optional),
                    "REQUIRED" => Ok(FieldBehavior::Required),
                    "OUTPUT_ONLY" => Ok(FieldBehavior::OutputOnly),
                    "INPUT_ONLY" => Ok(FieldBehavior::InputOnly),
                    "IMMUTABLE" => Ok(FieldBehavior::Immutable),
                    "UNORDERED_LIST" => Ok(FieldBehavior::UnorderedList),
                    "NON_EMPTY_DEFAULT" => Ok(FieldBehavior::NonEmptyDefault),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Http {
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
        let mut struct_ser = serializer.serialize_struct("google.api.Http", len)?;
        if true {
            struct_ser.serialize_field("rules", &self.rules)?;
        }
        if true {
            struct_ser.serialize_field("fullyDecodeReservedExpansion", &self.fully_decode_reserved_expansion)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Http {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rules",
            "fully_decode_reserved_expansion",
            "fullyDecodeReservedExpansion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rules,
            FullyDecodeReservedExpansion,
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
                            "rules" => Ok(GeneratedField::Rules),
                            "fullyDecodeReservedExpansion" | "fully_decode_reserved_expansion" => Ok(GeneratedField::FullyDecodeReservedExpansion),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Http;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.api.Http")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Http, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rules__ = None;
                let mut fully_decode_reserved_expansion__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Rules => {
                            if rules__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rules"));
                            }
                            rules__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullyDecodeReservedExpansion => {
                            if fully_decode_reserved_expansion__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullyDecodeReservedExpansion"));
                            }
                            fully_decode_reserved_expansion__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Http {
                    rules: rules__.unwrap_or_default(),
                    fully_decode_reserved_expansion: fully_decode_reserved_expansion__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.api.Http", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for HttpRule {
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
        if self.pattern.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("google.api.HttpRule", len)?;
        if true {
            struct_ser.serialize_field("selector", &self.selector)?;
        }
        if true {
            struct_ser.serialize_field("body", &self.body)?;
        }
        if true {
            struct_ser.serialize_field("responseBody", &self.response_body)?;
        }
        if true {
            struct_ser.serialize_field("additionalBindings", &self.additional_bindings)?;
        }
        if let Some(v) = self.pattern.as_ref() {
            match v {
                http_rule::Pattern::Get(v) => {
                    struct_ser.serialize_field("get", v)?;
                }
                http_rule::Pattern::Put(v) => {
                    struct_ser.serialize_field("put", v)?;
                }
                http_rule::Pattern::Post(v) => {
                    struct_ser.serialize_field("post", v)?;
                }
                http_rule::Pattern::Delete(v) => {
                    struct_ser.serialize_field("delete", v)?;
                }
                http_rule::Pattern::Patch(v) => {
                    struct_ser.serialize_field("patch", v)?;
                }
                http_rule::Pattern::Custom(v) => {
                    struct_ser.serialize_field("custom", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for HttpRule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "selector",
            "body",
            "response_body",
            "responseBody",
            "additional_bindings",
            "additionalBindings",
            "get",
            "put",
            "post",
            "delete",
            "patch",
            "custom",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Selector,
            Body,
            ResponseBody,
            AdditionalBindings,
            Get,
            Put,
            Post,
            Delete,
            Patch,
            Custom,
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
                            "selector" => Ok(GeneratedField::Selector),
                            "body" => Ok(GeneratedField::Body),
                            "responseBody" | "response_body" => Ok(GeneratedField::ResponseBody),
                            "additionalBindings" | "additional_bindings" => Ok(GeneratedField::AdditionalBindings),
                            "get" => Ok(GeneratedField::Get),
                            "put" => Ok(GeneratedField::Put),
                            "post" => Ok(GeneratedField::Post),
                            "delete" => Ok(GeneratedField::Delete),
                            "patch" => Ok(GeneratedField::Patch),
                            "custom" => Ok(GeneratedField::Custom),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = HttpRule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.api.HttpRule")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<HttpRule, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut selector__ = None;
                let mut body__ = None;
                let mut response_body__ = None;
                let mut additional_bindings__ = None;
                let mut pattern__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Selector => {
                            if selector__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selector"));
                            }
                            selector__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Body => {
                            if body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("body"));
                            }
                            body__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ResponseBody => {
                            if response_body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("responseBody"));
                            }
                            response_body__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AdditionalBindings => {
                            if additional_bindings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("additionalBindings"));
                            }
                            additional_bindings__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Get => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("get"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Get);
                        }
                        GeneratedField::Put => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("put"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Put);
                        }
                        GeneratedField::Post => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("post"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Post);
                        }
                        GeneratedField::Delete => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("delete"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Delete);
                        }
                        GeneratedField::Patch => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("patch"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Patch);
                        }
                        GeneratedField::Custom => {
                            if pattern__.is_some() {
                                return Err(serde::de::Error::duplicate_field("custom"));
                            }
                            pattern__ = map_.next_value::<::std::option::Option<_>>()?.map(http_rule::Pattern::Custom)
;
                        }
                    }
                }
                Ok(HttpRule {
                    selector: selector__.unwrap_or_default(),
                    body: body__.unwrap_or_default(),
                    response_body: response_body__.unwrap_or_default(),
                    additional_bindings: additional_bindings__.unwrap_or_default(),
                    pattern: pattern__,
                })
            }
        }
        deserializer.deserialize_struct("google.api.HttpRule", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Visibility {
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
        let mut struct_ser = serializer.serialize_struct("google.api.Visibility", len)?;
        if true {
            struct_ser.serialize_field("rules", &self.rules)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Visibility {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rules",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Rules,
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
                            "rules" => Ok(GeneratedField::Rules),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Visibility;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.api.Visibility")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Visibility, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rules__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Rules => {
                            if rules__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rules"));
                            }
                            rules__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Visibility {
                    rules: rules__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.api.Visibility", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VisibilityRule {
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
        let mut struct_ser = serializer.serialize_struct("google.api.VisibilityRule", len)?;
        if true {
            struct_ser.serialize_field("selector", &self.selector)?;
        }
        if true {
            struct_ser.serialize_field("restriction", &self.restriction)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for VisibilityRule {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "selector",
            "restriction",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Selector,
            Restriction,
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
                            "selector" => Ok(GeneratedField::Selector),
                            "restriction" => Ok(GeneratedField::Restriction),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VisibilityRule;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct google.api.VisibilityRule")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<VisibilityRule, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut selector__ = None;
                let mut restriction__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Selector => {
                            if selector__.is_some() {
                                return Err(serde::de::Error::duplicate_field("selector"));
                            }
                            selector__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Restriction => {
                            if restriction__.is_some() {
                                return Err(serde::de::Error::duplicate_field("restriction"));
                            }
                            restriction__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(VisibilityRule {
                    selector: selector__.unwrap_or_default(),
                    restriction: restriction__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("google.api.VisibilityRule", FIELDS, GeneratedVisitor)
    }
}
