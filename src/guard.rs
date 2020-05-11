//! Field guards

use crate::{GqlContext, GqlFieldResult};

/// Field guard
///
/// Guard is a precondition for a field that is resolved if `Ok(()` is returned, otherwise an error is returned.
#[async_trait::async_trait]
pub trait Guard {
    #[allow(missing_docs)]
    async fn check(&self, ctx: &GqlContext<'_>) -> GqlFieldResult<()>;
}

/// An extension trait for `Guard`
pub trait GuardExt: Guard + Sized {
    /// Merge the two guards.
    fn and<R: Guard>(self, other: R) -> GuardAnd<Self, R> {
        GuardAnd(self, other)
    }
}

impl<T: Guard> GuardExt for T {}

/// Guard for `GuardExt::and`
pub struct GuardAnd<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for GuardAnd<A, B> {
    async fn check(&self, ctx: &GqlContext<'_>) -> GqlFieldResult<()> {
        self.0.check(ctx).await?;
        self.1.check(ctx).await
    }
}
