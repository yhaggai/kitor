//! UTC timestamps shared by database models and the frontend.
use time::{format_description::well_known::Rfc3339, OffsetDateTime, PrimitiveDateTime};

pub fn naive_utc(value: OffsetDateTime) -> PrimitiveDateTime {
    let utc = value.to_offset(time::UtcOffset::UTC);
    PrimitiveDateTime::new(utc.date(), utc.time())
}

pub fn now_naive() -> PrimitiveDateTime {
    naive_utc(OffsetDateTime::now_utc())
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("UTC timestamp is valid RFC3339")
}

// Keep the existing Chrono JSON representation (an ISO timestamp without an offset).
pub mod serde_naive {
    use super::*;
    use serde::{Deserialize, Deserializer, Serializer};

    fn format(value: &PrimitiveDateTime) -> String {
        let mut text = value
            .format(time::macros::format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]"
            ))
            .expect("valid timestamp");
        if value.nanosecond() != 0 {
            text.push('.');
            let nanos = value.nanosecond();
            if nanos % 1_000_000 == 0 {
                text.push_str(&format!("{:03}", nanos / 1_000_000));
            } else if nanos % 1_000 == 0 {
                text.push_str(&format!("{:06}", nanos / 1_000));
            } else {
                text.push_str(&format!("{nanos:09}"));
            }
        }
        text
    }

    fn parse(value: &str) -> Result<PrimitiveDateTime, time::error::Parse> {
        PrimitiveDateTime::parse(
            value,
            time::macros::format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]"
            ),
        )
        .or_else(|_| {
            PrimitiveDateTime::parse(
                value,
                time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]"),
            )
        })
    }

    pub fn serialize<S: Serializer>(
        value: &PrimitiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format(value))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<PrimitiveDateTime, D::Error> {
        parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }

    pub mod option {
        use super::*;
        pub fn serialize<S: Serializer>(
            value: &Option<PrimitiveDateTime>,
            serializer: S,
        ) -> Result<S::Ok, S::Error> {
            match value {
                Some(value) => serializer.serialize_some(&format(value)),
                None => serializer.serialize_none(),
            }
        }
        pub fn deserialize<'de, D: Deserializer<'de>>(
            deserializer: D,
        ) -> Result<Option<PrimitiveDateTime>, D::Error> {
            Option::<String>::deserialize(deserializer)?
                .map(|s| parse(&s).map_err(serde::de::Error::custom))
                .transpose()
        }
    }
}

#[cfg(test)]
#[path = "date_time_tests.rs"]
mod tests;
