use crate::{GqlInputValueResult, GqlResult, GqlValue, ScalarType};
use async_graphql_derive::GqlScalar;
use serde::de::DeserializeOwned;

/// Any scalar
///
/// The `Any` scalar is used to pass representations of entities from external services into the root `_entities` field for execution.
#[derive(Clone, PartialEq, Debug)]
pub struct Any(pub GqlValue);

#[GqlScalar(internal)]
impl ScalarType for Any {
    fn type_name() -> &'static str {
        "_Any"
    }

    fn description() -> Option<&'static str> {
        Some("The `_Any` scalar is used to pass representations of entities from external services into the root `_entities` field for execution.")
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        Ok(Self(value))
    }

    fn is_valid(_value: &GqlValue) -> bool {
        true
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
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
    T: Into<GqlValue>,
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
        let value = GqlValue::List(vec![
            GqlValue::Int(1.into()),
            GqlValue::Float(2.0),
            GqlValue::Null,
        ]);
        let expected = Any(value.clone());
        let output: Any = value.into();
        assert_eq!(output, expected);
    }
}
