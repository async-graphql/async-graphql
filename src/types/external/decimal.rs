use std::str::FromStr;

use rust_decimal::Decimal;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[Scalar(internal, name = "Decimal")]
impl ScalarType for Decimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(Decimal::from_str(s)?),
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    return Decimal::try_from(f).map_err(InputValueError::custom);
                }

                if let Some(f) = n.as_i64() {
                    return Ok(Decimal::from(f));
                }

                // unwrap safe here, because we have check the other possibility
                Ok(Decimal::from(n.as_u64().unwrap()))
            }
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
