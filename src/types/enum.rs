use crate::{GQLType, Result};
use graphql_parser::query::Value;

#[doc(hidden)]
pub struct GQLEnumItem<T> {
    pub name: &'static str,
    pub value: T,
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait GQLEnum: GQLType + Sized + Eq + Send + Copy + Sized + 'static {
    fn items() -> &'static [GQLEnumItem<Self>];

    fn parse_enum(value: &Value) -> Option<Self> {
        match value {
            Value::Enum(s) => {
                let items = Self::items();
                for item in items {
                    if item.name == s {
                        return Some(item.value);
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn parse_json_enum(value: &serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(s) => {
                let items = Self::items();
                for item in items {
                    if item.name == s {
                        return Some(item.value);
                    }
                }
            }
            _ => {}
        }
        None
    }

    fn resolve_enum(&self) -> Result<serde_json::Value> {
        let items = Self::items();
        for item in items {
            if item.value == *self {
                return Ok(item.name.clone().into());
            }
        }
        unreachable!()
    }
}
