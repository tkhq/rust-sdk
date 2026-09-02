//! Typed placeholder values used by configuration templates.

use serde::{
    Deserialize, Serialize,
    de::{Error as _, Unexpected},
};
use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// Supplies the fixed placeholder text for a configuration field.
pub trait PlaceholderText {
    const TEXT: &'static str;
}

/// A configuration value that is either populated or still a placeholder.
#[derive(PartialEq, Eq)]
pub enum WithPlaceholder<T, P> {
    Value(T),
    Placeholder(PhantomData<P>),
}

impl<T, P: PlaceholderText> WithPlaceholder<T, P> {
    pub const TEXT: &'static str = P::TEXT;

    pub fn placeholder() -> Self {
        Self::Placeholder(PhantomData)
    }

    /// Try to convert into the value. [`Result::Err`] variant is the placeholder text
    pub fn try_into(self) -> Result<T, &'static str> {
        match self {
            WithPlaceholder::Value(val) => Ok(val),
            WithPlaceholder::Placeholder(_) => Err(P::TEXT),
        }
    }
}

impl<T: Clone, P> Clone for WithPlaceholder<T, P> {
    fn clone(&self) -> Self {
        match self {
            Self::Value(val) => Self::Value(val.clone()),
            Self::Placeholder(_) => Self::Placeholder(PhantomData),
        }
    }
}

impl<T: Copy, P> Copy for WithPlaceholder<T, P> {}

impl<T, P> Default for WithPlaceholder<T, P> {
    fn default() -> Self {
        Self::Placeholder(PhantomData)
    }
}

impl<T: Debug, P> Debug for WithPlaceholder<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(value) => f
                .debug_tuple("WithPlaceholder::Value")
                .field(value)
                .finish(),
            Self::Placeholder(_) => f.debug_tuple("WithPlaceholder::Placeholder").finish(),
        }
    }
}

impl<T: Display, P: PlaceholderText> Display for WithPlaceholder<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WithPlaceholder::Value(val) => Display::fmt(val, f),
            WithPlaceholder::Placeholder(_) => f.write_fmt(format_args!("<{}>", P::TEXT)),
        }
    }
}

impl<T, P> From<T> for WithPlaceholder<T, P> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Serialize, P: PlaceholderText> Serialize for WithPlaceholder<T, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            WithPlaceholder::Value(val) => val.serialize(serializer),
            WithPlaceholder::Placeholder(_) => {
                serializer.collect_str(&format_args!("<{}>", P::TEXT))
            }
        }
    }
}

impl<'de, T: Deserialize<'de>, P: PlaceholderText> Deserialize<'de> for WithPlaceholder<T, P> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        /// T cannot be a `String` or `&str` here, it won't work properly
        enum StringOrValue<'a, T> {
            /// Serde derive implementation attempts to deserialize into the first variant first
            /// for untagged enums
            Value(T),
            /// Not every deserializer will borrow here properly
            #[serde(borrow)]
            String(Cow<'a, str>),
        }

        StringOrValue::<'de, T>::deserialize(deserializer).and_then(|string_or_value| {
            match string_or_value {
                StringOrValue::Value(value) => Ok(Self::Value(value)),
                StringOrValue::String(s) => {
                    let is_match = s.len() == P::TEXT.len() + 2
                        && s.starts_with('<')
                        && s.ends_with('>')
                        && &s[1..s.len() - 1] == P::TEXT;

                    if is_match {
                        Ok(Self::Placeholder(PhantomData))
                    } else {
                        Err(D::Error::invalid_type(
                            Unexpected::Str(s.as_ref()),
                            &format!("only {} is valid as a placeholder", P::TEXT)
                                .to_string()
                                .as_str(),
                        ))
                    }
                }
            }
        })
    }
}

pub type StringWithPlaceholder<P> = WithPlaceholder<NonPlaceHolderString, P>;

#[derive(Debug, Default, Serialize, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct NonPlaceHolderString(String);

impl Deref for NonPlaceHolderString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NonPlaceHolderString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for NonPlaceHolderString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<String> for NonPlaceHolderString {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl<P> From<String> for StringWithPlaceholder<P> {
    fn from(value: String) -> Self {
        NonPlaceHolderString::from(value).into()
    }
}

impl From<&'static str> for NonPlaceHolderString {
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl<P> From<&'static str> for StringWithPlaceholder<P> {
    fn from(value: &'static str) -> Self {
        NonPlaceHolderString::from(value).into()
    }
}

