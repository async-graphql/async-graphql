use indexmap::{IndexMap, IndexSet};

use crate::{
    dynamic::{InputValue, SchemaError, TypeRef},
    registry::{Deprecation, MetaField, MetaType, Registry},
};

/// A GraphQL interface field type
///
/// # Examples
///
/// ```
/// use async_graphql::{dynamic::*, value, Value};
///
/// let obj_a = Object::new("MyObjA")
///     .implement("MyInterface")
///     .field(Field::new("a", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(100))) })
///     }))
///     .field(Field::new("b", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(200))) })
///     }));
///
/// let obj_b = Object::new("MyObjB")
///     .implement("MyInterface")
///     .field(Field::new("a", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(300))) })
///     }))
///     .field(Field::new("c", TypeRef::named_nn(TypeRef::INT), |_| {
///         FieldFuture::new(async { Ok(Some(Value::from(400))) })
///     }));
///
/// let interface = Interface::new("MyInterface").field(InterfaceField::new("a", TypeRef::named_nn(TypeRef::INT)));
///
/// let query = Object::new("Query")
///     .field(Field::new("valueA", TypeRef::named_nn(interface.type_name()), |_| {
///         FieldFuture::new(async {
///             Ok(Some(FieldValue::with_type(FieldValue::NULL, "MyObjA")))
///         })
///     }))
///     .field(Field::new("valueB", TypeRef::named_nn(interface.type_name()), |_| {
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
///     .register(interface)
///     .register(query)
///     .finish()?;
///
/// let query = r#"
///     fragment A on MyObjA { b }
///
///     fragment B on MyObjB { c }
///
///     {
///         valueA { a ...A ...B }
///         valueB { a ...A ...B }
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
///             "a": 300,
///             "c": 400,
///         }
///     })
/// );
///
/// # Ok::<_, SchemaError>(())
/// # }).unwrap();
/// ```
#[derive(Debug)]
pub struct InterfaceField {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) arguments: IndexMap<String, InputValue>,
    pub(crate) ty: TypeRef,
    pub(crate) deprecation: Deprecation,
    pub(crate) external: bool,
    pub(crate) requires: Option<String>,
    pub(crate) provides: Option<String>,
    pub(crate) shareable: bool,
    pub(crate) inaccessible: bool,
    pub(crate) tags: Vec<String>,
    pub(crate) override_from: Option<String>,
}

impl InterfaceField {
    /// Create a GraphQL interface field type
    pub fn new(name: impl Into<String>, ty: impl Into<TypeRef>) -> Self {
        Self {
            name: name.into(),
            description: None,
            arguments: Default::default(),
            ty: ty.into(),
            deprecation: Deprecation::NoDeprecated,
            external: false,
            requires: None,
            provides: None,
            shareable: false,
            inaccessible: false,
            tags: Vec::new(),
            override_from: None,
        }
    }

    impl_set_description!();
    impl_set_deprecation!();
    impl_set_external!();
    impl_set_requires!();
    impl_set_provides!();
    impl_set_shareable!();
    impl_set_inaccessible!();
    impl_set_tags!();
    impl_set_override_from!();

    /// Add an argument to the field
    #[inline]
    pub fn argument(mut self, input_value: InputValue) -> Self {
        self.arguments.insert(input_value.name.clone(), input_value);
        self
    }
}

/// A GraphQL interface type
#[derive(Debug)]
pub struct Interface {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) fields: IndexMap<String, InterfaceField>,
    pub(crate) implements: IndexSet<String>,
    keys: Vec<String>,
    extends: bool,
    inaccessible: bool,
    tags: Vec<String>,
}

