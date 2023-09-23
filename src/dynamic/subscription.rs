use std::{borrow::Cow, fmt, fmt::Debug, sync::Arc};

use futures_util::{
    future::BoxFuture, stream::BoxStream, Future, FutureExt, Stream, StreamExt, TryStreamExt,
};
use indexmap::IndexMap;

use crate::{
    dynamic::{
        resolve::resolve, FieldValue, InputValue, ObjectAccessor, ResolverContext, Schema,
        SchemaError, TypeRef,
    },
    extensions::ResolveInfo,
    parser::types::Selection,
    registry::{Deprecation, MetaField, MetaType, Registry},
    subscription::BoxFieldStream,
    ContextSelectionSet, Name, QueryPathNode, QueryPathSegment, Response, Result, ServerResult,
    Value,
};

type BoxResolveFut<'a> = BoxFuture<'a, Result<BoxStream<'a, Result<FieldValue<'a>>>>>;

/// A future that returned from field resolver
pub struct SubscriptionFieldFuture<'a>(pub(crate) BoxResolveFut<'a>);

impl<'a> SubscriptionFieldFuture<'a> {
    /// Create a ResolverFuture
    pub fn new<Fut, S, T>(future: Fut) -> Self
    where
        Fut: Future<Output = Result<S>> + Send + 'a,
        S: Stream<Item = Result<T>> + Send + 'a,
        T: Into<FieldValue<'a>> + Send + 'a,
    {
        Self(
            async move {
                let res = future.await?.map_ok(Into::into);
                Ok(res.boxed())
            }
            .boxed(),
        )
    }
}

type BoxResolverFn =
    Arc<(dyn for<'a> Fn(ResolverContext<'a>) -> SubscriptionFieldFuture<'a> + Send + Sync)>;

/// A GraphQL subscription field
pub struct SubscriptionField {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) arguments: IndexMap<String, InputValue>,
    pub(crate) ty: TypeRef,
    pub(crate) resolver_fn: BoxResolverFn,
    pub(crate) deprecation: Deprecation,
}

impl SubscriptionField {
    /// Create a GraphQL subscription field
    pub fn new<N, T, F>(name: N, ty: T, resolver_fn: F) -> Self
    where
        N: Into<String>,
        T: Into<TypeRef>,
        F: for<'a> Fn(ResolverContext<'a>) -> SubscriptionFieldFuture<'a> + Send + Sync + 'static,
    {
        Self {
            name: name.into(),
            description: None,
            arguments: Default::default(),
            ty: ty.into(),
            resolver_fn: Arc::new(resolver_fn),
            deprecation: Deprecation::NoDeprecated,
        }
    }

    impl_set_description!();
    impl_set_deprecation!();

    /// Add an argument to the subscription field
    #[inline]
    pub fn argument(mut self, input_value: InputValue) -> Self {
        self.arguments.insert(input_value.name.clone(), input_value);
        self
    }
}

impl Debug for SubscriptionField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("arguments", &self.arguments)
            .field("ty", &self.ty)
            .field("deprecation", &self.deprecation)
            .finish()
    }
}

/// A GraphQL subscription type
#[derive(Debug)]
pub struct Subscription {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) fields: IndexMap<String, SubscriptionField>,
}

