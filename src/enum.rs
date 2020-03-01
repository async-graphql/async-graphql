use crate::{GQLQueryError, GQLType, Result};
use graphql_parser::query::Value;

#[doc(hidden)]
pub struct GQLEnumItem<T> {
    pub name: &'static str,
    pub desc: Option<&'static str>,
    pub value: T,
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait GQLEnum: GQLType + Sized + Eq + Send + Copy + Sized + 'static {
    fn items() -> &'static [GQLEnumItem<Self>];

    fn parse_enum(value: Value) -> Result<Self> {
        match value {
            Value::Enum(s) => {
                let items = Self::items();
                for item in items {
                    if item.name == s {
                        return Ok(item.value);
                    }
                }
                Err(GQLQueryError::InvalidEnumValue {
                    enum_type: Self::type_name(),
                    value: s,
                }
                .into())
            }
            _ => {
                return Err(GQLQueryError::ExpectedType {
                    expect: Self::type_name(),
                    actual: value,
                }
                .into())
            }
        }
    }

    fn resolve_enum(self) -> Result<serde_json::Value> {
        let items = Self::items();
        for item in items {
            if item.value == self {
                return Ok(item.name.clone().into());
            }
        }
        unreachable!()
    }
}
