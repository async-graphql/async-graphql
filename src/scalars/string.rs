use crate::parser::Pos;
use crate::{
    registry, GqlContextSelectionSet, GqlInputValueResult, GqlResult, GqlValue, InputValueError,
    OutputValueType, ScalarType, Type,
};
use async_graphql_derive::GqlScalar;
use std::borrow::Cow;

const STRING_DESC: &str = "The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.";

#[GqlScalar(internal)]
impl ScalarType for String {
    fn type_name() -> &'static str {
        "String"
    }

    fn description() -> Option<&'static str> {
        Some(STRING_DESC)
    }

    fn parse(value: GqlValue) -> GqlInputValueResult<Self> {
        match value {
            GqlValue::String(s) => Ok(s),
            _ => Err(InputValueError::ExpectedType(value)),
        }
    }

    fn is_valid(value: &GqlValue) -> bool {
        match value {
            GqlValue::String(_) => true,
            _ => false,
        }
    }

    fn to_json(&self) -> GqlResult<serde_json::Value> {
        Ok(self.clone().into())
    }
}

impl<'a> Type for &'a str {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        registry.create_type::<Self, _>(|_| registry::Type::Scalar {
            name: Self::type_name().to_string(),
            description: Some(STRING_DESC),
            is_valid: |value| match value {
                GqlValue::String(_) => true,
                _ => false,
            },
        })
    }
}

#[async_trait::async_trait]
impl<'a> OutputValueType for &'a str {
    async fn resolve(
        &self,
        _: &GqlContextSelectionSet<'_>,
        _pos: Pos,
    ) -> GqlResult<serde_json::Value> {
        Ok((*self).into())
    }
}
