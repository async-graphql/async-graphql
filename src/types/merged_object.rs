use indexmap::IndexMap;

use crate::parser::types::Field;
use crate::registry::{MetaType, Registry};
use crate::resolver_utils::resolve_container;
use crate::{
    CacheControl, ContainerType, Context, ContextSelectionSet, ObjectType, OutputType, Positioned,
    ServerResult, SimpleObject, Type, Value,
};
use async_graphql_value::ConstValue;

#[doc(hidden)]
pub struct MergedObject<A, B>(pub A, pub B);

impl<A: Type, B: Type> Type for MergedObject<A, B> {
    fn type_name() -> &'static str {
        static NAME: once_cell::sync::OnceCell<String> = once_cell::sync::OnceCell::new();
        NAME.get_or_init(|| format!("{}_{}", A::type_name(), B::type_name()))
            .as_str()
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_type::<Self, _>(|registry| {
            let mut fields = IndexMap::new();
            let mut cc = CacheControl::default();

            if let MetaType::Object {
                fields: a_fields,
                cache_control: a_cc,
                ..
            } = registry.create_dummy_type::<A>()
            {
                fields.extend(a_fields);
                cc = cc.merge(&a_cc);
            }

            if let MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            } = registry.create_dummy_type::<B>()
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
                visible: None,
            }
        })
    }
}

#[async_trait::async_trait]
impl<A, B> ContainerType for MergedObject<A, B>
where
    A: ObjectType,
    B: ObjectType,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        match self.0.resolve_field(ctx).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.resolve_field(ctx).await,
            Err(err) => Err(err),
        }
    }

    async fn find_entity(
        &self,
        ctx: &Context<'_>,
        params: &ConstValue,
    ) -> ServerResult<Option<ConstValue>> {
        match self.0.find_entity(ctx, params).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.find_entity(ctx, params).await,
            Err(err) => Err(err),
        }
    }
}

#[async_trait::async_trait]
impl<A, B> OutputType for MergedObject<A, B>
where
    A: ObjectType,
    B: ObjectType,
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
    A: ObjectType,
    B: ObjectType,
{
}

#[doc(hidden)]
#[derive(SimpleObject, Default)]
#[graphql(internal, dummy)]
pub struct MergedObjectTail;
