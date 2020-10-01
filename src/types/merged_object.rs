use crate::parser::types::Field;
use crate::registry::{MetaType, Registry};
use crate::resolver_utils::resolve_container;
use crate::{
    CacheControl, ContainerType, Context, ContextSelectionSet, ObjectType, OutputValueType,
    Positioned, ServerResult, SimpleObject, Subscription, SubscriptionType, Type,
};
use futures::Stream;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::pin::Pin;

#[doc(hidden)]
pub struct MergedObject<A, B>(pub A, pub B);

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
                cc = cc.merge(&a_cc);
            }

            B::create_type_info(registry);
            if let Some(MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            }) = registry.types.remove(&*B::type_name())
            {
                fields.extend(b_fields);
                cc = cc.merge(&b_cc);
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
impl<A, B> ContainerType for MergedObject<A, B>
where
    A: ObjectType + Send + Sync,
    B: ObjectType + Send + Sync,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<serde_json::Value>> {
        match self.0.resolve_field(ctx).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.resolve_field(ctx).await,
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
    ) -> ServerResult<serde_json::Value> {
        resolve_container(ctx, self).await
    }
}

impl<A, B> ObjectType for MergedObject<A, B>
where
    A: ObjectType + Send + Sync,
    B: ObjectType + Send + Sync,
{
}

impl<A, B> SubscriptionType for MergedObject<A, B>
where
    A: SubscriptionType + Send + Sync,
    B: SubscriptionType + Send + Sync,
{
    fn create_field_stream<'a>(
        &'a self,
        ctx: &'a Context<'a>,
    ) -> Option<Pin<Box<dyn Stream<Item = ServerResult<serde_json::Value>> + Send + 'a>>> {
        match self.0.create_field_stream(ctx) {
            Some(stream) => Some(stream),
            None => self.1.create_field_stream(ctx),
        }
    }
}

#[doc(hidden)]
#[derive(SimpleObject, Default)]
#[graphql(internal)]
pub struct MergedObjectTail;

#[doc(hidden)]
#[derive(Default)]
pub struct MergedObjectSubscriptionTail;

#[Subscription(internal)]
impl MergedObjectSubscriptionTail {}
