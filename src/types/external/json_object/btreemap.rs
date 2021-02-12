use std::collections::BTreeMap;

use crate::{
    InputType, InputValueError, InputValueResult, Name, OutputType, Scalar, ScalarType, Value,
};

/// A scalar that can represent any JSON Object value.
#[Scalar(internal, name = "JSONObject")]
impl<T> ScalarType for BTreeMap<String, T>
where
    T: OutputType + InputType,
{
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(name, value)| Ok((name.to_string(), T::parse(Some(value))?)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        let mut map = BTreeMap::new();
        for (name, value) in self {
            map.insert(Name::new(name), value.to_value());
        }
        Value::Object(map)
    }
}
