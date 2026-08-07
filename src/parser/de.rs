//! Вспомогательные десериализаторы: даты модели и прямая десериализация
//! секций из пар "ключ-значение" без промежуточного `serde_json::Value`.

use chrono::NaiveDate;
use serde::de::value::{Error as AttrError, MapDeserializer};
use serde::de::{IntoDeserializer, Visitor};
use serde::{Deserialize, Deserializer};

const DATE_FORMAT: &str = "%d.%m.%Y";

/// Десериализует структуру напрямую из владеющих пар атрибутов секции.
pub fn from_owned_attrs<'de, T>(
    attrs: impl Iterator<Item = (String, String)>,
) -> Result<T, AttrError>
where
    T: Deserialize<'de>,
{
    T::deserialize(MapDeserializer::new(attrs.map(|(k, v)| (k, OwnedAttr(v)))))
}

/// Десериализует структуру из заимствованных пар атрибутов секции.
pub fn from_borrowed_attrs<'de, T>(
    attrs: impl Iterator<Item = (&'de str, &'de str)>,
) -> Result<T, AttrError>
where
    T: Deserialize<'de>,
{
    T::deserialize(MapDeserializer::new(
        attrs.map(|(k, v)| (k, BorrowedAttr(v))),
    ))
}

/// Парсинг опциональных дат формата "%d.%m.%Y".
pub fn try_deserialize_dt<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<std::borrow::Cow<'de, str>>::deserialize(deserializer)? {
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
    match Option::<std::borrow::Cow<'de, str>>::deserialize(deserializer)? {
        Some(s) if s.trim().is_empty() => {
            Err(serde::de::Error::custom("empty string is not a valid date"))
        }
        Some(s) => NaiveDate::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom),
        None => Err(serde::de::Error::custom("missing date field")),
    }
}

/// Владеющее значение атрибута: строковые поля модели забирают `String`
/// без копирования.
struct OwnedAttr(String);

impl<'de> Deserializer<'de> for OwnedAttr {
    type Error = AttrError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.0)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> IntoDeserializer<'de, AttrError> for OwnedAttr {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

/// Заимствованное значение атрибута — для секций, чьи поля не содержат строк.
struct BorrowedAttr<'de>(&'de str);

impl<'de> Deserializer<'de> for BorrowedAttr<'de> {
    type Error = AttrError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple tuple_struct
        map struct enum identifier ignored_any
    }
}

impl<'de> IntoDeserializer<'de, AttrError> for BorrowedAttr<'de> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}
