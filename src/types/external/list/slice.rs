use std::{borrow::Cow, sync::Arc};

use crate::{
    ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType,
    OutputTypeMarker, Positioned, ServerResult, Value, parser::types::Field, registry,
    resolver_utils::resolve_list,
};

impl<'a, T: OutputTypeMarker + 'a> OutputTypeMarker for &'a [T] {
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!(
            "[{}]",
            <T as OutputTypeMarker>::qualified_type_name()
        ))
    }

    fn qualified_type_name() -> String {
        format!("[{}]!", <T as OutputTypeMarker>::qualified_type_name())
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <T as OutputTypeMarker>::create_type_info(registry);
        Self::qualified_type_name()
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<'a, T: OutputType + OutputTypeMarker + 'a> OutputType for &'a [T] {
    #[cfg(feature = "boxed-trait")]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list::<T>(
            ctx,
            field,
            self.iter().map(|item| item as &dyn OutputType),
            Some(self.len()),
        )
        .await
    }
    #[cfg(not(feature = "boxed-trait"))]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(
            ctx,
            field,
            self.iter(),
            Some(self.len()),
        )
        .await
    }
}

macro_rules! impl_output_slice_for_smart_ptr {
    ($ty:ty) => {
        
        impl<T: OutputTypeMarker> OutputTypeMarker for $ty {
            fn type_name() -> Cow<'static, str> {
                Cow::Owned(format!(
                    "[{}]",
                    <T as OutputTypeMarker>::qualified_type_name()
                ))
            }

            fn qualified_type_name() -> String {
                format!("[{}]!", <T as OutputTypeMarker>::qualified_type_name())
            }

            fn create_type_info(registry: &mut registry::Registry) -> String {
                <T as OutputTypeMarker>::create_type_info(registry);
                Self::qualified_type_name()
            }
        }
        
        #[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
        impl<T: OutputType + OutputTypeMarker> OutputType for $ty {
            #[cfg(feature = "boxed-trait")]
            async fn resolve(
                &self,
                ctx: &ContextSelectionSet<'_>,
                field: &Positioned<Field>,
            ) -> ServerResult<Value> {
                resolve_list::<T>(
                    ctx,
                    field,
                    self.iter().map(|item| item as &dyn OutputType),
                    Some(self.len()),
                )
                .await
            }
            #[cfg(not(feature = "boxed-trait"))]
            async fn resolve(
                &self,
                ctx: &ContextSelectionSet<'_>,
                field: &Positioned<Field>,
            ) -> ServerResult<Value> {
                resolve_list(
                    ctx,
                    field,
                    self.iter(),
                    Some(self.len()),
                )
                .await
            }
        }
    };
}

impl_output_slice_for_smart_ptr!(Box<[T]>);
impl_output_slice_for_smart_ptr!(Arc<[T]>);

macro_rules! impl_input_slice_for_smart_ptr {
    ($ty:ty) => {
        impl<T: InputType> InputType for $ty {
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
                match value.unwrap_or_default() {
                    Value::List(values) => values
                        .into_iter()
                        .map(|value| InputType::parse(Some(value)))
                        .collect::<Result<_, _>>()
                        .map_err(InputValueError::propagate),
                    value => {
                        Ok(
                            vec![
                                InputType::parse(Some(value))
                                    .map_err(InputValueError::propagate)?,
                            ]
                            .into(),
                        )
                    }
                }
            }

            fn to_value(&self) -> Value {
                Value::List(self.iter().map(InputType::to_value).collect())
            }

            fn as_raw_value(&self) -> Option<&Self::RawValueType> {
                Some(self)
            }
        }
    };
}

impl_input_slice_for_smart_ptr!(Box<[T]>);
impl_input_slice_for_smart_ptr!(Arc<[T]>);
