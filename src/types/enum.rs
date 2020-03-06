use crate::{GQLType, Result};
use graphql_parser::query::Value;

pub struct GQLEnumItem<T> {
    pub name: &'static str,
    pub value: T,
}

#[async_trait::async_trait]
pub trait GQLEnum: GQLType + Sized + Eq + Send + Copy + Sized + 'static {
    fn items() -> &'static [GQLEnumItem<Self>];

    fn parse_enum(value: &Value) -> Option<Self> {
        let value = match value {
            Value::Enum(s) => Some(s.as_str()),
            Value::String(s) => Some(s.as_str()),
            _ => None,
        };

        value.and_then(|value| {
            let items = Self::items();
            for item in items {
                if item.name == value {
                    return Some(item.value);
                }
            }
            None
        })
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
