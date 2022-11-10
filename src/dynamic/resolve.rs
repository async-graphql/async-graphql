use std::{borrow::Cow, pin::Pin};

use futures_util::{future::BoxFuture, Future, FutureExt};
use indexmap::IndexMap;

use crate::{
    dynamic::{
        type_ref::TypeRefInner, FieldValue, Object, ObjectAccessor, ResolverContext, Schema, Type,
    },
    extensions::ResolveInfo,
    parser::types::Selection,
    resolver_utils::create_value_object,
    Context, ContextSelectionSet, Error, IntrospectionMode, Name, ServerError, ServerResult, Value,
};

type BoxFieldFuture<'a> = Pin<Box<dyn Future<Output = ServerResult<(Name, Value)>> + 'a + Send>>;

pub(crate) async fn resolve_container(
    schema: &Schema,
    object: &Object,
    ctx: &ContextSelectionSet<'_>,
    parent_value: &FieldValue<'_>,
    serial: bool,
) -> ServerResult<Option<Value>> {
    let mut fields = Vec::new();
    collect_fields(&mut fields, schema, object, ctx, parent_value)?;

    let res = if serial {
        futures_util::future::try_join_all(fields).await?
    } else {
        let mut results = Vec::with_capacity(fields.len());
        for field in fields {
            results.push(field.await?);
        }
        results
    };

    Ok(Some(create_value_object(res)))
}

