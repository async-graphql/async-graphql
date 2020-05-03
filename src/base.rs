use crate::registry::Registry;
use crate::{registry, Context, ContextSelectionSet, FieldResult, QueryError, Result, ID};
use graphql_parser::query::Value;
use graphql_parser::Pos;
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

    /// Returns a `GlobalID` that is unique among all types.
    fn global_id(id: ID) -> ID {
        base64::encode(format!("{}:{}", Self::type_name(), id)).into()
    }

    /// Parse `GlobalID`.
    fn from_global_id(id: ID) -> Option<ID> {
        let v: Vec<&str> = id.splitn(2, ':').collect();
        if v.len() != 2 {
            return None;
        }
        if v[0] != Self::type_name() {
            return None;
        }
        Some(v[1].to_string().into())
    }
}

/// Represents a GraphQL input value
pub trait InputValueType: Type + Sized {
    /// Parse from `Value`
    fn parse(value: &Value) -> Option<Self>;
}

/// Represents a GraphQL output value
#[async_trait::async_trait]
pub trait OutputValueType: Type {
    /// Resolve an output value to `serde_json::Value`.
    async fn resolve(
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
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
        _pos: Pos,
        ctx: &ContextSelectionSet<'a>,
        futures: &mut Vec<BoxFieldFuture<'a>>,
    ) -> Result<()>
    where
        Self: Send + Sync + Sized,
    {
        if name == Self::type_name().as_ref()
            || ctx
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
    async fn find_entity(
        &self,
        _ctx: &Context<'_>,
        pos: Pos,
        _params: &Value,
    ) -> Result<serde_json::Value> {
        Err(QueryError::EntityNotFound.into_error(pos))
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
///     fn type_name() -> &'static str {
///         "MyInt"
///     }
///
///     fn parse(value: &Value) -> Option<Self> {
///         if let Value::Int(n) = value {
///             Some(MyInt(n.as_i64().unwrap() as i32))
///         } else {
///             None
///         }
///     }
///
///     fn to_json(&self) -> Result<serde_json::Value> {
///         Ok(self.0.into())
///     }
/// }
/// ```
pub trait ScalarType: Sized + Send {
    /// The type name of a scalar.
    fn type_name() -> &'static str;

    /// The description of a scalar.
    fn description() -> Option<&'static str> {
        None
    }

    /// Parse a scalar value, return `Some(Self)` if successful, otherwise return `None`.
    fn parse(value: &Value) -> Option<Self>;

    /// Checks for a valid scalar value.
    ///
    /// The default implementation is to try to parse it, and in some cases you can implement this on your own to improve performance.
    fn is_valid(value: &Value) -> bool {
        Self::parse(value).is_some()
    }

    /// Convert the scalar value to json value.
    fn to_json(&self) -> Result<serde_json::Value>;
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
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
    ) -> Result<serde_json::Value> {
        T::resolve(*value, ctx, pos).await
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
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
    ) -> Result<serde_json::Value> {
        T::resolve(&*value, ctx, pos).await
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
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
    ) -> Result<serde_json::Value> {
        T::resolve(&*value, ctx, pos).await
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
        value: &Self,
        ctx: &ContextSelectionSet<'_>,
        pos: Pos,
    ) -> crate::Result<serde_json::Value> where {
        match value.as_ref() {
            Ok(value) => Ok(OutputValueType::resolve(value, ctx, pos).await?),
            Err(err) => Err(err.clone().into_error_with_path(
                pos,
                match &ctx.path_node {
                    Some(path) => path.to_json(),
                    None => serde_json::Value::Null,
                },
            )),
        }
    }
}
