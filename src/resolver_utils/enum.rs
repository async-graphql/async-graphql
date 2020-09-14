use crate::parser::types::Name;
use crate::{InputValueError, InputValueResult, Type, Value};

/// A variant of an enum.
pub struct EnumItem<T> {
    /// The name of the variant.
    pub name: &'static str,
    /// The value of the variant.
    pub value: T,
}

/// An enum value.
pub trait EnumType: Type + Sized + Eq + Send + Copy + Sized + 'static {
    /// Get a list of the variants of the enum value.
    fn items() -> &'static [EnumItem<Self>];
}

/// Parse a value as an enum value.
///
/// This can be used to implement `InputValueType::parse`.
pub fn parse_enum<T: EnumType>(value: Value) -> InputValueResult<T> {
    let value = match &value {
        Value::Enum(s) => s,
        Value::String(s) => s.as_str(),
        _ => return Err(InputValueError::ExpectedType(value)),
    };

    T::items()
        .iter()
        .find(|item| item.name == value)
        .map(|item| item.value)
        .ok_or_else(|| {
            InputValueError::Custom(format!(
                r#"Enumeration type "{}" does not contain the value "{}""#,
                T::type_name(),
                value,
            ))
        })
}

/// Convert the enum value into a GraphQL value.
///
/// This can be used to implement `InputValueType::to_value` or `OutputValueType::resolve`.
pub fn enum_value<T: EnumType>(value: T) -> Value {
    let item = T::items().iter().find(|item| item.value == value).unwrap();
    Value::Enum(Name::new_unchecked(item.name.to_owned()))
}
