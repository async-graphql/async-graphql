use std::{borrow::Cow, collections::HashSet as StdHashSet, hash::Hash};

use hashbrown::HashSet;

use crate::{
    parser::types::Field, registry, resolver_utils::resolve_list, ContextSelectionSet, InputType,
    InputValueError, InputValueResult, OutputType, Positioned, Result, ServerResult, Value,
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

#[cfg_attr(feature = "boxed-trait", async_trait::async_trait)]
impl<T: OutputType + Hash + Eq> OutputType for HashSet<T> {
    fn type_name() -> Cow<'static, str> {
        <StdHashSet<T> as OutputType>::type_name()
    }

    fn qualified_type_name() -> String {
        <StdHashSet<T> as OutputType>::qualified_type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <StdHashSet<T> as OutputType>::create_type_info(registry)
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        resolve_list(ctx, field, self, Some(self.len())).await
    }
}
