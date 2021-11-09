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

/// Define a scalar
///
/// If your type implemented `serde::Serialize` and `serde::Deserialize`, then you can use this macro to define a scalar more simply.
/// It helps you implement the `ScalarType::parse` and `ScalarType::to_value` functions by calling the [from_value](fn.from_value.html) and [to_value](fn.to_value.html) functions.
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
/// use serde::{Serialize, Deserialize};
/// use std::collections::HashMap;
///
/// #[derive(Serialize, Deserialize)]
/// struct MyValue {
///     a: i32,
///     b: HashMap<String, i32>,     
/// }
///
/// scalar!(MyValue);
///
/// // Rename to `MV`.
/// // scalar!(MyValue, "MV");
///
/// // Rename to `MV` and add description.
/// // scalar!(MyValue, "MV", "This is my value");
///
/// // Rename to `MV`, add description and specifiedByURL.
/// // scalar!(MyValue, "MV", "This is my value", "https://tools.ietf.org/html/rfc4122");
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self, input: MyValue) -> MyValue {
///         input
///     }
/// }
///
/// tokio::runtime::Runtime::new().unwrap().block_on(async move {
///     let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
///     let res = schema.execute(r#"{ value(input: {a: 10, b: {v1: 1, v2: 2} }) }"#).await.into_result().unwrap().data;
///     assert_eq!(res, value!({
///         "value": {
///             "a": 10,
///             "b": {"v1": 1, "v2": 2},
///         }
///     }));
/// });
///
///
/// ```
#[macro_export]
macro_rules! scalar {
    ($ty:ty, $name:literal, $desc:literal, $specified_by_url:literal) => {
        $crate::scalar_internal!(
            $ty,
            $name,
            ::std::option::Option::Some($desc),
            ::std::option::Option::Some($specified_by_url)
        );
    };

    ($ty:ty, $name:literal, $desc:literal) => {
        $crate::scalar_internal!(
            $ty,
            $name,
            ::std::option::Option::Some($desc),
            ::std::option::Option::None
        );
    };

    ($ty:ty, $name:literal) => {
        $crate::scalar_internal!(
            $ty,
            $name,
            ::std::option::Option::None,
            ::std::option::Option::None
        );
    };

    ($ty:ty) => {
        $crate::scalar_internal!(
            $ty,
            ::std::stringify!($ty),
            ::std::option::Option::None,
            ::std::option::Option::None
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! scalar_internal {
    ($ty:ty, $name:expr, $desc:expr, $specified_by_url:expr) => {
        impl $crate::ScalarType for $ty {
            fn parse(value: $crate::Value) -> $crate::InputValueResult<Self> {
                ::std::result::Result::Ok($crate::from_value(value)?)
            }

            fn to_value(&self) -> $crate::Value {
                $crate::to_value(self).unwrap_or_else(|_| $crate::Value::Null)
            }
        }

        impl $crate::InputType for $ty {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed($name)
            }

            fn create_type_info(
                registry: &mut $crate::registry::Registry,
            ) -> ::std::string::String {
                registry.create_input_type::<$ty, _>(|_| $crate::registry::MetaType::Scalar {
                    name: ::std::borrow::ToOwned::to_owned($name),
                    description: $desc,
                    is_valid: |value| <$ty as $crate::ScalarType>::is_valid(value),
                    visible: ::std::option::Option::None,
                    specified_by_url: $specified_by_url,
                })
            }

            fn parse(
                value: ::std::option::Option<$crate::Value>,
            ) -> $crate::InputValueResult<Self> {
                <$ty as $crate::ScalarType>::parse(value.unwrap_or_default())
            }

            fn to_value(&self) -> $crate::Value {
                <$ty as $crate::ScalarType>::to_value(self)
            }
        }

        #[$crate::async_trait::async_trait]
        impl $crate::OutputType for $ty {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed($name)
            }

            fn create_type_info(
                registry: &mut $crate::registry::Registry,
            ) -> ::std::string::String {
                registry.create_output_type::<$ty, _>(|_| $crate::registry::MetaType::Scalar {
                    name: ::std::borrow::ToOwned::to_owned($name),
                    description: $desc,
                    is_valid: |value| <$ty as $crate::ScalarType>::is_valid(value),
                    visible: ::std::option::Option::None,
                    specified_by_url: $specified_by_url,
                })
            }

            async fn resolve(
                &self,
                _: &$crate::ContextSelectionSet<'_>,
                _field: &$crate::Positioned<$crate::parser::types::Field>,
            ) -> $crate::ServerResult<$crate::Value> {
                ::std::result::Result::Ok($crate::ScalarType::to_value(self))
            }
        }
    };
}
