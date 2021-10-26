use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use indexmap::IndexMap;

use crate::{
    InputType, InputValueError, InputValueResult, Name, OutputType, Scalar, ScalarType, Value,
};

/// A scalar that can represent any JSON Object value.
#[Scalar(internal, name = "JSONObject")]
impl<K, T> ScalarType for BTreeMap<K, T>
where
    K: ToString + FromStr + Ord + Sync + Send,
    K::Err: Display,
    T: OutputType + InputType,
{
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(name, value)| {
                    Ok((
                        K::from_str(&name).map_err(|err| {
                            InputValueError::custom(format!("object key: {}", err))
                        })?,
                        T::parse(Some(value))?,
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
            map.insert(Name::new(name.to_string()), value.to_value());
        }
        Value::Object(map)
    }
}
