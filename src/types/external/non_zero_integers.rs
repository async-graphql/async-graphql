use std::num::{
    NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroU8, NonZeroUsize,
};

use crate::{InputValueError, InputValueResult, Number, Scalar, ScalarType, Value};

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroI8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_i64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i8::MIN as i64 || n > i8::MAX as i64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i8::MIN,
                        i8::MAX
                    )));
                }
                Ok(NonZeroI8::new(n as i8).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as i64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroI16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_i64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i16::MIN as i64 || n > i16::MAX as i64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i16::MIN,
                        i16::MAX
                    )));
                }
                Ok(NonZeroI16::new(n as i16).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as i64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroI32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_i64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < i32::MIN as i64 || n > i32::MAX as i64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        i32::MIN,
                        i32::MAX
                    )));
                }
                Ok(NonZeroI32::new(n as i32).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as i64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroI64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_i64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n == 0 {
                    return Err(InputValueError::from("Only non zero are accepted."));
                }
                Ok(NonZeroI64::new(n).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get()))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroIsize {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_i64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n < isize::MIN as i64 || n > isize::MAX as i64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        isize::MIN,
                        isize::MAX
                    )));
                }
                Ok(NonZeroIsize::new(n as isize).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as i64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroU8 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u8::MAX as u64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u8::MAX
                    )));
                }
                Ok(NonZeroU8::new(n as u8).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as u64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroU16 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u16::MAX as u64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u16::MAX
                    )));
                }
                Ok(NonZeroU16::new(n as u16).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as u64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroU32 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > u32::MAX as u64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        u32::MAX
                    )));
                }
                Ok(NonZeroU32::new(n as u32).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as u64))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroU64 {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n == 0 {
                    return Err(InputValueError::from("Only non zero are accepted."));
                }
                Ok(NonZeroU64::new(n).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get()))
    }
}

/// The `Int` scalar type represents non-fractional whole numeric values.
#[Scalar(internal, name = "Int")]
impl ScalarType for NonZeroUsize {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                let n = n
                    .as_u64()
                    .ok_or_else(|| InputValueError::from("Invalid number"))?;
                if n > usize::MAX as u64 || n == 0 {
                    return Err(InputValueError::from(format!(
                        "Only integers from {} to {} or non zero are accepted.",
                        1,
                        usize::MAX
                    )));
                }
                Ok(NonZeroUsize::new(n as usize).unwrap())
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::Number(n) if n.is_i64())
    }

    fn to_value(&self) -> Value {
        Value::Number(Number::from(self.get() as u64))
    }
}