fn collect_fields<'a>(
    fields: &mut Vec<BoxFieldFuture<'a>>,
    schema: &'a Schema,
    object: &'a Object,
    ctx: &ContextSelectionSet<'a>,
    parent_value: &'a FieldValue,
) -> ServerResult<()> {
    for selection in &ctx.item.node.items {
        match &selection.node {
            Selection::Field(field) => {
                if field.node.name.node == "__typename" {
                    if matches!(
                        ctx.schema_env.registry.introspection_mode,
                        IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly
                    ) && matches!(
                        ctx.query_env.introspection_mode,
                        IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly,
                    ) {
                        fields.push(
                            async move {
                                Ok((
                                    field.node.response_key().node.clone(),
                                    Value::from(object.name.as_str()),
                                ))
                            }
                            .boxed(),
                        )
                    } else {
                        fields.push(
                            async move { Ok((field.node.response_key().node.clone(), Value::Null)) }.boxed(),
                        )
                    }
                    continue;
                }

                if object.name == schema.0.env.registry.query_type
                    && matches!(
                        ctx.schema_env.registry.introspection_mode,
                        IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly
                    )
                    && matches!(
                        ctx.query_env.introspection_mode,
                        IntrospectionMode::Enabled | IntrospectionMode::IntrospectionOnly,
                    )
                {
                    // is query root
                    if field.node.name.node == "__schema" {
                        let ctx = ctx.clone();
                        fields.push(
                            async move {
                                let ctx_field = ctx.with_field(field);
                                let mut ctx_obj =
                                    ctx.with_selection_set(&ctx_field.item.node.selection_set);
                                ctx_obj.is_for_introspection = true;
                                let visible_types =
                                    ctx.schema_env.registry.find_visible_types(&ctx_field);
                                let value = crate::OutputType::resolve(
                                    &crate::model::__Schema::new(
                                        &ctx.schema_env.registry,
                                        &visible_types,
                                    ),
                                    &ctx_obj,
                                    ctx_field.item,
                                )
                                .await?;
                                Ok((field.node.response_key().node.clone(), value))
                            }
                            .boxed(),
                        );
                        continue;
                    } else if field.node.name.node == "__type" {
                        let ctx = ctx.clone();
                        fields.push(
                            async move {
                                let ctx_field = ctx.with_field(field);
                                let (_, type_name) =
                                    ctx_field.param_value::<String>("name", None)?;
                                let mut ctx_obj =
                                    ctx.with_selection_set(&ctx_field.item.node.selection_set);
                                ctx_obj.is_for_introspection = true;
                                let visible_types =
                                    ctx.schema_env.registry.find_visible_types(&ctx_field);
                                let value = crate::OutputType::resolve(
                                    &ctx.schema_env
                                        .registry
                                        .types
                                        .get(&type_name)
                                        .filter(|_| visible_types.contains(type_name.as_str()))
                                        .map(|ty| {
                                            crate::model::__Type::new_simple(
                                                &ctx.schema_env.registry,
                                                &visible_types,
                                                ty,
                                            )
                                        }),
                                    &ctx_obj,
                                    ctx_field.item,
                                )
                                .await?;
                                Ok((field.node.response_key().node.clone(), value))
                            }
                            .boxed(),
                        );
                        continue;
                    }
                }

                if ctx.schema_env.registry.introspection_mode
                    == IntrospectionMode::IntrospectionOnly
                    || ctx.query_env.introspection_mode == IntrospectionMode::IntrospectionOnly
                {
                    fields.push(
                        async move { Ok((field.node.response_key().node.clone(), Value::Null)) }
                            .boxed(),
                    );
                    continue;
                }

                if let Some(field_def) = object.fields.get(field.node.name.node.as_str()) {
                    let ctx = ctx.clone();
                    fields.push(
                        async move {
                            let ctx_field = ctx.with_field(field);
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

                            let resolve_info = ResolveInfo {
                                path_node: ctx_field.path_node.as_ref().unwrap(),
                                parent_type: &object.name,
                                return_type: &field_def.ty.to_string(),
                                name: &field.node.name.node,
                                alias: field.node.alias.as_ref().map(|alias| &*alias.node),
                                is_for_introspection: ctx_field.is_for_introspection,
                            };

                            let resolve_fut = async {
                                let field_value = (field_def.resolver_fn)(ResolverContext {
                                    ctx: &ctx_field,
                                    args: arguments,
                                    parent_value,
                                })
                                .0
                                .await
                                .map_err(|err| err.into_server_error(field.pos))?;
                                let value = resolve(
                                    schema,
                                    &ctx_field,
                                    &field_def.ty.0,
                                    field_value.as_ref(),
                                )
                                .await?;
                                Ok(value)
                            };
                            futures_util::pin_mut!(resolve_fut);

                            let res_value = ctx_field
                                .query_env
                                .extensions
                                .resolve(resolve_info, &mut resolve_fut)
                                .await?
                                .unwrap_or_default();
                            Ok((field.node.response_key().node.clone(), res_value))
                        }
                        .boxed(),
                    );
                }
            }
            selection => {
                let (type_condition, selection_set) = match selection {
                    Selection::Field(_) => unreachable!(),
                    Selection::FragmentSpread(spread) => {
                        let fragment = ctx.query_env.fragments.get(&spread.node.fragment_name.node);
                        let fragment = match fragment {
                            Some(fragment) => fragment,
                            None => {
                                return Err(ServerError::new(
                                    format!(
                                        "Unknown fragment \"{}\".",
                                        spread.node.fragment_name.node
                                    ),
                                    Some(spread.pos),
                                ));
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
                let introspection_type_name = &object.name;

                if type_condition.is_none() || type_condition == Some(introspection_type_name) {
                    collect_fields(
                        fields,
                        schema,
                        object,
                        &ctx.with_selection_set(selection_set),
                        parent_value,
                    )?;
                }
            }
        }
    }

    Ok(())
}

pub(crate) fn resolve<'a>(
    schema: &'a Schema,
    ctx: &'a Context<'a>,
    type_ref: &'a TypeRefInner,
    value: Option<&'a FieldValue>,
) -> BoxFuture<'a, ServerResult<Option<Value>>> {
    async move {
        match (type_ref, value) {
            (TypeRefInner::Named(type_name), Some(value)) => {
                resolve_value(schema, ctx, &schema.0.types[type_name.as_ref()], value).await
            }
            (TypeRefInner::Named(_), None) => Ok(None),

            (TypeRefInner::NonNull(type_ref), Some(value)) => {
                resolve(schema, ctx, type_ref, Some(value)).await
            }
            (TypeRefInner::NonNull(_), None) => Err(ctx.set_error_path(
                Error::new("internal: non-null types require a return value")
                    .into_server_error(ctx.item.pos),
            )),

            (TypeRefInner::List(type_def), Some(FieldValue::List(values))) => {
                let mut futures = Vec::with_capacity(values.len());
                for (idx, value) in values.iter().enumerate() {
                    let ctx_item = ctx.with_index(idx);

                    futures.push(async move {
                        let parent_type = format!("[{}]", type_def);
                        let return_type = type_def.to_string();
                        let resolve_info = ResolveInfo {
                            path_node: ctx_item.path_node.as_ref().unwrap(),
                            parent_type: &parent_type,
                            return_type: &return_type,
                            name: ctx.item.node.name.node.as_str(),
                            alias: ctx
                                .item
                                .node
                                .alias
                                .as_ref()
                                .map(|alias| alias.node.as_str()),
                            is_for_introspection: ctx_item.is_for_introspection,
                        };

                        let resolve_fut =
                            async { resolve(schema, &ctx_item, type_def, Some(value)).await };
                        futures_util::pin_mut!(resolve_fut);

                        let res_value = ctx_item
                            .query_env
                            .extensions
                            .resolve(resolve_info, &mut resolve_fut)
                            .await?;
                        Ok::<_, ServerError>(res_value.unwrap_or_default())
                    });
                }
                let values = futures_util::future::try_join_all(futures).await?;
                Ok(Some(Value::List(values)))
            }
            (TypeRefInner::List(_), Some(_)) => Err(ctx.set_error_path(
                Error::new("internal: expects an array").into_server_error(ctx.item.pos),
            )),
            (TypeRefInner::List(_), None) => Ok(Some(Value::List(vec![]))),
        }
    }
    .boxed()
}

async fn resolve_value(
    schema: &Schema,
    ctx: &Context<'_>,
    field_type: &Type,
    value: &FieldValue<'_>,
) -> ServerResult<Option<Value>> {
    match (field_type, value) {
        (Type::Scalar(_), FieldValue::Value(value)) => Ok(Some(value.clone())),
        (Type::Scalar(scalar), _) => Err(ctx.set_error_path(
            Error::new(format!(
                "internal: invalid value for scalar \"{}\", expected \"FieldValue::Value\"",
                scalar.name
            ))
            .into_server_error(ctx.item.pos),
        )),

        (Type::Object(object), value) => {
            resolve_container(
                schema,
                object,
                &ctx.with_selection_set(&ctx.item.node.selection_set),
                value,
                true,
            )
            .await
        }

        (Type::InputObject(obj), _) => Err(ctx.set_error_path(
            Error::new(format!(
                "internal: cannot use input object \"{}\" as output value",
                obj.name
            ))
            .into_server_error(ctx.item.pos),
        )),

        (Type::Enum(e), FieldValue::Value(Value::Enum(name))) => {
            if !e.enum_values.contains_key(name.as_str()) {
                return Err(ctx.set_error_path(
                    Error::new(format!("internal: invalid item for enum \"{}\"", e.name))
                        .into_server_error(ctx.item.pos),
                ));
            }
            Ok(Some(Value::Enum(name.clone())))
        }
        (Type::Enum(e), FieldValue::Value(Value::String(name))) => {
            if !e.enum_values.contains_key(name) {
                return Err(ctx.set_error_path(
                    Error::new(format!("internal: invalid item for enum \"{}\"", e.name))
                        .into_server_error(ctx.item.pos),
                ));
            }
            Ok(Some(Value::Enum(Name::new(name))))
        }
        (Type::Enum(e), _) => Err(ctx.set_error_path(
            Error::new(format!("internal: invalid item for enum \"{}\"", e.name))
                .into_server_error(ctx.item.pos),
        )),

        (Type::Interface(interface), FieldValue::WithType { value, ty }) => {
            let is_contains_obj = schema
                .0
                .env
                .registry
                .types
                .get(&interface.name)
                .and_then(|meta_type| {
                    meta_type
                        .possible_types()
                        .map(|possible_types| possible_types.contains(ty.as_ref()))
                })
                .unwrap_or_default();
            if !is_contains_obj {
                return Err(ctx.set_error_path(
                    Error::new(format!(
                        "internal: object \"{}\" does not implement interface \"{}\"",
                        ty, interface.name,
                    ))
                    .into_server_error(ctx.item.pos),
                ));
            }

            let object_type = schema
                .0
                .types
                .get(ty.as_ref())
                .ok_or_else(|| {
                    ctx.set_error_path(
                        Error::new(format!("internal: object \"{}\" does not registered", ty))
                            .into_server_error(ctx.item.pos),
                    )
                })?
                .as_object()
                .ok_or_else(|| {
                    ctx.set_error_path(
                        Error::new(format!("internal: type \"{}\" is not object", ty))
                            .into_server_error(ctx.item.pos),
                    )
                })?;

            resolve_container(
                schema,
                object_type,
                &ctx.with_selection_set(&ctx.item.node.selection_set),
                value,
                true,
            )
            .await
        }
        (Type::Interface(interface), _) => Err(ctx.set_error_path(
            Error::new(format!(
                "internal: invalid value for interface \"{}\", expected \"FieldValue::WithType\"",
                interface.name
            ))
            .into_server_error(ctx.item.pos),
        )),

        (Type::Union(union), FieldValue::WithType { value, ty }) => {
            if !union.possible_types.contains(ty.as_ref()) {
                return Err(ctx.set_error_path(
                    Error::new(format!(
                        "internal: union \"{}\" does not contain object \"{}\"",
                        union.name, ty,
                    ))
                    .into_server_error(ctx.item.pos),
                ));
            }

            let object_type = schema
                .0
                .types
                .get(ty.as_ref())
                .ok_or_else(|| {
                    ctx.set_error_path(
                        Error::new(format!("internal: object \"{}\" does not registered", ty))
                            .into_server_error(ctx.item.pos),
                    )
                })?
                .as_object()
                .ok_or_else(|| {
                    ctx.set_error_path(
                        Error::new(format!("internal: type \"{}\" is not object", ty))
                            .into_server_error(ctx.item.pos),
                    )
                })?;

            resolve_container(
                schema,
                object_type,
                &ctx.with_selection_set(&ctx.item.node.selection_set),
                value,
                true,
            )
            .await
        }
        (Type::Union(union), _) => Err(ctx.set_error_path(
            Error::new(format!(
                "internal: invalid value for union \"{}\", expected \"FieldValue::WithType\"",
                union.name
            ))
            .into_server_error(ctx.item.pos),
        )),

        (Type::Subscription(subscription), _) => Err(ctx.set_error_path(
            Error::new(format!(
                "internal: cannot use subscription \"{}\" as output value",
                subscription.name
            ))
            .into_server_error(ctx.item.pos),
        )),
    }
}
