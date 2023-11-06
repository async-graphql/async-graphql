use std::borrow::Cow;

use crate::{
    extensions::ResolveFut, parser::types::Directive, registry::Registry, Context,
    ContextDirective, ServerResult, Value,
};

#[doc(hidden)]
pub trait CustomDirectiveFactory: 'static {
    fn name(&self) -> Cow<'static, str>;

    fn register(&self, registry: &mut Registry);

    fn create(
        &self,
        ctx: &ContextDirective<'_>,
        directive: &Directive,
    ) -> ServerResult<Box<dyn CustomDirective>>;
}

#[doc(hidden)]
// minimal amount required to register directive into registry
pub trait TypeDirective {
    fn name(&self) -> Cow<'static, str>;

    fn register(&self, registry: &mut Registry);
}

/// Represents a custom directive.
#[async_trait::async_trait(?Send)]
#[allow(unused_variables)]
pub trait CustomDirective: 'static {
    /// Called at resolve field.
    async fn resolve_field(
        &self,
        ctx: &Context<'_>,
        resolve: ResolveFut<'_>,
    ) -> ServerResult<Option<Value>> {
        resolve.await
    }
}
