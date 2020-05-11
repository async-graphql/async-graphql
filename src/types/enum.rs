use crate::{GqlInputValueResult, GqlResult, GqlValue, InputValueError, Type};

#[allow(missing_docs)]
pub struct EnumItem<T> {
    pub name: &'static str,
    pub value: T,
}

#[allow(missing_docs)]
#[async_trait::async_trait]
pub trait EnumType: Type + Sized + Eq + Send + Copy + Sized + 'static {
    fn items() -> &'static [EnumItem<Self>];

    fn parse_enum(value: GqlValue) -> GqlInputValueResult<Self> {
        let value = match value {
            GqlValue::Enum(s) => s,
            GqlValue::String(s) => s,
            _ => return Err(InputValueError::ExpectedType(value)),
        };

        let items = Self::items();
        for item in items {
            if item.name == value {
                return Ok(item.value);
            }
        }
        Err(InputValueError::Custom(format!(
            r#"Enumeration type "{}" does not contain the value "{}""#,
            Self::type_name(),
            value
        )))
    }

    fn resolve_enum(&self) -> GqlResult<serde_json::Value> {
        let items = Self::items();
        for item in items {
            if item.value == *self {
                return Ok(item.name.into());
            }
        }
        unreachable!()
    }
}
