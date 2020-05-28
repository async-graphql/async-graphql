use crate::registry::Registry;
use crate::{
    registry, Context, ContextSelectionSet, FieldResult, InputValueResult, Positioned, QueryError,
    Result, Value,
};
use async_graphql_parser::query::Field;
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
    /// Parse from `Value`，None represent undefined.
    fn parse(value: Option<Value>) -> InputValueResult<Self>;

    /// Convert to `Value` for introspection
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
    ) -> Result<serde_json::Value>;
}

#[allow(missing_docs)]
pub type BoxFieldFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(String, serde_json::Value)>> + 'a + Send>>;

/// Represents a GraphQL object
#[async_trait::async_trait]
pub trait ObjectType: OutputValueType {
    /// This function returns true of type `EmptyMutation` only
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value `serde_json::Value`.
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value>;

    /// Collect the fields with the `name` inline object
    fn collect_inline_fields<'a>(
        &'a self,
        name: &str,
        ctx: &ContextSelectionSet<'a>,
        futures: &mut Vec<BoxFieldFuture<'a>>,
    ) -> Result<()>
    where
        Self: Send + Sync + Sized,
    {
        if name == Self::type_name().as_ref()
            || ctx
                .schema_env
                .registry
                .implements
                .get(Self::type_name().as_ref())
                .map(|ty| ty.contains(name))
                .unwrap_or_default()
        {
            crate::collect_fields(ctx, self, futures)
        } else {
            Ok(())
        }
    }

    /// Query entities with params
    async fn find_entity(&self, ctx: &Context<'_>, _params: &Value) -> Result<serde_json::Value> {
        Err(QueryError::EntityNotFound.into_error(ctx.position()))
    }
}

/// Represents a GraphQL input object
pub trait InputObjectType: InputValueType {}

/// Represents a GraphQL scalar
///
/// You can implement the trait to create a custom scalar.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct MyInt(i32);
///
/// #[Scalar]
/// impl ScalarType for MyInt {
///     fn parse(value: Value) -> InputValueResult<Self> {
///         if let Value::Int(n) = value {
///             Ok(MyInt(n as i32))
///         } else {
///             Err(InputValueError::ExpectedType(value))
///         }
///     }
///
///     fn to_value(&self) -> Value {
///         Value::Int(self.0)
///     }
/// }
/// ```
pub trait ScalarType: Sized + Send {
    /// Parse a scalar value, return `Some(Self)` if successful, otherwise return `None`.
    fn parse(value: Value) -> InputValueResult<Self>;

    /// Checks for a valid scalar value.
    ///
    /// Implementing this function can find incorrect input values during the verification phase, which can improve performance.
    fn is_valid(_value: &Value) -> bool {
        true
    }

    /// Convert the scalar to `Value`.
    fn to_value(&self) -> Value;
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
    ) -> Result<serde_json::Value> {
        T::resolve(*self, ctx, field).await
    }
}

impl<T: Type + Send + Sync> Type for Box<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for Box<T> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    #[allow(clippy::borrowed_box)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        T::resolve(&*self, ctx, field).await
    }
}

impl<T: Type + Send + Sync> Type for Arc<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        T::create_type_info(registry)
    }
}

#[async_trait::async_trait]
impl<T: OutputValueType + Send + Sync> OutputValueType for Arc<T> {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        T::resolve(&*self, ctx, field).await
    }
}

impl<T: Type> Type for FieldResult<T> {
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
impl<T: OutputValueType + Sync> OutputValueType for FieldResult<T> {
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        field: &Positioned<Field>,
    ) -> crate::Result<serde_json::Value> {
        match self {
            Ok(value) => Ok(OutputValueType::resolve(value, ctx, field).await?),
            Err(err) => Err(err.clone().into_error_with_path(
                field.position(),
                match &ctx.path_node {
                    Some(path) => path.to_json(),
                    None => Vec::new(),
                },
            )),
        }
    }
}
