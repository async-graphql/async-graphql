use indexmap::{IndexMap, IndexSet};

use crate::{
    dynamic::{Field, SchemaError},
    registry::{MetaField, MetaType, Registry},
};

/// A GraphQL object type
///
/// # Examples
///
/// ```
/// use async_graphql::{dynamic::*, value, Value};
///
/// let query = Object::new("Query").field(Field::new("value", TypeRef::named_nn(TypeRef::STRING), |ctx| {
///     FieldFuture::new(async move { Ok(Some(Value::from("abc"))) })
/// }));
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
///
/// let schema = Schema::build(query.type_name(), None, None)
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
pub struct Object {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) fields: IndexMap<String, Field>,
    pub(crate) implements: IndexSet<String>,
    keys: Vec<String>,
    extends: bool,
    shareable: bool,
    resolvable: bool,
    inaccessible: bool,
    interface_object: bool,
    tags: Vec<String>,
}

impl Object {
    /// Create a GraphQL object type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            fields: Default::default(),
            implements: Default::default(),
            keys: Vec::new(),
            extends: false,
            shareable: false,
            resolvable: true,
            inaccessible: false,
            interface_object: false,
            tags: Vec::new(),
        }
    }

    impl_set_description!();
    impl_set_extends!();
    impl_set_shareable!();
    impl_set_inaccessible!();
    impl_set_interface_object!();
    impl_set_tags!();

    /// Add an field to the object
    #[inline]
    pub fn field(mut self, field: Field) -> Self {
        assert!(
            !self.fields.contains_key(&field.name),
            "Field `{}` already exists",
            field.name
        );
        self.fields.insert(field.name.clone(), field);
        self
    }

    /// Add an implement to the object
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
    /// # Examples
    ///
    /// ```
    /// use async_graphql::{dynamic::*, Value};
    ///
    /// let obj = Object::new("MyObj")
    ///     .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
    ///         FieldFuture::new(async move { Ok(Some(Value::from(10))) })
    ///     }))
    ///     .field(Field::new("b", TypeRef::named(TypeRef::INT), |_| {
    ///         FieldFuture::new(async move { Ok(Some(Value::from(20))) })
    ///     }))
    ///     .field(Field::new("c", TypeRef::named(TypeRef::INT), |_| {
    ///         FieldFuture::new(async move { Ok(Some(Value::from(30))) })
    ///     }))
    ///     .key("a b")
    ///     .key("c");
    /// ```
    pub fn key(mut self, fields: impl Into<String>) -> Self {
        self.keys.push(fields.into());
        self
    }

    /// Make the entity unresolvable by the current subgraph
    ///
    /// Most commonly used to reference an entity without contributing fields.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_graphql::{dynamic::*, Value};
    ///
    /// let obj = Object::new("MyObj")
    ///     .field(Field::new("a", TypeRef::named(TypeRef::INT), |_| {
    ///         FieldFuture::new(async move { Ok(Some(Value::from(10))) })
    ///     }))
    ///     .unresolvable("a");
    /// ```
    ///
    /// This references the `MyObj` entity with the key `a` that cannot be
    /// resolved by the current subgraph.
    pub fn unresolvable(mut self, fields: impl Into<String>) -> Self {
        self.resolvable = false;
        self.keys.push(fields.into());
        self
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
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
            MetaType::Object {
                name: self.name.clone(),
                description: self.description.clone(),
                fields,
                cache_control: Default::default(),
                extends: self.extends,
                shareable: self.shareable,
                resolvable: self.resolvable,
                keys: if !self.keys.is_empty() {
                    Some(self.keys.clone())
                } else {
                    None
                },
                visible: None,
                inaccessible: self.inaccessible,
                interface_object: self.interface_object,
                tags: self.tags.clone(),
                is_subscription: false,
                rust_typename: None,
                directive_invocations: vec![],
            },
        );

        for interface in &self.implements {
            registry.add_implements(&self.name, interface);
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn is_entity(&self) -> bool {
        !self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{dynamic::*, value, Value};

    #[tokio::test]
    async fn borrow_context() {
        struct MyObjData {
            value: i32,
        }

        let my_obj =
            Object::new("MyObj").field(Field::new("value", TypeRef::named(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    Ok(Some(Value::from(
                        ctx.parent_value.try_downcast_ref::<MyObjData>()?.value,
                    )))
                })
            }));

        let query = Object::new("Query").field(Field::new(
            "obj",
            TypeRef::named_nn(my_obj.type_name()),
            |ctx| {
                FieldFuture::new(async move {
                    Ok(Some(FieldValue::borrowed_any(
                        ctx.data_unchecked::<MyObjData>(),
                    )))
                })
            },
        ));

        let schema = Schema::build("Query", None, None)
            .register(query)
            .register(my_obj)
            .data(MyObjData { value: 123 })
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ obj { value } }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "obj": {
                    "value": 123,
                }
            })
        );
    }
}
