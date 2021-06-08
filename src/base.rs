use std::borrow::Cow;
use std::sync::Arc;

use async_graphql_value::ConstValue;

use crate::parser::types::Field;
use crate::registry::{self, Registry};
use crate::{
    ContainerType, Context, ContextSelectionSet, Error, InputValueError, InputValueResult,
    Positioned, Result, ServerResult, Value,
};

#[doc(hidden)]
pub trait Description {
    fn description() -> &'static str;
}

/// Represents a GraphQL type.
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

/// Represents a GraphQL input value.
pub trait InputType: Type + Send + Sync + Sized {
    /// Parse from `Value`. None represents undefined.
    fn parse(value: Option<Value>) -> InputValueResult<Self>;

    /// Convert to a `Value` for introspection.
    fn to_value(&self) -> Value;

    /// Get the federation fields, only for InputObject.
    #[doc(hidden)]
    fn federation_fields() -> Option<String> {
        None
    }
}

/// Represents a GraphQL output value.
#[async_trait::async_trait]
pub trait OutputType: Type + Send + Sync {
    /// Resolve an output value to `async_graphql::Value`.
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value>;
}

impl<T: Type + ?Sized> Type for &T {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for &T {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        T::resolve(*self, ctx, field).await
    }
}

impl<T: Type, E: Into<Error> + Send + Sync + Clone> Type for Result<T, E> {
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
impl<T: OutputType + Sync, E: Into<Error> + Send + Sync + Clone> OutputType for Result<T, E> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        match self {
            Ok(value) => value.resolve(ctx, field).await,
            Err(err) => {
                return Err(ctx.set_error_path(err.clone().into().into_server_error(field.pos)))
            }
        }
    }
}

/// A GraphQL object.
pub trait ObjectType: ContainerType {}

#[async_trait::async_trait]
impl<T: ObjectType> ObjectType for &T {}

/// A GraphQL interface.
pub trait InterfaceType: ContainerType {}

/// A GraphQL interface.
pub trait UnionType: ContainerType {}

/// A GraphQL input object.
pub trait InputObjectType: InputType {}

impl<T: Type + ?Sized> Type for Box<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for Box<T> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        T::resolve(&**self, ctx, field).await
    }
}

#[async_trait::async_trait]
impl<T: InputType> InputType for Box<T> {
    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value)
            .map(Box::new)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(&self)
    }
}

impl<T: Type + ?Sized> Type for Arc<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for Arc<T> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        T::resolve(&**self, ctx, field).await
    }
}

impl<T: InputType> InputType for Arc<T> {
    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value)
            .map(Arc::new)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(&self)
    }
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait ComplexObject {
    fn fields(registry: &mut registry::Registry) -> Vec<(String, registry::MetaField)>;

    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>>;
}
