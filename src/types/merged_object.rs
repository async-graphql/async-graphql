use crate::parser::types::Field;
use crate::registry::{MetaType, Registry};
use crate::resolver_utils::resolve_container;
use crate::{
    CacheControl, ContainerType, Context, ContextSelectionSet, ObjectType, OutputValueType,
    Positioned, ServerResult, SimpleObject, Type, Value,
};
use indexmap::IndexMap;
use std::borrow::Cow;

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
            }) = registry.types.get(&*A::type_name())
            {
                fields.extend(a_fields.clone());
                cc = cc.merge(&a_cc);
            }

            B::create_type_info(registry);
            if let Some(MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            }) = registry.types.get(&*B::type_name())
            {
                fields.extend(b_fields.clone());
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
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
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
    ) -> ServerResult<Value> {
        resolve_container(ctx, self).await
    }
}

impl<A, B> ObjectType for MergedObject<A, B>
where
    A: ObjectType + Send + Sync,
    B: ObjectType + Send + Sync,
{
}

#[doc(hidden)]
#[derive(SimpleObject, Default)]
#[graphql(internal)]
pub struct MergedObjectTail;
