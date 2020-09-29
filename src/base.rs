use crate::parser::types::Field;
use crate::registry::Registry;
use crate::{
    registry, ContextSelectionSet, InputValueResult, Positioned, Result, ServerResult, Value,
};
use std::borrow::Cow;

/// Represents a GraphQL type
///
/// All GraphQL types implement this trait, such as `Scalar`, `Object`, `Union` ...
pub trait Type {
    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    /// Introspection type name
    ///
    /// Is the return value of field `__typename`, the interface and union should return the current type, and the others return `Type::type_name`.
    fn introspection_type_name(&self) -> Cow<'static, str> {
        Self::type_name()
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> String;
}

/// Represents a GraphQL input value
pub trait InputValueType: Type + Sized {
    /// Parse from `Value`. None represents undefined.
    fn parse(value: Option<Value>) -> InputValueResult<Self>;

    /// Convert to a `Value` for introspection.
    fn to_value(&self) -> Value;
}

/// Represents a GraphQL output value
#[async_trait::async_trait]
pub trait OutputValueType: Type {
    /// Resolve an output value to `serde_json::Value`.
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<serde_json::Value>;
}

impl<T: Type + Send + Sync> Type for &T {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for &T {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<serde_json::Value> {
        T::resolve(*self, ctx, field).await
    }
}

impl<T: Type> Type for Result<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::qualified_type_name()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Sync> OutputValueType for Result<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<serde_json::Value> {
        match self {
            Ok(value) => Ok(value.resolve(ctx, field).await?),
            Err(err) => Err(err.clone().into_server_error().at(field.pos)),
        }
    }
}
