use crate::extensions::ResolveFut;
use crate::parser::types::Directive;
use crate::registry::Registry;
use crate::{Context, ContextDirective, ServerResult, Value};

#[doc(hidden)]
pub trait CustomDirectiveFactory: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn register(&self, registry: &mut Registry);

    fn create(
        &self,
        ctx: &ContextDirective<'_>,
        directive: &Directive,
    ) -> ServerResult<Box<dyn CustomDirective>>;
}

/// Represents a custom directive.
#[async_trait::async_trait]
#[allow(unused_variables)]
pub trait CustomDirective: Sync + Send + 'static {
    /// Called at resolve field.
    async fn resolve_field(
        &self,
        ctx: &Context<'_>,
        resolve: ResolveFut<'_>,
    ) -> ServerResult<Option<Value>> {
        resolve.await
    }
}
