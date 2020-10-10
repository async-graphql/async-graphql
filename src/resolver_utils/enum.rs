use crate::parser::types::Name;
use crate::{InputValueError, InputValueResult, InputValueType, Type, Value};

/// A variant of an enum.
pub struct EnumItem<T> {
    /// The name of the variant.
    pub name: &'static str,
    /// The value of the variant.
    pub value: T,
}

/// A GraphQL enum.
pub trait EnumType: Type + Sized + Eq + Send + Copy + Sized + 'static {
    /// Get a list of possible variants of the enum and their values.
    fn items() -> &'static [EnumItem<Self>];
}

/// Parse a value as an enum value.
///
/// This can be used to implement `InputValueType::parse`.
pub fn parse_enum<T: EnumType + InputValueType>(value: Value) -> InputValueResult<T> {
    let value = match &value {
        Value::Enum(s) => s,
        Value::String(s) => s.as_str(),
        _ => return Err(InputValueError::expected_type(value)),
    };

    T::items()
        .iter()
        .find(|item| item.name == value)
        .map(|item| item.value)
        .ok_or_else(|| {
            InputValueError::custom(format_args!(
                r#"Enumeration type does not contain value "{}"."#,
                value,
            ))
        })
}

/// Convert the enum value into a GraphQL value.
///
/// This can be used to implement `InputValueType::to_value` or `OutputValueType::resolve`.
pub fn enum_value<T: EnumType>(value: T) -> Value {
    let item = T::items().iter().find(|item| item.value == value).unwrap();
    Value::Enum(Name::new(item.name))
}
