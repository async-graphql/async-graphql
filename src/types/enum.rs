use crate::{InputValueError, InputValueResult, Type, Value};

#[allow(missing_docs)]
pub struct EnumItem<T> {
    pub name: &'static str,
    pub value: T,
}

#[allow(missing_docs)]
#[async_trait::async_trait]
pub trait EnumType: Type + Sized + Eq + Send + Copy + Sized + 'static {
    fn items() -> &'static [EnumItem<Self>];

    fn parse_enum(value: Value) -> InputValueResult<Self> {
        let value = match &value {
            Value::Enum(s) => s,
            Value::String(s) => s.as_str(),
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

    fn to_value(&self) -> Value {
        let items = Self::items();
        for item in items {
            if item.value == *self {
                return Value::Enum(item.name.into());
            }
        }
        unreachable!()
    }
}
