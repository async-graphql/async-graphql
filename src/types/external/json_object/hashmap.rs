use crate::{
    InputValueError, InputValueResult, InputValueType, Name, OutputValueType, Scalar, ScalarType,
    Value,
};
use std::collections::{BTreeMap, HashMap};

/// A scalar that can represent any JSON Object value.
#[Scalar(internal, name = "JSONObject")]
impl<T> ScalarType for HashMap<String, T>
where
    T: OutputValueType + InputValueType + Send + Sync,
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