impl<'de> Deserialize<'de> for NonPlaceHolderString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let borrowed = Cow::<'de, str>::deserialize(deserializer)?;

        let mut chars = borrowed.chars();

        match (chars.next(), chars.last()) {
            (Some('<'), Some('>')) => Err(D::Error::custom(
                "String be surrounded by placeholder brackets '<' and '>'",
            )),
            _ => Ok(Self(borrowed.into_owned())),
        }
    }
}

pub mod text {
    use super::PlaceholderText;

    pub struct FillInAppName;
    pub struct FillInManifestSetName;
    pub struct FillInOperatorPublicKey;
    pub struct RemoveMeIfPivotContainerUrlIsPublic;
    pub struct FillInAppId;
    pub struct FillInPivotContainerImageUrl;
    pub struct FillInPivotPath;
    pub struct FillInExpectedPivotDigest;

    impl PlaceholderText for FillInAppName {
        const TEXT: &'static str = "FILL_IN_APP_NAME";
    }

    impl PlaceholderText for FillInManifestSetName {
        const TEXT: &'static str = "FILL_IN_MANIFEST_SET_NAME";
    }

    impl PlaceholderText for FillInOperatorPublicKey {
        const TEXT: &'static str = "FILL_IN_OPERATOR_PUBLIC_KEY";
    }

    impl PlaceholderText for RemoveMeIfPivotContainerUrlIsPublic {
        const TEXT: &'static str = "REMOVE_ME_IF_PIVOT_CONTAINER_URL_IS_PUBLIC";
    }

    impl PlaceholderText for FillInAppId {
        const TEXT: &'static str = "FILL_IN_APP_ID";
    }

    impl PlaceholderText for FillInPivotContainerImageUrl {
        const TEXT: &'static str = "FILL_IN_PIVOT_CONTAINER_IMAGE_URL";
    }

    impl PlaceholderText for FillInPivotPath {
        const TEXT: &'static str = "FILL_IN_PIVOT_PATH";
    }

    impl PlaceholderText for FillInExpectedPivotDigest {
        const TEXT: &'static str = "FILL_IN_EXPECTED_PIVOT_DIGEST";
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, de::DeserializeOwned};
    use std::fmt::Debug;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct TestPlaceholder;

    impl PlaceholderText for TestPlaceholder {
        const TEXT: &'static str = "TEST_PLACEHOLDER";
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Outer<T> {
        value: T,
    }

    fn assert_json_serializes_value<T>(value: T, json_value: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let value = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Value(value),
        };
        let expected_json = format!(r#"{{"value":{json_value}}}"#);

        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, expected_json);

