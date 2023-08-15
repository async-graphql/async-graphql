use indexmap::IndexMap;

use crate::{
    dynamic::SchemaError,
    registry::{Deprecation, MetaEnumValue, MetaType, Registry},
};

/// A GraphQL enum item
#[derive(Debug)]
pub struct EnumItem {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) deprecation: Deprecation,
    inaccessible: bool,
    tags: Vec<String>,
}

impl<T: Into<String>> From<T> for EnumItem {
    #[inline]
    fn from(name: T) -> Self {
        EnumItem {
            name: name.into(),
            description: None,
            deprecation: Deprecation::NoDeprecated,
            inaccessible: false,
            tags: Vec::new(),
        }
    }
}

impl EnumItem {
    /// Create a new EnumItem
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        name.into().into()
    }

    impl_set_description!();
    impl_set_deprecation!();
    impl_set_inaccessible!();
    impl_set_tags!();
}

/// A GraphQL enum type
#[derive(Debug)]
pub struct Enum {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) enum_values: IndexMap<String, EnumItem>,
    inaccessible: bool,
    tags: Vec<String>,
}

impl Enum {
    /// Create a GraphqL enum type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            enum_values: Default::default(),
            inaccessible: false,
            tags: Vec::new(),
        }
    }

    impl_set_description!();

    /// Add an item
    #[inline]
    pub fn item(mut self, item: impl Into<EnumItem>) -> Self {
        let item = item.into();
        self.enum_values.insert(item.name.clone(), item);
        self
    }

    /// Add items
    pub fn items(mut self, items: impl IntoIterator<Item = impl Into<EnumItem>>) -> Self {
        for item in items {
            let item = item.into();
            self.enum_values.insert(item.name.clone(), item);
        }
        self
    }

    impl_set_inaccessible!();
    impl_set_tags!();

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), SchemaError> {
        let mut enum_values = IndexMap::new();

        for item in self.enum_values.values() {
            enum_values.insert(
                item.name.clone(),
                MetaEnumValue {
                    name: item.name.as_str().into(),
                    description: item.description.clone(),
                    deprecation: item.deprecation.clone(),
                    visible: None,
                    inaccessible: item.inaccessible,
                    tags: item.tags.clone(),
                },
            );
        }

        registry.types.insert(
            self.name.clone(),
            MetaType::Enum {
                name: self.name.clone(),
                description: self.description.clone(),
                enum_values,
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
    use crate::{dynamic::*, value, Name, PathSegment, Pos, ServerError, Value};

    #[tokio::test]
    async fn enum_type() {
        let my_enum = Enum::new("MyEnum").item("A").item("B");

        let query = Object::new("Query")
            .field(Field::new(
                "value",
                TypeRef::named_nn(my_enum.type_name()),
                |_| FieldFuture::new(async { Ok(Some(Value::from(Name::new("A")))) }),
            ))
            .field(
                Field::new("value2", TypeRef::named_nn(my_enum.type_name()), |ctx| {
                    FieldFuture::new(async move {
                        Ok(Some(FieldValue::value(Name::new(
                            ctx.args.try_get("input")?.enum_name()?,
                        ))))
                    })
                })
                .argument(InputValue::new(
                    "input",
                    TypeRef::named_nn(my_enum.type_name()),
                )),
            )
            .field(Field::new(
                "errValue",
                TypeRef::named_nn(my_enum.type_name()),
                |_| FieldFuture::new(async { Ok(Some(Value::from(Name::new("C")))) }),
            ));
        let schema = Schema::build("Query", None, None)
            .register(my_enum)
            .register(query)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ value value2(input: B) }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "value": "A",
                "value2": "B"
            })
        );

        assert_eq!(
            schema
                .execute("{ errValue }")
                .await
                .into_result()
                .unwrap_err(),
            vec![ServerError {
                message: "internal: invalid item for enum \"MyEnum\"".to_owned(),
                source: None,
                locations: vec![Pos { column: 3, line: 1 }],
                path: vec![PathSegment::Field("errValue".to_owned())],
                extensions: None,
            }]
        );
    }
}
