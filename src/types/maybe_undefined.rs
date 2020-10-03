use crate::{registry, InputValueError, InputValueResult, InputValueType, Type, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

/// Similar to `Option`, but it has three states, `undefined`, `null` and `x`.
///
/// **Reference:** <https://spec.graphql.org/June2018/#sec-Null-Value>
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct Query;
///
/// #[Object]
/// impl Query {
///     async fn value1(&self, input: MaybeUndefined<i32>) -> i32 {
///         if input.is_null() {
///             1
///         } else if input.is_undefined() {
///             2
///         } else {
///             input.take().unwrap()
///         }
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
///     let query = r#"
///         {
///             v1:value1(input: 99)
///             v2:value1(input: null)
///             v3:value1
///         }"#;
///     assert_eq!(
///         schema.execute(query).await.into_result().unwrap().data,
///         serde_json::json!({
///             "v1": 99,
///             "v2": 1,
///             "v3": 2,
///         })
///     );
/// }
/// ```
#[allow(missing_docs)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum MaybeUndefined<T> {
    Undefined,
    Null,
    Value(T),
}

impl<T> Default for MaybeUndefined<T> {
    fn default() -> Self {
        Self::Undefined
    }
}

impl<T> MaybeUndefined<T> {
    /// Returns true if the MaybeUndefined<T> is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        if let MaybeUndefined::Undefined = self {
            true
        } else {
            false
        }
    }

    /// Returns true if the MaybeUndefined<T> is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        if let MaybeUndefined::Null = self {
            true
        } else {
            false
        }
    }

    /// Returns true if the MaybeUndefined<T> is value.
    #[inline]
    pub fn is_value(&self) -> bool {
        if let MaybeUndefined::Value(_) = self {
            true
        } else {
            false
        }
    }

    /// Borrow the value, returns `None` if the value is `undefined` or `null`, otherwise returns `Some(T)`.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        match self {
            MaybeUndefined::Value(value) => Some(value),
            _ => None,
        }
    }

    /// Convert MaybeUndefined<T> to Option<T>.
    #[inline]
    pub fn take(self) -> Option<T> {
        match self {
            MaybeUndefined::Value(value) => Some(value),
            _ => None,
        }
    }
}

impl<T: Type> Type for MaybeUndefined<T> {
    fn type_name() -> Cow<'static, str> {
        T::type_name()
    }

    fn qualified_type_name() -> String {
        T::type_name().to_string()
    }

    fn create_type_info(registry: &mut registry::Registry) -> String {
        T::create_type_info(registry);
        T::type_name().to_string()
    }
}

impl<T: InputValueType> InputValueType for MaybeUndefined<T> {
    fn parse(value: Option<Value>) -> InputValueResult<Self> {
        match value {
            None => Ok(MaybeUndefined::Undefined),
            Some(Value::Null) => Ok(MaybeUndefined::Null),
            Some(value) => Ok(MaybeUndefined::Value(
                T::parse(Some(value)).map_err(InputValueError::propagate)?,
            )),
        }
    }

    fn to_value(&self) -> Value {
        match self {
            MaybeUndefined::Value(value) => value.to_value(),
            _ => Value::Null,
        }
    }
}

impl<T: Serialize> Serialize for MaybeUndefined<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            MaybeUndefined::Value(value) => value.serialize(serializer),
            _ => serializer.serialize_none(),
        }
    }
}

impl<'de, T> Deserialize<'de> for MaybeUndefined<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<MaybeUndefined<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|value| match value {
            Some(value) => MaybeUndefined::Value(value),
            None => MaybeUndefined::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_maybe_undefined_type() {
        assert_eq!(MaybeUndefined::<i32>::type_name(), "Int");
        assert_eq!(MaybeUndefined::<i32>::qualified_type_name(), "Int");
        assert_eq!(&MaybeUndefined::<i32>::type_name(), "Int");
        assert_eq!(&MaybeUndefined::<i32>::qualified_type_name(), "Int");
    }

    #[test]
    fn test_maybe_undefined_serde() {
        assert_eq!(
            serde_json::to_string(&MaybeUndefined::Value(100i32)).unwrap(),
            "100"
        );

        assert_eq!(
            serde_json::from_str::<MaybeUndefined<i32>>("100").unwrap(),
            MaybeUndefined::Value(100)
        );
        assert_eq!(
            serde_json::from_str::<MaybeUndefined<i32>>("null").unwrap(),
            MaybeUndefined::Null
        );

        #[derive(Serialize, Deserialize, Eq, PartialEq, Debug)]
        struct A {
            a: MaybeUndefined<i32>,
        }

        assert_eq!(
            serde_json::to_string(&A {
                a: MaybeUndefined::Value(100i32)
            })
            .unwrap(),
            r#"{"a":100}"#
        );

        assert_eq!(
            serde_json::to_string(&A {
                a: MaybeUndefined::Null,
            })
            .unwrap(),
            r#"{"a":null}"#
        );

        assert_eq!(
            serde_json::to_string(&A {
                a: MaybeUndefined::Undefined,
            })
            .unwrap(),
            r#"{"a":null}"#
        );

        assert_eq!(
            serde_json::from_str::<A>(r#"{"a":100}"#).unwrap(),
            A {
                a: MaybeUndefined::Value(100i32)
            }
        );

        assert_eq!(
            serde_json::from_str::<A>(r#"{"a":null}"#).unwrap(),
            A {
                a: MaybeUndefined::Null
            }
        );

        assert_eq!(
            serde_json::from_str::<A>(r#"{}"#).unwrap(),
            A {
                a: MaybeUndefined::Null
            }
        );
    }
}
