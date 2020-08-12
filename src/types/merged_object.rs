use crate::registry::{MetaType, Registry};
use crate::{
    do_resolve, CacheControl, Context, ContextSelectionSet, Error, ObjectType, OutputValueType,
    Positioned, QueryError, Result, Type,
};
use async_graphql_parser::query::Field;
use indexmap::IndexMap;
use std::borrow::Cow;

#[doc(hidden)]
pub struct MergedObject<A, B>(pub A, pub B);

impl<A, B> Default for MergedObject<A, B>
where
    A: Default,
    B: Default,
{
    fn default() -> Self {
        Self(A::default(), B::default())
    }
}

impl<A, B> Type for MergedObject<A, B>
where
    A: ObjectType,
    B: ObjectType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", A::type_name(), B::type_name()))
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            let mut fields = IndexMap::new();
            let mut cc = CacheControl::default();

            A::create_type_info(registry);
            if let Some(MetaType::Object {
                fields: a_fields,
                cache_control: a_cc,
                ..
            }) = registry.types.remove(&*A::type_name())
            {
                fields.extend(a_fields);
                cc.merge(&a_cc);
            }

            B::create_type_info(registry);
            if let Some(MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            }) = registry.types.remove(&*B::type_name())
            {
                fields.extend(b_fields);
                cc.merge(&b_cc);
            }

            MetaType::Object {
                name: Self::type_name().to_string(),
                description: None,
                fields,
                cache_control: cc,
                extends: false,
                keys: None,
            }
        })
    }
}

#[async_trait::async_trait]
impl<A, B> ObjectType for MergedObject<A, B>
where
    A: ObjectType + Send + Sync,
    B: ObjectType + Send + Sync,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> Result<serde_json::Value> {
        match self.0.resolve_field(ctx).await {
            Ok(value) => Ok(value),
            Err(Error::Query {
                err: QueryError::FieldNotFound { .. },
                ..
            }) => self.1.resolve_field(ctx).await,
            Err(err) => Err(err),
        }
    }
}

#[async_trait::async_trait]
impl<A, B> OutputValueType for MergedObject<A, B>
where
    A: ObjectType + Send + Sync,
    B: ObjectType + Send + Sync,
{
    async fn resolve(
        &self,
        ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> Result<serde_json::Value> {
        do_resolve(ctx, self).await
    }
}

#[doc(hidden)]
#[async_graphql_derive::SimpleObject(internal)]
#[derive(Default)]
pub struct MergedObjectTail;
