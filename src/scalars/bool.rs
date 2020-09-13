use crate::{GQLScalar, InputValueError, InputValueResult, ScalarType, Value};

/// The `Boolean` scalar type represents `true` or `false`.
#[GQLScalar(internal, name = "Boolean")]
impl ScalarType for bool {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::Boolean(_) => true,
            _ => false,
        }
    }

    fn to_value(&self) -> Value {
        Value::Boolean(*self)
    }
}
