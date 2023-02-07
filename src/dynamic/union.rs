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
    inaccessible: bool,
    tags: Vec<String>,
}

impl Union {
    /// Create a GraphQL union type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            possible_types: Default::default(),
            inaccessible: false,
            tags: Vec::new(),
        }
    }

    impl_set_description!();
    impl_set_inaccessible!();
    impl_set_tags!();

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
                inaccessible: self.inaccessible,
                tags: self.tags.clone(),
                rust_typename: None,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_graphql_parser::Pos;

    use crate::{dynamic::*, value, PathSegment, Request, ServerError, Value};

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

    #[tokio::test]
    async fn test_query() {
        struct Dog;
        struct Cat;
        struct Snake;
        // enum
        #[allow(dead_code)]
        enum Animal {
            Dog(Dog),
            Cat(Cat),
            Snake(Snake),
        }
        struct Query {
            pet: Animal,
        }

        impl Animal {
            fn to_field_value(&self) -> FieldValue {
                match self {
                    Animal::Dog(dog) => FieldValue::borrowed_any(dog).with_type("Dog"),
                    Animal::Cat(cat) => FieldValue::borrowed_any(cat).with_type("Cat"),
                    Animal::Snake(snake) => FieldValue::borrowed_any(snake).with_type("Snake"),
                }
            }
        }
        fn create_schema() -> Schema {
            // interface
            let named = Interface::new("Named");
            let named = named.field(InterfaceField::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
            ));
            // dog
            let dog = Object::new("Dog");
            let dog = dog.field(Field::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
                |_ctx| FieldFuture::new(async move { Ok(Some(Value::from("dog"))) }),
            ));
            let dog = dog.field(Field::new(
                "power",
                TypeRef::named_nn(TypeRef::INT),
                |_ctx| FieldFuture::new(async move { Ok(Some(Value::from(100))) }),
            ));
            let dog = dog.implement("Named");
            // cat
            let cat = Object::new("Cat");
            let cat = cat.field(Field::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
                |_ctx| FieldFuture::new(async move { Ok(Some(Value::from("cat"))) }),
            ));
            let cat = cat.field(Field::new(
                "life",
                TypeRef::named_nn(TypeRef::INT),
                |_ctx| FieldFuture::new(async move { Ok(Some(Value::from(9))) }),
            ));
            let cat = cat.implement("Named");
            // snake
            let snake = Object::new("Snake");
            let snake = snake.field(Field::new(
                "length",
                TypeRef::named_nn(TypeRef::INT),
                |_ctx| FieldFuture::new(async move { Ok(Some(Value::from(200))) }),
            ));
            // animal
            let animal = Union::new("Animal");
            let animal = animal.possible_type("Dog");
            let animal = animal.possible_type("Cat");
            let animal = animal.possible_type("Snake");
            // query

            let query = Object::new("Query");
            let query = query.field(Field::new("pet", TypeRef::named_nn("Animal"), |ctx| {
                FieldFuture::new(async move {
                    let query = ctx.parent_value.try_downcast_ref::<Query>()?;
                    Ok(Some(query.pet.to_field_value()))
                })
            }));

            let schema = Schema::build(query.type_name(), None, None);
            let schema = schema
                .register(query)
                .register(named)
                .register(dog)
                .register(cat)
                .register(snake)
                .register(animal);

            schema.finish().unwrap()
        }

        let schema = create_schema();
        let query = r#"
            query {
                dog: pet {
                    ... on Dog {
                        __dog_typename: __typename
                        name
                        power
                    }
                }
                named: pet {
                    ... on Named {
                        __named_typename: __typename
                        name
                    }
                }
            }
        "#;
        let root = Query {
            pet: Animal::Dog(Dog),
        };
        let req = Request::new(query).root_value(FieldValue::owned_any(root));
        let res = schema.execute(req).await;

        assert_eq!(
            res.data.into_json().unwrap(),
            serde_json::json!({
                "dog": {
                    "__dog_typename": "Dog",
                    "name": "dog",
                    "power": 100
                },
                "named": {
                    "__named_typename": "Dog",
                    "name": "dog"
                }
            })
        );
    }
}
