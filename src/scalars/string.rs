use crate::{
    impl_scalar_internal, registry, ContextSelectionSet, JsonWriter, OutputValueType, Result,
    Scalar, Type, Value,
};
use std::borrow::Cow;

const STRING_DESC: &str = "The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.";

impl Scalar for String {
    fn type_name() -> &'static str {
        "String"
    }

    fn description() -> Option<&'static str> {
        Some(STRING_DESC)
    }

    fn parse(value: &Value) -> Option<Self> {
        match value {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn is_valid(value: &Value) -> bool {
        match value {
            Value::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self, w: &mut JsonWriter) -> Result<()> {
        w.string(self.as_str());
        Ok(())
    }
}

impl_scalar_internal!(String);

impl<'a> Type for &'a str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Scalar {
            name: Self::type_name().to_string(),
            description: Some(STRING_DESC),
            is_valid: |value| match value {
                Value::String(_) => true,
                _ => false,
            },
        })
    }
}

#[async_trait::async_trait]
impl<'a> OutputValueType for &'a str {
    async fn resolve(value: &Self, _: &ContextSelectionSet<'_>, w: &mut JsonWriter) -> Result<()> {
        w.string(*value);
        Ok(())
    }
}
