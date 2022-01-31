use std::borrow::Cow;
use std::sync::Arc;

use crate::parser::types::Field;
use crate::resolver_utils::resolve_list;
use crate::{
    registry, ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType,
    Positioned, ServerResult, Value,
};

impl<T: InputType, const N: usize> InputType for [T; N] {
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        if let Some(Value::List(values)) = value {
            let items: Vec<T> = values
                .into_iter()
                .map(|value| InputType::parse(Some(value)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate)?;
            let len = items.len();
            items.try_into().map_err(|_| {
                InputValueError::custom(format!(
                    "Expected input type \"[{}; {}]\", found [{}; {}].",
                    T::type_name(),
                    N,
                    T::type_name(),
                    len
                ))
            })
        } else {
            Err(InputValueError::expected_type(value.unwrap_or_default()))
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputType::to_value).collect())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

#[async_trait::async_trait]
impl<T: OutputType, const N: usize> OutputType for [T; N] {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("[{}]", T::qualified_type_name()))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", T::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        Self::qualified_type_name()
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self.iter(), Some(self.len())).await
    }
}

#[async_trait::async_trait]
impl<T: InputType> InputType for Box<[T]> {
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        <Vec<T> as InputType>::type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <Vec<T> as InputType>::create_type_info(registry)
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => values
                .into_iter()
                .map(|value| InputType::parse(Some(value)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            value => Ok(Box::<[T]>::from([
                InputType::parse(Some(value)).map_err(InputValueError::propagate)?
            ])),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputType::to_value).collect())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

macro_rules! impl_output_arr_for_smart_ptr {
    ($name:ident) => {
        #[async_trait::async_trait]
        impl<T: OutputType> OutputType for $name<[T]> {
            fn type_name() -> Cow<'static, str> {
                <Vec<T> as OutputType>::type_name()
            }

            fn qualified_type_name() -> String {
                <Vec<T> as OutputType>::qualified_type_name()
            }

            fn create_type_info(registry: &mut registry::Registry) -> String {
                <Vec<T> as OutputType>::create_type_info(registry)
            }

            async fn resolve(
                &self,
                ctx: &ContextSelectionSet<'_>,
                field: &Positioned<Field>,
            ) -> ServerResult<Value> {
                resolve_list(ctx, field, self.iter(), Some(self.len())).await
            }
        }
    };
}

impl_output_arr_for_smart_ptr!(Box);
impl_output_arr_for_smart_ptr!(Arc);
