use std::net::IpAddr;

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// Implement the IP scalar
///
/// The input/output is in the respective IP format
#[Scalar(
    internal,
    name = "IpAddr",
    specified_by_url = "https://en.wikipedia.org/wiki/IP_address"
)]
impl ScalarType for IpAddr {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}
