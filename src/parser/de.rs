//! Вспомогательные десериализаторы для полей модели.

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

const DATE_FORMAT: &str = "%d.%m.%Y";

/// Парсинг опциональных дат формата "%d.%m.%Y".
pub fn try_deserialize_dt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => NaiveDate::parse_from_str(&s, DATE_FORMAT)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Парсинг обязательных дат формата "%d.%m.%Y".
pub fn deserialize_dt<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() => {
            Err(serde::de::Error::custom("empty string is not a valid date"))
        }
        Some(s) => NaiveDate::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom),
        None => Err(serde::de::Error::custom("missing date field")),
    }
}