impl Interface {
    /// Create a GraphQL interface type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            fields: Default::default(),
            implements: Default::default(),
            keys: Vec::new(),
            extends: false,
            inaccessible: false,
            tags: Vec::new(),
        }
    }

    impl_set_description!();
    impl_set_extends!();
    impl_set_inaccessible!();
    impl_set_tags!();

    /// Add a field to the interface type
    #[inline]
    pub fn field(mut self, field: InterfaceField) -> Self {
        assert!(
            !self.fields.contains_key(&field.name),
            "Field `{}` already exists",
            field.name
        );
        self.fields.insert(field.name.clone(), field);
        self
    }

    /// Add an implement to the interface type
    #[inline]
    pub fn implement(mut self, interface: impl Into<String>) -> Self {
        let interface = interface.into();
        assert!(
            !self.implements.contains(&interface),
            "Implement `{}` already exists",
            interface
        );
        self.implements.insert(interface);
        self
    }

    /// Add an entity key
    ///
    /// See also: [`Object::key`](crate::dynamic::Object::key)
    pub fn key(mut self, fields: impl Into<String>) -> Self {
        self.keys.push(fields.into());
        self
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub(crate) fn is_entity(&self) -> bool {
        !self.keys.is_empty()
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        let mut fields = IndexMap::new();

        for field in self.fields.values() {
            let mut args = IndexMap::new();

            for argument in field.arguments.values() {
                args.insert(argument.name.clone(), argument.to_meta_input_value());
            }

            fields.insert(
                field.name.clone(),
                MetaField {
                    name: field.name.clone(),
                    description: field.description.clone(),
                    args,
                    ty: field.ty.to_string(),
                    deprecation: field.deprecation.clone(),
                    cache_control: Default::default(),
                    external: field.external,
                    requires: field.requires.clone(),
                    provides: field.provides.clone(),
                    visible: None,
                    shareable: field.shareable,
                    inaccessible: field.inaccessible,
                    tags: field.tags.clone(),
                    override_from: field.override_from.clone(),
                    compute_complexity: None,
                    directive_invocations: vec![],
                },
            );
        }

        registry.types.insert(
            self.name.clone(),
            MetaType::Interface {
                name: self.name.clone(),
                description: self.description.clone(),
                fields,
                possible_types: Default::default(),
                extends: self.extends,
                keys: if !self.keys.is_empty() {
                    Some(self.keys.clone())
                } else {
                    None
                },
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

    use crate::{dynamic::*, value, PathSegment, ServerError, Value};

    #[tokio::test]
    async fn basic_interface() {
        let obj_a = Object::new("MyObjA")
            .implement("MyInterface")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));

        let obj_b = Object::new("MyObjB")
            .implement("MyInterface")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(300))) })
            }))
            .field(Field::new("c", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(400))) })
            }));

        let interface = Interface::new("MyInterface")
            .field(InterfaceField::new("a", TypeRef::named(TypeRef::INT)));

        let query = Object::new("Query")
            .field(Field::new(
                "valueA",
                TypeRef::named_nn(interface.type_name()),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjA"))) }),
            ))
            .field(Field::new(
                "valueB",
                TypeRef::named_nn(interface.type_name()),
                |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjB"))) }),
            ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(obj_a)
            .register(obj_b)
            .register(interface)
            .register(query)
            .finish()
            .unwrap();

        let query = r#"
        fragment A on MyObjA {
            b
        }

        fragment B on MyObjB {
            c
        }
        
        {
            valueA { __typename a ...A ...B }
            valueB { __typename a ...A ...B }
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
                    "a": 300,
                    "c": 400,
                }
            })
        );
    }

    #[tokio::test]
    async fn does_not_implement() {
        let obj_a = Object::new("MyObjA")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));

        let interface = Interface::new("MyInterface")
            .field(InterfaceField::new("a", TypeRef::named(TypeRef::INT)));

        let query = Object::new("Query").field(Field::new(
            "valueA",
            TypeRef::named_nn(interface.type_name()),
            |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL.with_type("MyObjA"))) }),
        ));

        let schema = Schema::build(query.type_name(), None, None)
            .register(obj_a)
            .register(interface)
            .register(query)
            .finish()
            .unwrap();

        let query = r#"
        {
            valueA { a }
        }
        "#;
        assert_eq!(
            schema.execute(query).await.into_result().unwrap_err(),
            vec![ServerError {
                message: "internal: object \"MyObjA\" does not implement interface \"MyInterface\""
                    .to_owned(),
                source: None,
                locations: vec![Pos {
                    column: 13,
                    line: 3
                }],
                path: vec![PathSegment::Field("valueA".to_owned())],
                extensions: None,
            }]
        );
    }
    #[tokio::test]
    async fn query_type_condition() {
        struct MyObjA;
        let obj_a = Object::new("MyObjA")
            .implement("MyInterface")
            .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(100))) })
            }))
            .field(Field::new("b", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(Some(Value::from(200))) })
            }));
        let interface = Interface::new("MyInterface")
            .field(InterfaceField::new("a", TypeRef::named(TypeRef::INT)));
        let query = Object::new("Query");
        let query = query.field(Field::new(
            "valueA",
            TypeRef::named_nn(obj_a.type_name()),
            |_| FieldFuture::new(async { Ok(Some(FieldValue::owned_any(MyObjA))) }),
        ));
        let schema = Schema::build(query.type_name(), None, None)
            .register(obj_a)
            .register(interface)
            .register(query)
            .finish()
            .unwrap();
        let query = r#"
        {
            valueA { __typename
            b
            ... on MyInterface { a } }
        }
        "#;
        assert_eq!(
            schema.execute(query).await.into_result().unwrap().data,
            value!({
                "valueA": {
                    "__typename": "MyObjA",
                    "b": 200,
                    "a": 100,
                }
            })
        );
    }
}
