use crate::schema::{__EnumValue, __InputValue, __TypeKind};
use crate::{ContextField, Result};
use async_graphql_derive::Object;

#[Object(
    internal,
    desc = r#"
The fundamental unit of any GraphQL Schema is the type. There are many kinds of types in GraphQL as represented by the `__TypeKind` enum.

Depending on the kind of a type, certain fields describe information about that type. Scalar types provide no information beyond a name and description, while Enum types provide their values. Object and Interface types provide the fields they describe. Abstract types, Union and Interface, provide the Object types possible at runtime. List and NonNull types compose other types.
"#,
    field(name = "kind", type = "__TypeKind", owned),
    field(name = "name", type = "String", owned),
    field(name = "description", type = "Option<String>", owned),
    field(
        name = "fields",
        type = "Option<Vec<__Type>>",
        owned,
        arg(name = "includeDeprecated", type = "bool")
    ),
    field(name = "interfaces", type = "Option<Vec<__Type>>", owned),
    field(name = "possibleTypes", type = "Option<Vec<__Type>>", owned),
    field(name = "enumValues", type = "Option<Vec<__EnumValue>>", owned),
    field(name = "inputFields", type = "Option<Vec<__InputValue>>", owned),
    field(name = "ofType", type = "Option<__Type>", owned)
)]
pub struct __Type {}

#[async_trait::async_trait]
impl __TypeFields for __Type {
    async fn kind<'a>(&'a self, _: &ContextField<'_>) -> Result<__TypeKind> {
        todo!()
    }

    async fn name<'a>(&'a self, _: &ContextField<'_>) -> Result<String> {
        todo!()
    }

    async fn description<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<String>> {
        todo!()
    }

    async fn fields<'a>(
        &'a self,
        _: &ContextField<'_>,
        include_deprecated: bool,
    ) -> Result<Option<Vec<__Type>>> {
        todo!()
    }

    async fn interfaces<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<Vec<__Type>>> {
        todo!()
    }

    async fn possible_types<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<Vec<__Type>>> {
        todo!()
    }

    async fn enum_values<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<Vec<__EnumValue>>> {
        todo!()
    }

    async fn input_fields<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<Vec<__InputValue>>> {
        todo!()
    }

    async fn of_type<'a>(&'a self, _: &ContextField<'_>) -> Result<Option<__Type>> {
        todo!()
    }
}
