use std::{
    borrow::Cow, collections::HashMap as StdHashMap, fmt::Display, hash::Hash, str::FromStr,
};

use async_graphql_parser::{Positioned, types::Field};
use async_graphql_value::{from_value, to_value};
use hashbrown::HashMap;
use indexmap::IndexMap;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    ContextSelectionSet, InputType, InputValueError, InputValueResult, Name, OutputType, OutputTypeMarker, ServerResult, Value, registry::Registry
};

impl<K, V> InputType for HashMap<K, V>
where
    K: ToString + FromStr + Eq + Hash + Send + Sync,
    K::Err: Display,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        <StdHashMap<K, V> as InputType>::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        <StdHashMap<K, V> as InputType>::create_type_info(registry)
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        let value = value.unwrap_or_default();
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(name, value)| {
                    Ok((
                        K::from_str(&name).map_err(|err| {
                            InputValueError::<Self>::custom(format!("object key: {}", err))
                        })?,
                        from_value(value).map_err(|err| format!("object value: {}", err))?,
                    ))
                })
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        let mut map = IndexMap::new();
        for (name, value) in self {
            map.insert(
                Name::new(name.to_string()),
                to_value(value).unwrap_or_default(),
            );
        }
        Value::Object(map)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<K, V> OutputType for HashMap<K, V>
where
    K: ToString + Eq + Hash + Send + Sync,
    V: Serialize + Send + Sync,
{

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        let mut map = IndexMap::new();
        for (name, value) in self {
            map.insert(
                Name::new(name.to_string()),
                to_value(value).unwrap_or_default(),
            );
        }
        Ok(Value::Object(map))
    }
}
