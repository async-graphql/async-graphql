use crate::extensions::{ErrorLogger, Extension, ResolveInfo};
use crate::parser::types::Selection;
use crate::registry::MetaType;
use crate::{Context, ContextSelectionSet, Error, OutputValueType, QueryError, Result, Value};
use futures::TryFutureExt;
use std::future::Future;
use std::pin::Pin;

/// A GraphQL object.
///
/// This helper trait allows the type to call `resolve_object` on itself in its
/// `OutputValueType::resolve` implementation.
#[async_trait::async_trait]
pub trait ObjectType: OutputValueType {
    /// This function returns true of type `EmptyMutation` only.
    #[doc(hidden)]
    fn is_empty() -> bool {
        false
    }

    /// Resolves a field value and outputs it as a json value `serde_json::Value`.
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value>;

    /// Collect all the fields of the object that are queried in the selection set.
    ///
    /// Objects do not have to override this, but interfaces and unions must call it on their
    /// internal type.
    fn collect_all_fields<'a>(
        &'a self,
        ctx: &ContextSelectionSet<'a>,
        fields: &mut Fields<'a>,
    ) -> Result<()>
    where
        Self: Sized + Send + Sync,
    {
        fields.add_set(ctx, self)
    }

    /// Find the GraphQL entity with the given name from the parameter.
    ///
    /// Objects should override this in case they are the query root.
    async fn find_entity(&self, ctx: &Context<'_>, _params: &Value) -> Result<serde_json::Value> {
        Err(QueryError::EntityNotFound.into_error(ctx.item.pos))
    }
}

#[async_trait::async_trait]
impl<T: ObjectType + Send + Sync> ObjectType for &T {
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        T::resolve_field(*self, ctx).await
    }
}

// TODO: reduce code duplication between the two below functions?

/// Resolve an object by executing each of the fields concurrently.
pub async fn resolve_object<'a, T: ObjectType + Send + Sync>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root)?;
    let futures = fields.0;

    let res = futures::future::try_join_all(futures).await?;
    let mut map = serde_json::Map::new();
    for (name, value) in res {
        if let serde_json::Value::Object(b) = value {
            if let Some(serde_json::Value::Object(a)) = map.get_mut(&name) {
                a.extend(b);
            } else {
                map.insert(name, b.into());
            }
        } else {
            map.insert(name, value);
        }
    }
    Ok(map.into())
}

/// Resolve an object by executing each of the fields serially.
pub async fn resolve_object_serial<'a, T: ObjectType + Send + Sync>(
    ctx: &ContextSelectionSet<'a>,
    root: &'a T,
) -> Result<serde_json::Value> {
    let mut fields = Fields(Vec::new());
    fields.add_set(ctx, root)?;
    let futures = fields.0;

    let mut map = serde_json::Map::new();
    for field in futures {
        let (name, value) = field.await?;

        if let serde_json::Value::Object(b) = value {
            if let Some(serde_json::Value::Object(a)) = map.get_mut(&name) {
                a.extend(b);
            } else {
                map.insert(name, b.into());
            }
        } else {
            map.insert(name, value);
        }
    }
    Ok(map.into())
}

type BoxFieldFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(String, serde_json::Value)>> + 'a + Send>>;

/// A set of fields on an object that are being selected.
pub struct Fields<'a>(Vec<BoxFieldFuture<'a>>);

