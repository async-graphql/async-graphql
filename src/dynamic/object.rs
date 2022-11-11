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
                    external: false,
                    requires: None,
                    provides: None,
                    visible: None,
                    shareable: false,
                    inaccessible: false,
                    tags: vec![],
                    override_from: None,
                    compute_complexity: None,
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
                extends: false,
                shareable: false,
                keys: None,
                visible: None,
                inaccessible: false,
                tags: vec![],
                is_subscription: false,
                rust_typename: None,
            },
        );

        for interface in &self.implements {
            registry.add_implements(&self.name, interface);
        }

        Ok(())
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
