use crate::parser::types::Field;
use crate::registry::Registry;
use crate::{
    registry, ContextSelectionSet, FieldResult, InputValueResult, Positioned, Result, Value,
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
    ) -> Result<serde_json::Value>;
}

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
///         if let Value::Number(n) = &value {
///             if let Some(n) = n.as_i64() {
///                 return Ok(MyInt(n as i32));
///             }
///         }
///         Err(InputValueError::ExpectedType(value))
///     }
///
///     fn to_value(&self) -> Value {
///         Value::Number(self.0.into())
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
            Err(err) => Err(err
                .clone()
                .into_error_with_path(field.pos, ctx.path_node.as_ref())),
        }
    }
}
