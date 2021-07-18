use std::borrow::Cow;

use bytes::Bytes;

use crate::parser::types::Field;
use crate::parser::Positioned;
use crate::{registry, ContextSelectionSet, OutputType, ServerResult, Type, Value};

impl Type for Bytes {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("Binary")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        <String as Type>::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl OutputType for Bytes {
    async fn resolve(
        &self,
        _: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        Ok(Value::Binary(self.clone()))
    }
}
