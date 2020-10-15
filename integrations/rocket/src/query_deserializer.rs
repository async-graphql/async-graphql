use rocket::http::RawStr;
use rocket::request::Query;
use serde::de::{DeserializeSeed, Deserializer, Error as _, IntoDeserializer, MapAccess, Visitor};
use serde::forward_to_deserialize_any;

/// A wrapper around `rocket::request::Query` that implements `Deserializer`.
pub(crate) struct QueryDeserializer<'q>(pub(crate) Query<'q>);

impl<'q, 'de> Deserializer<'de> for QueryDeserializer<'q> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_map(QueryMapAccess {
            query: self.0,
            value: None,
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct QueryMapAccess<'q> {
    query: Query<'q>,
    value: Option<&'q RawStr>,
}

impl<'q, 'de> MapAccess<'de> for QueryMapAccess<'q> {
    type Error = serde::de::value::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        self.query
            .next()
            .map(|item| {
                self.value = Some(item.value);
                seed.deserialize(
                    item.key
                        .url_decode()
                        .map_err(Self::Error::custom)?
                        .into_deserializer(),
                )
            })
            .transpose()
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(
            self.value
                .take()
                .unwrap()
                .url_decode()
                .map_err(Self::Error::custom)?
                .into_deserializer(),
        )
    }
}
