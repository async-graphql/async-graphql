use std::borrow::Cow;

use crate::{
    ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType, OutputTypeMarker, Positioned, Scalar, ScalarType, ServerResult, Value, parser::types::Field, registry::{self, Registry}
};

/// The `String` scalar type represents textual data, represented as UTF-8
/// character sequences. The String type is most often used by GraphQL to
/// represent free-form human-readable text.
#[Scalar(internal)]
impl ScalarType for String {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn is_valid(value: &Value) -> bool {
        matches!(value, Value::String(_))
    }

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }
}

macro_rules! impl_input_string_for_smart_ptr {
    ($ty:ty) => {
        impl InputType for $ty {
            type RawValueType = Self;

            fn type_name() -> Cow<'static, str> {
                Cow::Borrowed("String")
            }

            fn create_type_info(registry: &mut Registry) -> String {
                <String as OutputTypeMarker>::create_type_info(registry)
            }

            fn parse(value: Option<Value>) -> InputValueResult<Self> {
                let value = value.unwrap_or_default();
                match value {
                    Value::String(s) => Ok(s.into()),
                    _ => Err(InputValueError::expected_type(value)),
                }
            }

            fn to_value(&self) -> Value {
                Value::String(self.to_string())
            }

            fn as_raw_value(&self) -> Option<&Self::RawValueType> {
                Some(self)
            }
        }
    };
}

impl_input_string_for_smart_ptr!(Box<str>);
impl_input_string_for_smart_ptr!(std::sync::Arc<str>);

impl OutputTypeMarker for str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
       <String as OutputTypeMarker>::create_type_info(registry)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl OutputType for str {


    async fn resolve(
        &self,
        _: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(Value::String(self.to_string()))
    }
}
