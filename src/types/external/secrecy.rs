use std::borrow::Cow;

use secrecy::{Secret, Zeroize};

use crate::{registry, InputType, InputValueError, InputValueResult, Value};

impl<T: InputType + Zeroize> InputType for Secret<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::qualified_type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry)
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        T::parse(value)
            .map(Secret::new)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> Value {
        Value::Null
    }
}
