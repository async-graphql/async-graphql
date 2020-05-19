use crate::{InputValueResult, Result, ScalarType, Value};
use async_graphql_derive::Scalar;
use serde::de::DeserializeOwned;

/// Any scalar
///
/// The `Any` scalar is used to pass representations of entities from external services into the root `_entities` field for execution.
#[derive(Clone, PartialEq, Debug)]
pub struct Any(pub Value);

/// The `_Any` scalar is used to pass representations of entities from external services into the root `_entities` field for execution.
#[Scalar(internal, name = "_Any")]
impl ScalarType for Any {
    fn parse(value: Value) -> InputValueResult<Self> {
        Ok(Self(value))
    }

    fn is_valid(_value: &Value) -> bool {
        true
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.0.clone().into())
    }
}

impl Any {
    /// Parse this `Any` value to T by `serde_json`.
    pub fn parse_value<T: DeserializeOwned>(&self) -> std::result::Result<T, serde_json::Error> {
        serde_json::from_value(self.to_json().unwrap())
    }
}

impl<T> From<T> for Any
where
    T: Into<Value>,
{
    fn from(value: T) -> Any {
        Any(value.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_conversion_ok() {
        let value = Value::List(vec![Value::Int(1.into()), Value::Float(2.0), Value::Null]);
        let expected = Any(value.clone());
        let output: Any = value.into();
        assert_eq!(output, expected);
    }
}
