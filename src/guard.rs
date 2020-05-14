//! Field guards

use crate::{Context, FieldResult};

/// Field guard
///
/// Guard is a precondition for a field that is resolved if `Ok(()` is returned, otherwise an error is returned.
#[async_trait::async_trait]
pub trait Guard {
    #[allow(missing_docs)]
    async fn check(&self, ctx: &Context<'_>) -> FieldResult<()>;
}

/// An extension trait for `Guard`
pub trait GuardExt: Guard + Sized {
    /// Merge the two guards.
    fn and<R: Guard>(self, other: R) -> And<Self, R> {
        And(self, other)
    }
}

impl<T: Guard> GuardExt for T {}

/// Guard for `GuardExt::and`
pub struct And<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for And<A, B> {
    async fn check(&self, ctx: &Context<'_>) -> FieldResult<()> {
        self.0.check(ctx).await?;
        self.1.check(ctx).await
    }
}
