//! Hand-vendored `google.protobuf` well-known types used by the generated code.

use chrono::{DateTime, Utc};

/// `google.protobuf.Timestamp`.
///
/// (De)serializes as an RFC 3339 string per the proto3 JSON mapping.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    /// Seconds of UTC time since Unix epoch.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

impl TryFrom<&Timestamp> for DateTime<Utc> {
    type Error = &'static str;

    fn try_from(value: &Timestamp) -> Result<Self, Self::Error> {
        let nanos = u32::try_from(value.nanos).map_err(|_| "timestamp nanos out of range")?;
        DateTime::from_timestamp(value.seconds, nanos).ok_or("invalid or out-of-range timestamp")
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(value: DateTime<Utc>) -> Self {
        Self {
            seconds: value.timestamp(),
            nanos: value.timestamp_subsec_nanos() as i32,
        }
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let dt: DateTime<Utc> = self.try_into().map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&dt.to_rfc3339())
    }
}

struct TimestampVisitor;

impl serde::de::Visitor<'_> for TimestampVisitor {
    type Value = Timestamp;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("an RFC 3339 date string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let dt = DateTime::parse_from_rfc3339(s).map_err(serde::de::Error::custom)?;
        Ok(dt.with_timezone(&Utc).into())
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TimestampVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_rfc3339_string() {
        let ts = Timestamp {
            seconds: 1478621229,
            nanos: 500_000_000,
        };
        let encoded = serde_json::to_string(&ts).unwrap();
        assert_eq!(encoded, "\"2016-11-08T16:07:09.500+00:00\"");
    }

    #[test]
    fn deserializes_z_suffix_and_offset_forms() {
        for input in [
            "\"2016-11-08T16:07:09Z\"",
            "\"2016-11-08T21:07:09+05:00\"",
            "\"2016-11-08T16:07:09+00:00\"",
        ] {
            let ts: Timestamp = serde_json::from_str(input).unwrap();
            assert_eq!(ts.seconds, 1478621229, "{input}");
            assert_eq!(ts.nanos, 0, "{input}");
        }
    }

    #[test]
    fn roundtrips_with_fractional_seconds() {
        let ts = Timestamp {
            seconds: 1478621229,
            nanos: 123_456_789,
        };
        let encoded = serde_json::to_string(&ts).unwrap();
        let decoded: Timestamp = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, ts);
    }

    #[test]
    fn rejects_negative_nanos_on_serialize() {
        let ts = Timestamp {
            seconds: 0,
            nanos: -1,
        };
        assert!(serde_json::to_string(&ts).is_err());
    }

    #[test]
    fn rejects_malformed_strings() {
        assert!(serde_json::from_str::<Timestamp>("\"not a date\"").is_err());
        assert!(serde_json::from_str::<Timestamp>("12345").is_err());
    }
}
