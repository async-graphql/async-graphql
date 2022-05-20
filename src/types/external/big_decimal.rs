use std::str::FromStr;

use bigdecimal::BigDecimal;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

#[Scalar(internal, name = "BigDecimal")]
impl ScalarType for BigDecimal {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    return BigDecimal::try_from(f).map_err(InputValueError::custom);
                }

                if let Some(f) = n.as_i64() {
                    return Ok(BigDecimal::from(f)); 
                }

                // unwrap safe here, because we have check the other possibility
                Ok(BigDecimal::from(n.as_u64().unwrap()))
            },
            Value::String(s) => Ok(BigDecimal::from_str(s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}