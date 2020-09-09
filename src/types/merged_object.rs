use crate::parser::types::Field;
use crate::registry::{MetaType, Registry};
use crate::{
    do_resolve, CacheControl, Context, ContextSelectionSet, Error, ObjectType, OutputValueType,
    Positioned, QueryEnv, QueryError, Result, SchemaEnv, SubscriptionType, Type,
};
use futures::Stream;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::pin::Pin;

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

impl<A: Type, B: Type> Type for MergedObject<A, B> {
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

#[async_trait::async_trait]
impl<A, B> SubscriptionType for MergedObject<A, B>
where
    A: SubscriptionType + Send + Sync,
    B: SubscriptionType + Send + Sync,
{
    async fn create_field_stream(
        &self,
        idx: usize,
        ctx: &Context<'_>,
        schema_env: SchemaEnv,
        query_env: QueryEnv,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<serde_json::Value>> + Send>>> {
        match self
            .0
            .create_field_stream(idx, ctx, schema_env.clone(), query_env.clone())
            .await
        {
            Ok(value) => Ok(value),
            Err(Error::Query {
                err: QueryError::FieldNotFound { .. },
                ..
            }) => {
                self.1
                    .create_field_stream(idx, ctx, schema_env, query_env)
                    .await
            }
            Err(err) => Err(err),
        }
    }
}

#[doc(hidden)]
#[async_graphql_derive::SimpleObject(internal)]
#[derive(Default)]
pub struct MergedObjectTail;

#[doc(hidden)]
#[derive(Default)]
pub struct MergedObjectSubscriptionTail;

#[async_graphql_derive::Subscription(internal)]
impl MergedObjectSubscriptionTail {}