        let round_trip: Outer<WithPlaceholder<T, TestPlaceholder>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, value);
    }

    fn assert_json_deserializes_value<T>(value: T, json_value: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let expected = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Value(value),
        };
        let json = format!(r#"{{"value":{json_value}}}"#);

        let deserialized: Outer<WithPlaceholder<T, TestPlaceholder>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, expected);

        let round_trip = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(round_trip, json);
    }

    fn assert_toml_serializes_value<T>(value: T, expected_toml: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let value = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Value(value),
        };

        let toml = toml::to_string(&value).unwrap();
        assert_eq!(toml, expected_toml);

        let round_trip: Outer<WithPlaceholder<T, TestPlaceholder>> = toml::from_str(&toml).unwrap();
        assert_eq!(round_trip, value);
    }

    fn assert_toml_deserializes_value<T>(value: T, toml: &str)
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let expected = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Value(value),
        };

        let deserialized: Outer<WithPlaceholder<T, TestPlaceholder>> =
            toml::from_str(toml).unwrap();
        assert_eq!(deserialized, expected);

        let round_trip = toml::to_string(&deserialized).unwrap();
        assert_eq!(round_trip, toml);
    }

    fn assert_json_serializes_placeholder<T>()
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let value = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Placeholder(PhantomData),
        };
        let expected_json = format!(r#"{{"value":"<{}>"}}"#, TestPlaceholder::TEXT);

        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(json, expected_json);

        let round_trip: Outer<WithPlaceholder<T, TestPlaceholder>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, value);
    }

    fn assert_json_deserializes_placeholder<T>()
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let expected = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Placeholder(PhantomData),
        };
        let json = format!(r#"{{"value":"<{}>"}}"#, TestPlaceholder::TEXT);

        let deserialized: Outer<WithPlaceholder<T, TestPlaceholder>> =
            serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, expected);

        let round_trip = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(round_trip, json);
    }

    fn assert_toml_serializes_placeholder<T>()
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let value = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Placeholder(PhantomData),
        };
        let expected_toml = format!(
            r#"value = "<{}>"
"#,
            TestPlaceholder::TEXT
        );

        let toml = toml::to_string(&value).unwrap();
        assert_eq!(toml, expected_toml);

        let round_trip: Outer<WithPlaceholder<T, TestPlaceholder>> = toml::from_str(&toml).unwrap();
        assert_eq!(round_trip, value);
    }

    fn assert_toml_deserializes_placeholder<T>()
    where
        T: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let expected = Outer {
            value: WithPlaceholder::<T, TestPlaceholder>::Placeholder(PhantomData),
        };
        let toml = format!(
            r#"value = "<{}>"
"#,
            TestPlaceholder::TEXT
        );

        let deserialized: Outer<WithPlaceholder<T, TestPlaceholder>> =
            toml::from_str(&toml).unwrap();
        assert_eq!(deserialized, expected);

        let round_trip = toml::to_string(&deserialized).unwrap();
        assert_eq!(round_trip, toml);
    }

    macro_rules! primitive_test {
        ($module:ident, $ty:ty, $value:expr, $json:literal, $toml:literal) => {
            mod $module {
                use super::*;

                #[test]
                fn json_serializes_value() {
                    assert_json_serializes_value::<$ty>($value, $json);
                }

                #[test]
                fn json_deserializes_value() {
                    assert_json_deserializes_value::<$ty>($value, $json);
                }

                #[test]
                fn toml_serializes_value() {
                    assert_toml_serializes_value::<$ty>($value, $toml);
                }

                #[test]
                fn toml_deserializes_value() {
                    assert_toml_deserializes_value::<$ty>($value, $toml);
                }

                #[test]
                fn json_serializes_placeholder() {
                    assert_json_serializes_placeholder::<$ty>();
                }

                #[test]
                fn json_deserializes_placeholder() {
                    assert_json_deserializes_placeholder::<$ty>();
                }

                #[test]
                fn toml_serializes_placeholder() {
                    assert_toml_serializes_placeholder::<$ty>();
                }

                #[test]
                fn toml_deserializes_placeholder() {
                    assert_toml_deserializes_placeholder::<$ty>();
                }
            }
        };
    }

    primitive_test!(bool_value, bool, true, "true", "value = true\n");
    primitive_test!(char_value, char, 'x', r#""x""#, "value = \"x\"\n");
    primitive_test!(i8_value, i8, -8, "-8", "value = -8\n");
    primitive_test!(i16_value, i16, -16, "-16", "value = -16\n");
    primitive_test!(i32_value, i32, -32, "-32", "value = -32\n");
    primitive_test!(i64_value, i64, -64, "-64", "value = -64\n");
    primitive_test!(isize_value, isize, -42, "-42", "value = -42\n");
    primitive_test!(u8_value, u8, 8, "8", "value = 8\n");
    primitive_test!(u16_value, u16, 16, "16", "value = 16\n");
    primitive_test!(u32_value, u32, 32, "32", "value = 32\n");
    primitive_test!(u64_value, u64, 64, "64", "value = 64\n");
    primitive_test!(usize_value, usize, 42, "42", "value = 42\n");
    primitive_test!(f32_value, f32, 1.25, "1.25", "value = 1.25\n");
    primitive_test!(f64_value, f64, -2.5, "-2.5", "value = -2.5\n");

    mod unit_value {
        use super::*;

        #[test]
        fn json_serializes_value() {
            assert_json_serializes_value::<()>((), "null");
        }

        #[test]
        fn json_deserializes_value() {
            assert_json_deserializes_value::<()>((), "null");
        }

        #[test]
        fn json_serializes_placeholder() {
            assert_json_serializes_placeholder::<()>();
        }

        #[test]
        fn json_deserializes_placeholder() {
            assert_json_deserializes_placeholder::<()>();
        }

        #[test]
        fn toml_serializes_placeholder() {
            assert_toml_serializes_placeholder::<()>();
        }

        #[test]
        fn toml_deserializes_placeholder() {
            assert_toml_deserializes_placeholder::<()>();
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct ExampleStruct {
        enabled: bool,
        retries: u16,
    }

    primitive_test!(
        struct_value,
        ExampleStruct,
        ExampleStruct {
            enabled: true,
            retries: 3,
        },
        r#"{"enabled":true,"retries":3}"#,
        r#"[value]
enabled = true
retries = 3
"#
    );

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum ExampleEnum {
        Ready,
        Waiting,
    }

    primitive_test!(
        enum_value,
        ExampleEnum,
        ExampleEnum::Ready,
        r#""ready""#,
        "value = \"ready\"\n"
    );

    primitive_test!(
        optional_primitive_some,
        Option<u16>,
        Some(16),
        "16",
        "value = 16\n"
    );

    mod optional_primitive_none {
        use super::*;

        #[test]
        fn json_serializes_value() {
            assert_json_serializes_value::<Option<u16>>(None, "null");
        }

        #[test]
        fn json_deserializes_value() {
            assert_json_deserializes_value::<Option<u16>>(None, "null");
        }
    }

    mod placeholder_string {
        use super::*;

        const STRING_VALUE: &str = TestPlaceholder::TEXT;
        type TestString = StringWithPlaceholder<TestPlaceholder>;

        fn value() -> TestString {
            WithPlaceholder::Value(NonPlaceHolderString(STRING_VALUE.to_owned()))
        }

        fn assert_json_serializes_value() {
            let value = Outer { value: value() };
            let expected_json = format!(r#"{{"value":"{STRING_VALUE}"}}"#);

            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(json, expected_json);

            let round_trip: Outer<TestString> = serde_json::from_str(&json).unwrap();
            assert_eq!(round_trip, value);
        }

        fn assert_json_deserializes_value() {
            let expected = Outer { value: value() };
            let json = format!(r#"{{"value":"{STRING_VALUE}"}}"#);

            let deserialized: Outer<TestString> = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized, expected);

            let round_trip = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(round_trip, json);
        }

        fn assert_toml_serializes_value() {
            let value = Outer { value: value() };
            let expected_toml = format!(
                r#"value = "{STRING_VALUE}"
"#
            );

            let toml = toml::to_string(&value).unwrap();
            assert_eq!(toml, expected_toml);

            let round_trip: Outer<TestString> = toml::from_str(&toml).unwrap();
            assert_eq!(round_trip, value);
        }

        fn assert_toml_deserializes_value() {
            let expected = Outer { value: value() };
            let toml = format!(
                r#"value = "{STRING_VALUE}"
"#
            );

            let deserialized: Outer<TestString> = toml::from_str(&toml).unwrap();
            assert_eq!(deserialized, expected);

            let round_trip = toml::to_string(&deserialized).unwrap();
            assert_eq!(round_trip, toml);
        }

        #[test]
        fn json_serializes_value() {
            assert_json_serializes_value();
        }

        #[test]
        fn json_deserializes_value() {
            assert_json_deserializes_value();
        }

        #[test]
        fn toml_serializes_value() {
            assert_toml_serializes_value();
        }

        #[test]
        fn toml_deserializes_value() {
            assert_toml_deserializes_value();
        }

        #[test]
        fn json_serializes_placeholder() {
            super::assert_json_serializes_placeholder::<NonPlaceHolderString>();
        }

        #[test]
        fn json_deserializes_placeholder() {
            super::assert_json_deserializes_placeholder::<NonPlaceHolderString>();
        }

        #[test]
        fn toml_serializes_placeholder() {
            super::assert_toml_serializes_placeholder::<NonPlaceHolderString>();
        }

        #[test]
        fn toml_deserializes_placeholder() {
            super::assert_toml_deserializes_placeholder::<NonPlaceHolderString>();
        }

        #[test]
        fn json_rejects_different_bracketed_placeholder() {
            let json = r#"{"value":"<OTHER_PLACEHOLDER>"}"#;

            assert!(serde_json::from_str::<Outer<TestString>>(json).is_err());
        }

        #[test]
        fn toml_rejects_different_bracketed_placeholder() {
            let toml = r#"value = "<OTHER_PLACEHOLDER>"
"#;

            assert!(toml::from_str::<Outer<TestString>>(toml).is_err());
        }
    }
}
