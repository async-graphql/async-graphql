use crate::{
    registry, ContextSelectionSet, InputValueError, InputValueResult, OutputValueType, Positioned,
    Result, ScalarType, Type, Value,
};
use async_graphql_derive::Scalar;
use async_graphql_parser::query::Field;
use std::borrow::Cow;

/// The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.
#[Scalar(internal)]
impl ScalarType for String {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok(self.clone().into())
    }
}

impl<'a> Type for &'a str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <String as Type>::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<'a> OutputValueType for &'a str {
    async fn resolve(
        &self,
        _: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        Ok((*self).into())
    }
}
