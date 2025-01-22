use crate::{registry, Object};

pub struct __EnumValue<'a> {
    pub value: &'a registry::MetaEnumValue,
}

/// One possible value for a given Enum. Enum values are unique values, not a
/// placeholder for a string or numeric value. However an Enum value is returned
/// in a JSON response as a string.
#[Object(internal, name = "__EnumValue")]
impl __EnumValue<'_> {
    #[inline]
    async fn name(&self) -> &str {
        &self.value.name
    }

    #[inline]
    async fn description(&self) -> Option<&str> {
        self.value.description.as_deref()
    }

    #[inline]
    async fn is_deprecated(&self) -> bool {
        self.value.deprecation.is_deprecated()
    }

    #[inline]
    async fn deprecation_reason(&self) -> Option<&str> {
        self.value.deprecation.reason()
    }
}
