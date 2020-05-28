use crate::{registry, InputValueResult, InputValueType, Type, Value};
use std::borrow::Cow;

/// Similar to `Option`, but it has three states, `undefined`, `null` and `x`.
///
/// Spec: https://spec.graphql.org/June2018/#sec-Null-Value
#[allow(clippy::option_option)]
pub struct MaybeUndefined<T>(Option<Option<T>>);

impl<T> MaybeUndefined<T> {
    /// Returns true if the MaybeUndefined<T> is undefined.
    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.0.is_none()
    }

    /// Returns true if the MaybeUndefined<T> is null.
    #[inline]
    pub fn is_null(&self) -> bool {
        if let Some(None) = &self.0 {
            true
        } else {
            false
        }
    }

    /// Borrow the value, returns `None` if the value is `undefined` or `null`, otherwise returns `Some(T)`.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        match &self.0 {
            Some(Some(value)) => Some(value),
            _ => None,
        }
    }

    /// Convert MaybeUndefined<T> to Option<T>.
    #[inline]
    pub fn take(self) -> Option<T> {
        match self.0 {
            Some(Some(value)) => Some(value),
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
            None => Ok(Self(None)),
            Some(Value::Null) => Ok(Self(Some(None))),
            Some(value) => Ok(Self(Some(Option::<T>::parse(Some(value))?))),
        }
    }

    fn to_value(&self) -> Value {
        match &self.0 {
            Some(Some(value)) => value.to_value(),
            _ => Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_optional_type() {
        assert_eq!(MaybeUndefined::<i32>::type_name(), "Int");
        assert_eq!(MaybeUndefined::<i32>::qualified_type_name(), "Int");
        assert_eq!(&MaybeUndefined::<i32>::type_name(), "Int");
        assert_eq!(&MaybeUndefined::<i32>::qualified_type_name(), "Int");
    }
}
