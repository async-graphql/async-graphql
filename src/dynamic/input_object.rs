use indexmap::IndexMap;

use super::{directive::to_meta_directive_invocation, Directive};
use crate::{
    dynamic::InputValue,
    registry::{MetaInputValue, MetaType, Registry},
};

/// A GraphQL input object type
///
/// # Examples
///
/// ```
/// use async_graphql::{dynamic::*, value, Value};
///
/// let my_input = InputObject::new("MyInput")
///     .field(InputValue::new("a", TypeRef::named_nn(TypeRef::INT)))
///     .field(InputValue::new("b", TypeRef::named_nn(TypeRef::INT)));
///
/// let query = Object::new("Query").field(
///     Field::new("add", TypeRef::named_nn(TypeRef::INT), |ctx| {
///         FieldFuture::new(async move {
///             let input = ctx.args.try_get("input")?;
///             let input = input.object()?;
///             let a = input.try_get("a")?.i64()?;
///             let b = input.try_get("b")?.i64()?;
///             Ok(Some(Value::from(a + b)))
///         })
///     })
///     .argument(InputValue::new("input", TypeRef::named_nn(my_input.type_name())))
/// );
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async move {
///
/// let schema = Schema::build(query.type_name(), None, None)
///     .register(my_input)
///     .register(query)
///     .finish()?;
///
/// assert_eq!(
///    schema
///        .execute("{ add(input: { a: 10, b: 20 }) }")
///        .await
///        .into_result()
///        .unwrap()
///        .data,
///    value!({ "add": 30 })
/// );
///
/// # Ok::<_, SchemaError>(())
/// # }).unwrap();
/// ```
#[derive(Debug)]
pub struct InputObject {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) fields: IndexMap<String, InputValue>,
    pub(crate) oneof: bool,
    inaccessible: bool,
    tags: Vec<String>,
    directives: Vec<Directive>,
}

