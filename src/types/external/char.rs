use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// The `Char` scalar type represents a unicode char.
/// The input and output values are a string, and there can only be one unicode
/// character in this string.
#[Scalar(internal)]
impl ScalarType for char {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => {
                let mut chars = s.chars();
                match chars.next() {
                    Some(ch) if chars.next().is_none() => Ok(ch),
                    Some(_) => Err(InputValueError::custom(
                        "There can only be one unicode character in the string.",
                    )),
                    None => Err(InputValueError::custom("A unicode character is required.")),
                }
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String((*self).into())
    }
}
