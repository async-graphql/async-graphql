use std::borrow::Cow;

use crate::parser::types::Field;
use crate::{
    registry, ContextSelectionSet, InputValueError, InputValueResult, OutputType, Positioned,
    Scalar, ScalarType, ServerResult, Value,
};

/// The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.
#[Scalar(internal)]
impl ScalarType for String {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

#[async_trait::async_trait]
impl OutputType for str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <String as OutputType>::create_type_info(registry)
    }

    async fn resolve(
        &self,
        _: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(Value::String(self.to_string()))
    }
}