impl InputObject {
    /// Create a GraphQL input object type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            fields: Default::default(),
            oneof: false,
            inaccessible: false,
            tags: Vec::new(),
            directives: Vec::new(),
        }
    }

    impl_set_description!();
    impl_set_inaccessible!();
    impl_set_tags!();
    impl_directive!();

    /// Add a field
    #[inline]
    pub fn field(mut self, field: InputValue) -> Self {
        assert!(
            !self.fields.contains_key(&field.name),
            "Field `{}` already exists",
            field.name
        );
        self.fields.insert(field.name.clone(), field);
        self
    }

    /// Indicates this Input Object is a OneOf Input Object
    pub fn oneof(self) -> Self {
        Self {
            oneof: true,
            ..self
        }
    }

    /// Returns the type name
    #[inline]
    pub fn type_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn register(&self, registry: &mut Registry) -> Result<(), super::SchemaError> {
        let mut input_fields = IndexMap::new();

        for field in self.fields.values() {
            input_fields.insert(
                field.name.clone(),
                MetaInputValue {
                    name: field.name.clone(),
                    description: field.description.clone(),
                    ty: field.ty.to_string(),
                    deprecation: field.deprecation.clone(),
                    default_value: field.default_value.as_ref().map(ToString::to_string),
                    visible: None,
                    inaccessible: self.inaccessible,
                    tags: self.tags.clone(),
                    is_secret: false,
                    directive_invocations: to_meta_directive_invocation(field.directives.clone()),
                },
            );
        }

        registry.types.insert(
            self.name.clone(),
            MetaType::InputObject {
                name: self.name.clone(),
                description: self.description.clone(),
                input_fields,
                visible: None,
                inaccessible: self.inaccessible,
                tags: self.tags.clone(),
                rust_typename: None,
                oneof: self.oneof,
                directive_invocations: to_meta_directive_invocation(self.directives.clone()),
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{dynamic::*, value, Pos, ServerError, Value};

    #[tokio::test]
    async fn input_object() {
        let myinput = InputObject::new("MyInput")
            .field(InputValue::new("a", TypeRef::named_nn(TypeRef::INT)))
            .field(InputValue::new("b", TypeRef::named_nn(TypeRef::INT)));
        let query = Object::new("Query").field(
            Field::new("add", TypeRef::named_nn(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let input = ctx.args.try_get("input")?;
                    let input = input.object()?;
                    let a = input.try_get("a")?.i64()?;
                    let b = input.try_get("b")?.i64()?;
                    Ok(Some(Value::from(a + b)))
                })
            })
            .argument(InputValue::new(
                "input",
                TypeRef::named_nn(myinput.type_name()),
            )),
        );

        let schema = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(myinput)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ add(input: {a: 10, b: 20}) }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "add": 30
            })
        );
    }

    #[tokio::test]
    async fn oneof_input_object() {
        let myinput = InputObject::new("MyInput")
            .oneof()
            .field(InputValue::new("a", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("b", TypeRef::named(TypeRef::INT)));

        let query = Object::new("Query").field(
            Field::new("add10", TypeRef::named_nn(TypeRef::INT), |ctx| {
                FieldFuture::new(async move {
                    let input = ctx.args.try_get("input")?;
                    let input = input.object()?;
                    Ok(Some(Value::from(if let Some(a) = input.get("a") {
                        a.i64()? + 10
                    } else if let Some(b) = input.get("b") {
                        b.i64()? + 10
                    } else {
                        unreachable!()
                    })))
                })
            })
            .argument(InputValue::new(
                "input",
                TypeRef::named_nn(myinput.type_name()),
            )),
        );

        let schema = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(myinput)
            .finish()
            .unwrap();

        assert_eq!(
            schema
                .execute("{ add10(input: {a: 10}) }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "add10": 20
            })
        );

        assert_eq!(
            schema
                .execute("{ add10(input: {b: 20}) }")
                .await
                .into_result()
                .unwrap()
                .data,
            value!({
                "add10": 30
            })
        );

        assert_eq!(
            schema
                .execute("{ add10(input: {}) }")
                .await
                .into_result()
                .unwrap_err(),
            vec![ServerError {
                message: "Invalid value for argument \"input\", Oneof input objects requires have exactly one field".to_owned(),
                source: None,
                locations: vec![Pos { column: 9, line: 1 }],
                path: vec![],
                extensions: None,
            }]
        );

        assert_eq!(
            schema
                .execute("{ add10(input: { a: 10, b: 20 }) }")
                .await
                .into_result()
                .unwrap_err(),
            vec![ServerError {
                message: "Invalid value for argument \"input\", Oneof input objects requires have exactly one field".to_owned(),
                source: None,
                locations: vec![Pos { column: 9, line: 1 }],
                path: vec![],
                extensions: None,
            }]
        );
    }

    #[tokio::test]
    async fn invalid_oneof_input_object() {
        let myinput = InputObject::new("MyInput")
            .oneof()
            .field(InputValue::new("a", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("b", TypeRef::named_nn(TypeRef::INT)));

        let query = Object::new("Query").field(
            Field::new("value", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async move { Ok(Some(Value::from(10))) })
            })
            .argument(InputValue::new(
                "input",
                TypeRef::named_nn(myinput.type_name()),
            )),
        );

        let err = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(myinput)
            .finish()
            .unwrap_err();
        assert_eq!(err.0, "Field \"MyInput.b\" must be nullable".to_string());

        let myinput = InputObject::new("MyInput")
            .oneof()
            .field(InputValue::new("a", TypeRef::named(TypeRef::INT)))
            .field(InputValue::new("b", TypeRef::named(TypeRef::INT)).default_value(value!(10)));

        let query = Object::new("Query").field(
            Field::new("value", TypeRef::named_nn(TypeRef::INT), |_| {
                FieldFuture::new(async move { Ok(Some(Value::from(10))) })
            })
            .argument(InputValue::new(
                "input",
                TypeRef::named_nn(myinput.type_name()),
            )),
        );

        let err = Schema::build(query.type_name(), None, None)
            .register(query)
            .register(myinput)
            .finish()
            .unwrap_err();
        assert_eq!(
            err.0,
            "Field \"MyInput.b\" must not have a default value".to_string()
        );
    }
}
