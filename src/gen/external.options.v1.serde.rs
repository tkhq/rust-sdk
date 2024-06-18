impl serde::Serialize for Pagination {
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
        let mut struct_ser = serializer.serialize_struct("external.options.v1.Pagination", len)?;
        if true {
            struct_ser.serialize_field("limit", &self.limit)?;
        }
        if true {
            struct_ser.serialize_field("before", &self.before)?;
        }
        if true {
            struct_ser.serialize_field("after", &self.after)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Pagination {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "limit",
            "before",
            "after",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Limit,
            Before,
            After,
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
                            "limit" => Ok(GeneratedField::Limit),
                            "before" => Ok(GeneratedField::Before),
                            "after" => Ok(GeneratedField::After),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Pagination;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct external.options.v1.Pagination")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Pagination, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut limit__ = None;
                let mut before__ = None;
                let mut after__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Limit => {
                            if limit__.is_some() {
                                return Err(serde::de::Error::duplicate_field("limit"));
                            }
                            limit__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Before => {
                            if before__.is_some() {
                                return Err(serde::de::Error::duplicate_field("before"));
                            }
                            before__ = Some(map_.next_value()?);
                        }
                        GeneratedField::After => {
                            if after__.is_some() {
                                return Err(serde::de::Error::duplicate_field("after"));
                            }
                            after__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(Pagination {
                    limit: limit__.unwrap_or_default(),
                    before: before__.unwrap_or_default(),
                    after: after__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("external.options.v1.Pagination", FIELDS, GeneratedVisitor)
    }
}
