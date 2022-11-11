use indexmap::IndexSet;

use crate::{
    dynamic::SchemaError,
    registry::{MetaType, Registry},
};

/// A GraphQL union type
///
/// # Examples
///
/// ```
/// use async_graphql::{dynamic::*, value, Value};
///
/// let obj_a = Object::new("MyObjA")
///     .field(Field::new("a", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(100))) })
///     }))
///     .field(Field::new("b", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(200))) })
///     }));
///
/// let obj_b = Object::new("MyObjB")
///     .field(Field::new("c", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(300))) })
///     }))
///     .field(Field::new("d", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(400))) })
///     }));
///
/// let union = Union::new("MyUnion")
///     .possible_type(obj_a.type_name())
///     .possible_type(obj_b.type_name());
///
/// let query = Object::new("Query")
///     .field(Field::new("valueA", TypeRef::named_nn(union.type_name()), |_| {
///         FieldFuture::new(async {
///             Ok(Some(FieldValue::with_type(FieldValue::NULL, "MyObjA")))
///         })
///     }))
///     .field(Field::new("valueB", TypeRef::named_nn(union.type_name()), |_| {
///         FieldFuture::new(async {
///             Ok(Some(FieldValue::with_type(FieldValue::NULL, "MyObjB")))
///         })
///     }));
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
///
/// let schema = Schema::build(query.type_name(), None, None)
///     .register(obj_a)
///     .register(obj_b)
///     .register(union)
///     .register(query)
///     .finish()?;
///
/// let query = r#"
///     {
///         valueA { ... on MyObjA { a b } ... on MyObjB { c d } }
///         valueB { ... on MyObjA { a b } ... on MyObjB { c d } }
///     }
/// "#;
///
/// assert_eq!(
///     schema.execute(query).await.into_result().unwrap().data,
///     value!({
///         "valueA": {
///             "a": 100,
///             "b": 200,
///         },
///         "valueB": {
///             "c": 300,
///             "d": 400,
///         }
///     })
/// );
///
/// # Ok::<_, SchemaError>(())
/// # }).unwrap();
/// ```
#[derive(Debug)]
pub struct Union {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) possible_types: IndexSet<String>,
}

impl Union {
    /// Create a GraphQL union type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            possible_types: Default::default(),
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

    /// Add a possible type to the union that must be an object
    #[inline]
    pub fn possible_type(mut self, ty: impl Into<String>) -> Self {
        self.possible_types.insert(ty.into());
        self
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        registry.types.insert(
            self.name.clone(),
            MetaType::Union {
                name: self.name.clone(),
                description: self.description.clone(),
                possible_types: self.possible_types.clone(),
                visible: None,
                inaccessible: false,
                tags: vec![],
                rust_typename: None,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_parser::Pos;

    use crate::{dynamic::*, value, PathSegment, ServerError, Value};

    #[tokio::test]
    async fn basic_union() {
        let obj_a = Object::new("MyObjA")
            .field(Field::new("a", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new("b", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));

        let obj_b = Object::new("MyObjB")
            .field(Field::new("c", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(300))) })
            }))
            .field(Field::new("d", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(400))) })
            }));

        let union = Union::new("MyUnion")
            .possible_type(obj_a.type_name())
            .possible_type(obj_b.type_name());

        let query = Object::new("Query")
            .field(Field::new(
                "valueA",
                TypeRef::named_nn(union.type_name()),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjA"))) }),
            ))
            .field(Field::new(
                "valueB",
                TypeRef::named_nn(union.type_name()),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjB"))) }),
            ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(obj_a)
            .register(obj_b)
            .register(union)
            .register(query)
            .finish()
            .unwrap();

        let query = r#"
            {
                valueA { __typename ... on MyObjA { a b } ... on MyObjB { c d } }
                valueB { __typename ... on MyObjA { a b } ... on MyObjB { c d } }
            }
        "#;
        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
                "valueA": {
                    "__typename": "MyObjA",
                    "a": 100,
                    "b": 200,
                },
                "valueB": {
                    "__typename": "MyObjB",
                    "c": 300,
                    "d": 400,
                }
            })
        );
    }

    #[tokio::test]
    async fn does_not_contain() {
        let obj_a = Object::new("MyObjA")
            .field(Field::new("a", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new("b", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));

        let obj_b = Object::new("MyObjB")
            .field(Field::new("c", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(300))) })
            }))
            .field(Field::new("d", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(400))) })
            }));

        let union = Union::new("MyUnion").possible_type(obj_a.type_name());

        let query = Object::new("Query").field(Field::new(
            "valueA",
            TypeRef::named_nn(union.type_name()),
            |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjB"))) }),
        ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(obj_a)
            .register(obj_b)
            .register(union)
            .register(query)
            .finish()
            .unwrap();

        let query = r#"
            {
                valueA { ... on MyObjA { a b } }
            }
        "#;
        assert_eq!(
            schema.execute(query).await.into_result().unwrap_err(),
            vec![ServerError {
                message: "internal: union \"MyUnion\" does not contain object \"MyObjB\""
                    .to_owned(),
                source: None,
                locations: vec![Pos {
                    column: 17,
                    line: 3
                }],
                path: vec![PathSegment::Field("valueA".to_owned())],
                extensions: None,
            }]
        );
    }
}
