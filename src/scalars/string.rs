use crate::{ContextSelectionSet, GQLOutputValue, GQLType, QueryError, Result, Scalar, Value};
use std::borrow::Cow;

impl Scalar for String {
    fn type_name() -> &'static str {
        "String!"
    }

    fn parse(value: Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s),
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
            serde_json::Value::String(s) => Ok(s),
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
        Ok(self.clone().into())
    }
}

impl<'a> GQLType for &'a str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String!")
    }
}

#[async_trait::async_trait]
impl<'a> GQLOutputValue for &'a str {
    async fn resolve(&self, _: &ContextSelectionSet<'_>) -> Result<serde_json::Value> {
        Ok(self.to_string().into())
    }
}
