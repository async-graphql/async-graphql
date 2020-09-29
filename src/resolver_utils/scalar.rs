use crate::{InputValueResult, Value};

/// A GraphQL scalar.
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
///         Err(InputValueError::expected_type(value))
///     }
///
///     fn to_value(&self) -> Value {
///         Value::Number(self.0.into())
///     }
/// }
/// ```
pub trait ScalarType: Sized + Send {
    /// Parse a scalar value.
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
