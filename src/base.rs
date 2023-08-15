use std::{
    borrow::Cow,
    sync::{Arc, Weak},
};

use async_graphql_value::ConstValue;

use crate::{
    parser::types::Field,
    registry::{self, Registry},
    ContainerType, Context, ContextSelectionSet, Error, InputValueError, InputValueResult,
    Positioned, Result, ServerResult, Value,
};

#[doc(hidden)]
pub trait Description {
    fn description() -> &'static str;
}

/// Used to specify the GraphQL Type name.
pub trait TypeName: Send + Sync {
    /// Returns a GraphQL type name.
    fn type_name() -> Cow<'static, str>;
}

/// Represents a GraphQL input type.
pub trait InputType: Send + Sync + Sized {
    /// The raw type used for validator.
    ///
    /// Usually it is `Self`, but the wrapper type is its internal type.
    ///
    /// For example:
    ///
    /// `i32::RawValueType` is `i32`
    /// `Option<i32>::RawValueType` is `i32`.
    type RawValueType;

    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> String;

    /// Parse from `Value`. None represents undefined.
    fn parse(value: Option<Value>) -> InputValueResult<Self>;

    /// Convert to a `Value` for introspection.
    fn to_value(&self) -> Value;

    /// Get the federation fields, only for InputObject.
    #[doc(hidden)]
    fn federation_fields() -> Option<String> {
        None
    }

    /// Returns a reference to the raw value.
    fn as_raw_value(&self) -> Option<&Self::RawValueType>;
}

/// Represents a GraphQL output type.
#[async_trait::async_trait]
pub trait OutputType: Send + Sync {
    /// Type the name.
    fn type_name() -> Cow<'static, str>;

    /// Qualified typename.
    fn qualified_type_name() -> String {
        format!("{}!", Self::type_name())
    }

    /// Introspection type name
    ///
    /// Is the return value of field `__typename`, the interface and union
    /// should return the current type, and the others return `Type::type_name`.
    fn introspection_type_name(&self) -> Cow<'static, str> {
        Self::type_name()
    }

    /// Create type information in the registry and return qualified typename.
    fn create_type_info(registry: &mut registry::Registry) -> String;

    /// Resolve an output value to `async_graphql::Value`.
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value>;
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for &T {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        T::resolve(*self, ctx, field).await
    }
}

#[async_trait::async_trait]
impl<T: OutputType + Sync, E: Into<Error> + Send + Sync + Clone> OutputType for Result<T, E> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

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
impl<T: ObjectType + ?Sized> ObjectType for &T {}

#[async_trait::async_trait]
impl<T: ObjectType + ?Sized> ObjectType for Box<T> {}

#[async_trait::async_trait]
impl<T: ObjectType + ?Sized> ObjectType for Arc<T> {}

/// A GraphQL interface.
pub trait InterfaceType: ContainerType {}

/// A GraphQL interface.
pub trait UnionType: ContainerType {}

/// A GraphQL input object.
pub trait InputObjectType: InputType {}

/// A GraphQL oneof input object.
pub trait OneofObjectType: InputObjectType {}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for Box<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

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
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value)
            .map(Box::new)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(&self)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.as_ref().as_raw_value()
    }
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for Arc<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

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
    type RawValueType = T::RawValueType;

    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }

    fn parse(value: Option<ConstValue>) -> InputValueResult<Self> {
        T::parse(value)
            .map(Arc::new)
            .map_err(InputValueError::propagate)
    }

    fn to_value(&self) -> ConstValue {
        T::to_value(&self)
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        self.as_ref().as_raw_value()
    }
}

#[async_trait::async_trait]
impl<T: OutputType + ?Sized> OutputType for Weak<T> {
    fn type_name() -> Cow<'static, str> {
        <Option<Arc<T>> as OutputType>::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        <Option<Arc<T>> as OutputType>::create_type_info(registry)
    }

    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        self.upgrade().resolve(ctx, field).await
    }
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait ComplexObject {
    fn fields(registry: &mut registry::Registry) -> Vec<(String, registry::MetaField)>;

    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>>;
}