impl Subscription {
    /// Create a GraphQL object type
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            fields: Default::default(),
        }
    }

    impl_set_description!();

    /// Add an field to the object
    #[inline]
    pub fn field(mut self, field: SubscriptionField) -> Self {
        assert!(
            !self.fields.contains_key(&field.name),
            "Field `{}` already exists",
            field.name
        );
        self.fields.insert(field.name.clone(), field);
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
                extends: false,
                shareable: false,
                resolvable: true,
                keys: None,
                visible: None,
                inaccessible: false,
                interface_object: false,
                tags: vec![],
                is_subscription: true,
                rust_typename: None,
                directive_invocations: vec![],
            },
        );

        Ok(())
    }

    pub(crate) fn collect_streams<'a>(
        &self,
        schema: &Schema,
        ctx: &ContextSelectionSet<'a>,
        streams: &mut Vec<BoxFieldStream<'a>>,
        root_value: &'a FieldValue<'static>,
    ) {
        for selection in &ctx.item.node.items {
            if let Selection::Field(field) = &selection.node {
                if let Some(field_def) = self.fields.get(field.node.name.node.as_str()) {
                    let schema = schema.clone();
                    let field_type = field_def.ty.clone();
                    let resolver_fn = field_def.resolver_fn.clone();
                    let ctx = ctx.clone();

                    streams.push(
                        async_stream::try_stream! {
                            let ctx_field = ctx.with_field(field);
                            let field_name = ctx_field.item.node.response_key().node.clone();
                            let arguments = ObjectAccessor(Cow::Owned(
                                field
                                    .node
                                    .arguments
                                    .iter()
                                    .map(|(name, value)| {
                                        ctx_field
                                            .resolve_input_value(value.clone())
                                            .map(|value| (name.node.clone(), value))
                                    })
                                    .collect::<ServerResult<IndexMap<Name, Value>>>()?,
                            ));

                            let mut stream = resolver_fn(ResolverContext {
                                ctx: &ctx_field,
                                args: arguments,
                                parent_value: root_value,
                            })
                            .0
                            .await
                            .map_err(|err| ctx_field.set_error_path(err.into_server_error(ctx_field.item.pos)))?;

                            while let Some(value) = stream.next().await.transpose().map_err(|err| ctx_field.set_error_path(err.into_server_error(ctx_field.item.pos)))? {
                                let execute_fut = async {
                                    let ri = ResolveInfo {
                                        path_node: &QueryPathNode {
                                            parent: None,
                                            segment: QueryPathSegment::Name(&field_name),
                                        },
                                        parent_type: schema.0.env.registry.subscription_type.as_ref().unwrap(),
                                        return_type: &field_type.to_string(),
                                        name: field.node.name.node.as_str(),
                                        alias: field.node.alias.as_ref().map(|alias| alias.node.as_str()),
                                        is_for_introspection: false,
                                    };
                                    let resolve_fut = resolve(&schema, &ctx_field, &field_type, Some(&value));
                                    futures_util::pin_mut!(resolve_fut);
                                    let value = ctx_field.query_env.extensions.resolve(ri, &mut resolve_fut).await;

                                    match value {
                                        Ok(value) => {
                                            let mut map = IndexMap::new();
                                            map.insert(field_name.clone(), value.unwrap_or_default());
                                            Response::new(Value::Object(map))
                                        },
                                        Err(err) => Response::from_errors(vec![err]),
                                    }
                                };
                                futures_util::pin_mut!(execute_fut);
                                let resp = ctx_field.query_env.extensions.execute(ctx_field.query_env.operation_name.as_deref(), &mut execute_fut).await;
                                let is_err = !resp.errors.is_empty();
                                yield resp;
                                if is_err {
                                    break;
                                }
                            }
                        }.map(|res| {
                            res.unwrap_or_else(|err| Response::from_errors(vec![err]))
                        })
                        .boxed(),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures_util::StreamExt;

    use crate::{dynamic::*, value, Value};

    #[tokio::test]
    async fn subscription() {
        struct MyObjData {
            value: i32,
        }

        let my_obj = Object::new("MyObject").field(Field::new(
            "value",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async {
                    Ok(Some(Value::from(
                        ctx.parent_value.try_downcast_ref::<MyObjData>()?.value,
                    )))
                })
            },
        ));

        let query = Object::new("Query").field(Field::new(
            "value",
            TypeRef::named_nn(TypeRef::INT),
            |_| FieldFuture::new(async { Ok(FieldValue::none()) }),
        ));

        let subscription = Subscription::new("Subscription").field(SubscriptionField::new(
            "obj",
            TypeRef::named_nn(my_obj.type_name()),
            |_| {
                SubscriptionFieldFuture::new(async {
                    Ok(async_stream::try_stream! {
                        for i in 0..10 {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            yield FieldValue::owned_any(MyObjData { value: i });
                        }
                    })
                })
            },
        ));

        let schema = Schema::build(query.type_name(), None, Some(subscription.type_name()))
            .register(my_obj)
            .register(query)
            .register(subscription)
            .finish()
            .unwrap();

        let mut stream = schema.execute_stream("subscription { obj { value } }");
        for i in 0..10 {
            assert_eq!(
                stream.next().await.unwrap().into_result().unwrap().data,
                value!({
                    "obj": { "value": i }
                })
            );
        }
    }

    #[tokio::test]
    async fn borrow_context() {
        struct State {
            value: i32,
        }

        let query =
            Object::new("Query").field(Field::new("value", TypeRef::named(TypeRef::INT), |_| {
                FieldFuture::new(async { Ok(FieldValue::NONE) })
            }));

        let subscription = Subscription::new("Subscription").field(SubscriptionField::new(
            "values",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                SubscriptionFieldFuture::new(async move {
                    Ok(async_stream::try_stream! {
                        for i in 0..10 {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            yield FieldValue::value(ctx.data_unchecked::<State>().value + i);
                        }
                    })
                })
            },
        ));

        let schema = Schema::build("Query", None, Some(subscription.type_name()))
            .register(query)
            .register(subscription)
            .data(State { value: 123 })
            .finish()
            .unwrap();

        let mut stream = schema.execute_stream("subscription { values }");
        for i in 0..10 {
            assert_eq!(
                stream.next().await.unwrap().into_result().unwrap().data,
                value!({ "values": i + 123 })
            );
        }
    }
}