impl<'a> Fields<'a> {
    /// Add another set of fields to this set of fields using the given object.
    pub fn add_set<T: ObjectType + Send + Sync>(
        &mut self,
        ctx: &ContextSelectionSet<'a>,
        root: &'a T,
    ) -> Result<()> {
        for selection in &ctx.item.node.items {
            if ctx.is_skip(&selection.node.directives())? {
                continue;
            }

            match &selection.node {
                Selection::Field(field) => {
                    if field.node.name.node == "__typename" {
                        // Get the typename
                        let ctx_field = ctx.with_field(field);
                        let field_name = ctx_field
                            .item
                            .node
                            .response_key()
                            .node
                            .clone()
                            .into_string();
                        let typename = root.introspection_type_name().into_owned();

                        self.0.push(Box::pin(async move {
                            Ok((field_name, serde_json::Value::String(typename)))
                        }));
                        continue;
                    }

                    if ctx.is_ifdef(&field.node.directives) {
                        if let Some(MetaType::Object { fields, .. }) =
                            ctx.schema_env.registry.types.get(T::type_name().as_ref())
                        {
                            if !fields.contains_key(field.node.name.node.as_str()) {
                                continue;
                            }
                        }
                    }

                    self.0.push(Box::pin({
                        // TODO: investigate removing this
                        let ctx = ctx.clone();
                        async move {
                            let ctx_field = ctx.with_field(field);
                            let field_name = ctx_field
                                .item
                                .node
                                .response_key()
                                .node
                                .clone()
                                .into_string();

                            let resolve_info = ResolveInfo {
                                resolve_id: ctx_field.resolve_id,
                                path_node: ctx_field.path_node.as_ref().unwrap(),
                                parent_type: &T::type_name(),
                                return_type: match ctx_field
                                    .schema_env
                                    .registry
                                    .types
                                    .get(T::type_name().as_ref())
                                    .and_then(|ty| ty.field_by_name(field.node.name.node.as_str()))
                                    .map(|field| &field.ty)
                                {
                                    Some(ty) => &ty,
                                    None => {
                                        return Err(Error::Query {
                                            pos: field.pos,
                                            path: None,
                                            err: QueryError::FieldNotFound {
                                                field_name: field
                                                    .node
                                                    .name
                                                    .node
                                                    .clone()
                                                    .into_string(),
                                                object: T::type_name().to_string(),
                                            },
                                        })
                                    }
                                },
                                schema_env: ctx.schema_env,
                                query_env: ctx.query_env,
                            };

                            ctx_field
                                .query_env
                                .extensions
                                .lock()
                                .resolve_start(&resolve_info);

                            let res = root
                                .resolve_field(&ctx_field)
                                .map_ok(move |value| (field_name, value))
                                .await
                                .log_error(&ctx_field.query_env.extensions)?;

                            ctx_field
                                .query_env
                                .extensions
                                .lock()
                                .resolve_end(&resolve_info);
                            Ok(res)
                        }
                    }));
                }
                selection => {
                    let (type_condition, selection_set) = match selection {
                        Selection::Field(_) => unreachable!(),
                        Selection::FragmentSpread(spread) => {
                            let fragment =
                                ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                            let fragment = match fragment {
                                Some(fragment) => fragment,
                                None => {
                                    return Err(Error::Query {
                                        pos: spread.pos,
                                        path: None,
                                        err: QueryError::UnknownFragment {
                                            name: spread.node.fragment_name.to_string(),
                                        },
                                    })
                                }
                            };
                            (
                                Some(&fragment.node.type_condition),
                                &fragment.node.selection_set,
                            )
                        }
                        Selection::InlineFragment(fragment) => (
                            fragment.node.type_condition.as_ref(),
                            &fragment.node.selection_set,
                        ),
                    };
                    let type_condition =
                        type_condition.map(|condition| condition.node.on.node.as_str());

                    let introspection_type_name = root.introspection_type_name();

                    let applies_concrete_object = type_condition.map_or(false, |condition| {
                        introspection_type_name == condition
                            || ctx
                                .schema_env
                                .registry
                                .implements
                                .get(&*introspection_type_name)
                                .map_or(false, |interfaces| interfaces.contains(condition))
                    });
                    if applies_concrete_object {
                        // The fragment applies to the concrete object type.

                        // TODO: This solution isn't ideal. If there are two interfaces InterfaceA
                        // and InterfaceB and one type MyObj that implements both, then if you have
                        // a type condition for `InterfaceA` on an `InterfaceB` and when resolving,
                        // the `InterfaceB` is actually a `MyObj` then the contents of the fragment
                        // will be treated as a `MyObj` rather than an `InterfaceB`. Example:
                        //
                        // myObjAsInterfaceB {
                        //     ... on InterfaceA {
                        //         # here you can query MyObj fields even when you should only be
                        //         # able to query InterfaceA fields.
                        //     }
                        // }
                        root.collect_all_fields(&ctx.with_selection_set(selection_set), self)?;
                    } else if type_condition.map_or(true, |condition| T::type_name() == condition) {
                        // The fragment applies to an interface type.
                        self.add_set(&ctx.with_selection_set(selection_set), root)?;
                    }
                }
            }
        }
        Ok(())
    }
}
