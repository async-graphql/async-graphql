use std::{borrow::Cow, collections::HashSet as StdHashSet, hash::Hash};

use hashbrown::HashSet;

use crate::{
    ContextSelectionSet, InputType, InputValueError, InputValueResult, OutputType, OutputTypeMarker, Positioned, Result, ServerResult, Value, parser::types::Field, registry, resolver_utils::resolve_list
};

impl<T: InputType + Hash + Eq> InputType for HashSet<T> {
    type RawValueType = Self;

    fn type_name() -> Cow<'static, str> {
        <StdHashSet<T> as InputType>::type_name()
    }

    fn qualified_type_name() -> String {
        <StdHashSet<T> as InputType>::qualified_type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <StdHashSet<T> as InputType>::create_type_info(registry)
    }

    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value.unwrap_or_default() {
            Value::List(values) => values
                .into_iter()
                .map(|value| InputType::parse(Some(value)))
                .collect::<Result<_, _>>()
                .map_err(InputValueError::propagate),
            value => Ok({
                let mut result = Self::default();
                result.insert(InputType::parse(Some(value)).map_err(InputValueError::propagate)?);
                result
            }),
        }
    }

    fn to_value(&self) -> Value {
        Value::List(self.iter().map(InputType::to_value).collect())
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }
}

impl<T: OutputTypeMarker + Hash + Eq> OutputTypeMarker for HashSet<T> {
    fn type_name(&self) -> Cow<'static, str> {
        <StdHashSet<T> as OutputTypeMarker>::type_name()
    }

    fn qualified_type_name(&self) -> String {
        <StdHashSet<T> as OutputTypeMarker>::qualified_type_name()
    }

    fn create_type_info(&self, registry: &mut registry::Registry) -> String {
        <StdHashSet<T> as OutputTypeMarker>::create_type_info(registry)
    }
}

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: OutputType + Hash + Eq +  OutputTypeMarker> OutputType for HashSet<T> {    
    #[cfg(feature = "boxed-trait")]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self.iter().map(|item| item as &dyn OutputType), Some(self.len())).await
    }

    #[cfg(not(feature = "boxed-trait"))]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self.iter(), Some(self.len())).await
    }
}
