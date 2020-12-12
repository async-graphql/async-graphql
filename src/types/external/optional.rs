use std::borrow::Cow;

use crate::parser::types::Field;
use crate::{
    registry, ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType,
    Positioned, ServerResult, Type, Value,
};

impl<T: Type> Type for Option<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        T::type_name().to_string()
    }
}

impl<T: InputType> InputType for Option<T> {
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::Null => Ok(None),
            value => Ok(Some(
                T::parse(Some(value)).map_err(InputValueError::propagate)?,
            )),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Some(value) => value.to_value(),
            None => Value::Null,
        }
    }
}

#[async_trait::async_trait]
impl<T: OutputType + Sync> OutputType for Option<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        if let Some(inner) = self {
            OutputType::resolve(inner, ctx, field).await
        } else {
            Ok(Value::Null)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Type;

    #[test]
    fn test_optional_type() {
        assert_eq!(Option::<i32>::type_name(), "Int");
        assert_eq!(Option::<i32>::qualified_type_name(), "Int");
        assert_eq!(&Option::<i32>::type_name(), "Int");
        assert_eq!(&Option::<i32>::qualified_type_name(), "Int");
    }
}
