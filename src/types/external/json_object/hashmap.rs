use crate::parser::types::Name;
use crate::{
    GQLScalar, InputValueError, InputValueResult, InputValueType, OutputValueType, ScalarType,
    Value,
};
use std::collections::{BTreeMap, HashMap};

/// A scalar that can represent any JSON Object value.
#[GQLScalar(internal, name = "JSONObject")]
impl<T> ScalarType for HashMap<String, T>
where
    T: OutputValueType + InputValueType + Send + Sync,
{
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Object(map) => {
                let mut result = HashMap::new();
                for (name, value) in map {
                    result.insert(name.to_string(), T::parse(Some(value))?);
                }
                Ok(result)
            }
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn to_value(&self) -> Value {
        let mut map = BTreeMap::new();
        for (name, value) in self {
            if let Ok(name) = Name::new(name.clone()) {
                map.insert(name, value.to_value());
            }
        }
        Value::Object(map)
    }
}
