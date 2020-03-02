use crate::{GQLType, QueryError, Result, Scalar, Value};

impl Scalar for bool {
    fn type_name() -> &'static str {
        "Boolean!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::Boolean(n) => Ok(n),
            _ => {
                return Err(QueryError::ExpectedType {
                    expect: <Self as GQLType>::type_name(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn parse_from_json(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Bool(n) => Ok(n),
            _ => {
                return Err(QueryError::ExpectedJsonType {
                    expect: <Self as GQLType>::type_name(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn to_json(&self) -> Result<serde_json::Value> {
        Ok((*self).into())
    }
}
