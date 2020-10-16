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
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value(&self, input: MyValue) -> MyValue {
///         input
///     }
/// }
///
/// async_std::task::block_on(async move {
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
    ($ty:ty, $name:expr, $desc:literal) => {
        $crate::scalar_internal!(
            $ty,
            ::std::stringify!($ty),
            ::std::option::Option::Some($desc)
        );
    };

    ($ty:ty, $name:expr) => {
        $crate::scalar_internal!($ty, ::std::stringify!($ty), ::std::option::Option::None);
    };

    ($ty:ty) => {
        $crate::scalar_internal!($ty, ::std::stringify!($ty), ::std::option::Option::None);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! scalar_internal {
    ($ty:ty, $name:expr, $desc:expr) => {
        impl $crate::Type for $ty {
            fn type_name() -> ::std::borrow::Cow<'static, ::std::primitive::str> {
                ::std::borrow::Cow::Borrowed($name)
            }

            fn create_type_info(
                registry: &mut $crate::registry::Registry,
            ) -> ::std::string::String {
                registry.create_type::<$ty, _>(|_| $crate::registry::MetaType::Scalar {
                    name: $name.into(),
                    description: $desc,
                    is_valid: |value| <$ty as $crate::ScalarType>::is_valid(value),
                })
            }
        }

        impl $crate::ScalarType for $ty {
            fn parse(value: $crate::Value) -> $crate::InputValueResult<Self> {
                ::std::result::Result::Ok($crate::from_value(value)?)
            }

            fn to_value(&self) -> $crate::Value {
                $crate::to_value(self).unwrap_or_else(|_| $crate::Value::Null)
            }
        }

        impl $crate::InputValueType for $ty {
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
        impl $crate::OutputValueType for $ty {
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
