use std::{borrow::Cow, pin::Pin};

use indexmap::IndexMap;

use crate::{
    futures_util::stream::Stream,
    parser::types::Field,
    registry::{MetaType, MetaTypeId, Registry},
    CacheControl, ContainerType, Context, ContextSelectionSet, OutputType, Positioned, Response,
    ServerResult, SimpleObject, SubscriptionType, Value,
};

#[doc(hidden)]
pub struct MergedObject<A, B>(pub A, pub B);

#[async_trait::async_trait]
impl<A, B> ContainerType for MergedObject<A, B>
where
    A: ContainerType,
    B: ContainerType,
{
    async fn resolve_field(&self, ctx: &Context<'_>) -> ServerResult<Option<Value>> {
        match self.0.resolve_field(ctx).await {
            Ok(Some(value)) => Ok(Some(value)),
            Ok(None) => self.1.resolve_field(ctx).await,
            Err(err) => Err(err),
        }
    }

    async fn find_entity(&self, ctx: &Context<'_>, params: &Value) -> ServerResult<Option<Value>> {
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
    A: OutputType,
    B: OutputType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", A::type_name(), B::type_name()))
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_output_type::<Self, _>(MetaTypeId::Object, |registry| {
            let mut fields = IndexMap::new();
            let mut cc = CacheControl::default();

            if let MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            } = registry.create_fake_output_type::<B>()
            {
                fields.extend(b_fields);
                cc = cc.merge(&b_cc);
            }

            if let MetaType::Object {
                fields: a_fields,
                cache_control: a_cc,
                ..
            } = registry.create_fake_output_type::<A>()
            {
                fields.extend(a_fields);
                cc = cc.merge(&a_cc);
            }

            MetaType::Object {
                name: Self::type_name().to_string(),
                description: None,
                fields,
                cache_control: cc,
                extends: false,
                shareable: false,
                resolvable: true,
                keys: None,
                visible: None,
                inaccessible: false,
                interface_object: false,
                tags: Default::default(),
                is_subscription: false,
                rust_typename: Some(std::any::type_name::<Self>()),
                directive_invocations: Default::default(),
            }
        })
    }

    async fn resolve(
        &self,
        _ctx: &ContextSelectionSet<'_>,
        _field: &Positioned<Field>,
    ) -> ServerResult<Value> {
        unreachable!()
    }
}

#[async_trait::async_trait]
impl<A, B> SubscriptionType for MergedObject<A, B>
where
    A: SubscriptionType,
    B: SubscriptionType,
{
    fn type_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}_{}", A::type_name(), B::type_name()))
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_subscription_type::<Self, _>(|registry| {
            let mut fields = IndexMap::new();
            let mut cc = CacheControl::default();

            if let MetaType::Object {
                fields: b_fields,
                cache_control: b_cc,
                ..
            } = registry.create_fake_subscription_type::<B>()
            {
                fields.extend(b_fields);
                cc = cc.merge(&b_cc);
            }

            if let MetaType::Object {
                fields: a_fields,
                cache_control: a_cc,
                ..
            } = registry.create_fake_subscription_type::<A>()
            {
                fields.extend(a_fields);
                cc = cc.merge(&a_cc);
            }

            MetaType::Object {
                name: Self::type_name().to_string(),
                description: None,
                fields,
                cache_control: cc,
                extends: false,
                shareable: false,
                resolvable: true,
                keys: None,
                visible: None,
                inaccessible: false,
                interface_object: false,
                tags: Default::default(),
                is_subscription: false,
                rust_typename: Some(std::any::type_name::<Self>()),
                directive_invocations: Default::default(),
            }
        })
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>> {
        unreachable!()
    }
}

#[doc(hidden)]
#[derive(SimpleObject, Default)]
#[graphql(internal, fake)]
pub struct MergedObjectTail;

impl SubscriptionType for MergedObjectTail {
    fn type_name() -> Cow<'static, str> {
        Cow::Borrowed("MergedSubscriptionTail")
    }

    fn create_type_info(registry: &mut Registry) -> String {
        registry.create_subscription_type::<Self, _>(|_| MetaType::Object {
            name: "MergedSubscriptionTail".to_string(),
            description: None,
            fields: Default::default(),
            cache_control: Default::default(),
            extends: false,
            shareable: false,
            resolvable: true,
            keys: None,
            visible: None,
            inaccessible: false,
            interface_object: false,
            tags: Default::default(),
            is_subscription: false,
            rust_typename: Some(std::any::type_name::<Self>()),
            directive_invocations: Default::default(),
        })
    }

    fn create_field_stream<'a>(
        &'a self,
        _ctx: &'a Context<'_>,
    ) -> Option<Pin<Box<dyn Stream<Item = Response> + Send + 'a>>> {
        unreachable!()
    }
}
