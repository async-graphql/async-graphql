use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use super::{directive::to_meta_directive_invocation, Directive};
use crate::{
    dynamic::SchemaError,
    registry::{MetaType, Registry, ScalarValidatorFn},
    Value,
};

/// A GraphQL scalar type
///
/// # Examples
///
/// ```
/// use async_graphql::{dynamic::*, value, Value};
///
/// let my_scalar = Scalar::new("MyScalar");
///
/// let query = Object::new("Query").field(Field::new("value", TypeRef::named_nn(my_scalar.type_name()), |ctx| {
///     FieldFuture::new(async move { Ok(Some(Value::from("abc"))) })
/// }));
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
///
/// let schema = Schema::build(query.type_name(), None, None)
///     .register(my_scalar)
///     .register(query)
///     .finish()?;
///
/// assert_eq!(
///    schema
///        .execute("{ value }")
///        .await
///        .into_result()
///        .unwrap()
///        .data,
///    value!({ "value": "abc" })
/// );
///
/// # Ok::<_, SchemaError>(())
/// # }).unwrap();
/// ```
pub struct Scalar {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) specified_by_url: Option<String>,
    pub(crate) validator: Option<ScalarValidatorFn>,
    inaccessible: bool,
    tags: Vec<String>,
    pub(crate) directives: Vec<Directive>,
}

impl Debug for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scalar")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("specified_by_url", &self.specified_by_url)
            .field("inaccessible", &self.inaccessible)
            .field("tags", &self.tags)
            .finish()
    }
}

impl Scalar {
    /// Create a GraphQL scalar type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            specified_by_url: None,
            validator: None,
            inaccessible: false,
            tags: Vec::new(),
            directives: Vec::new(),
        }
    }

    impl_set_description!();
    impl_set_inaccessible!();
    impl_set_tags!();
    impl_directive!();

    /// Set the validator
    #[inline]
    pub fn validator(self, validator: impl Fn(&Value) -> bool + Send + Sync + 'static) -> Self {
        Self {
            validator: Some(Arc::new(validator)),
            ..self
        }
    }

    #[inline]
    pub(crate) fn validate(&self, value: &Value) -> bool {
        match &self.validator {
            Some(validator) => (validator)(value),
            None => true,
        }
    }

    /// Set the specified by url
    #[inline]
    pub fn specified_by_url(self, specified_by_url: impl Into<String>) -> Self {
        Self {
            specified_by_url: Some(specified_by_url.into()),
            ..self
        }
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        registry.types.insert(
            self.name.clone(),
            MetaType::Scalar {
                name: self.name.clone(),
                description: self.description.clone(),
                is_valid: self.validator.clone(),
                visible: None,
                inaccessible: self.inaccessible,
                tags: self.tags.clone(),
                specified_by_url: self.specified_by_url.clone(),
                directive_invocations: to_meta_directive_invocation(self.directives.clone()),
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_parser::Pos;

    use crate::{dynamic::*, value, PathSegment, ServerError};

    #[tokio::test]
    async fn custom_scalar() {
        let scalar = Scalar::new("MyScalar");
        let query = Object::new("Query").field(Field::new(
            "value",
            TypeRef::named_nn(scalar.type_name()),
            |_| {
                FieldFuture::new(async move {
                    Ok(Some(value!({
                        "a": 1,
                        "b": "abc",
                    })))
                })
            },
        ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(scalar)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ value }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "value": {
                    "a": 1,
                    "b": "abc",
                }
            })
        );
    }

    #[tokio::test]
    async fn invalid_scalar_value() {
        let scalar = Scalar::new("MyScalar");
        let query = Object::new("Query").field(Field::new(
            "value",
            TypeRef::named_nn(scalar.type_name()),
            |_| FieldFuture::new(async move { Ok(Some(FieldValue::owned_any(10i32))) }),
        ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(scalar)
            .finish()
            .unwrap();

        assert_eq!(
            schema.execute("{ value }").await.into_result().unwrap_err(),
            vec![ServerError {
                message: "internal: invalid value for scalar \"MyScalar\", expected \"FieldValue::Value\""
                    .to_owned(),
                source: None,
                locations: vec![Pos { column: 3, line: 1 }],
                path: vec![PathSegment::Field("value".to_owned())],
                extensions: None,
            }]
        );
    }
}
