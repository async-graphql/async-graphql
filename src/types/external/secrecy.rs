use std::borrow::Cow;

use secrecy::{zeroize::Zeroize, ExposeSecret, SecretBox, SecretString};

use crate::{registry, InputType, InputValueError, InputValueResult, Value};

impl<T: InputType + Zeroize> InputType for SecretBox<T> {
    type RawValueType = T::RawValueType;

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
            .map(|value| SecretBox::new(Box::new(value)))
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> Value {
        Value::Null
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.expose_secret().as_raw_value()
    }
}

impl InputType for SecretString {
    type RawValueType = str;

    fn type_name() -> Cow<'static, str> {
        String::type_name()
    }

    fn qualified_type_name() -> String {
        String::qualified_type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        String::create_type_info(registry)
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        String::parse(value)
            .map(SecretString::from)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> Value {
        Value::Null
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self.expose_secret())
    }
}
