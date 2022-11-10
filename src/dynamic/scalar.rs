use crate::{
    dynamic::{misc::NamedTypeRefBuilder, SchemaError, TypeRef},
    registry::{MetaType, Registry},
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
/// let query = Object::new("Query").field(Field::new("value", my_scalar.type_ref(), |ctx| {
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
#[derive(Debug)]
pub struct Scalar {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) specified_by_url: Option<String>,
}

impl Scalar {
    /// Create a GraphQL scalar type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            specified_by_url: None,
        }
    }

    /// Set the description
    #[inline]
    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
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

    /// Returns the type reference
    #[inline]
    pub fn type_ref(&self) -> NamedTypeRefBuilder {
        TypeRef::named(self.name.clone())
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        registry.types.insert(
            self.name.clone(),
            MetaType::Scalar {
                name: self.name.clone(),
                description: self.description.clone(),
                is_valid: |_| true,
                visible: None,
                inaccessible: false,
                tags: vec![],
                specified_by_url: self.specified_by_url.clone(),
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
        let query = Object::new("Query").field(Field::new("value", scalar.type_ref(), |_| {
            FieldFuture::new(async move {
                Ok(Some(value!({
                    "a": 1,
                    "b": "abc",
                })))
            })
        }));

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
        let query = Object::new("Query").field(Field::new("value", scalar.type_ref(), |_| {
            FieldFuture::new(async move { Ok(Some(FieldValue::owned_any(10i32))) })
        }));

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
