//! Field guards

use crate::{Context, FieldResult};
use serde::export::PhantomData;

/// Field guard
///
/// Guard is a pre-condition for a field that is resolved if `Ok(()` is returned, otherwise an error is returned.
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

/// Field post guard
///
/// Guard is a post-condition for a field that is resolved if `Ok(()` is returned, otherwise an error is returned.
#[async_trait::async_trait]
pub trait PostGuard<T: Send + Sync> {
    #[allow(missing_docs)]
    async fn check(&self, ctx: &Context<'_>, result: &T) -> FieldResult<()>;
}

/// An extension trait for `PostGuard<T>`
pub trait PostGuardExt<T: Send + Sync>: PostGuard<T> + Sized {
    /// Merge the two guards.
    fn and<R: PostGuard<T>>(self, other: R) -> PostAnd<T, Self, R> {
        PostAnd(self, other, PhantomData)
    }
}

impl<T: PostGuard<R>, R: Send + Sync> PostGuardExt<R> for T {}

/// PostGuard for `GuardExt<T>::and`
pub struct PostAnd<T: Send + Sync, A: PostGuard<T>, B: PostGuard<T>>(A, B, PhantomData<T>);

#[async_trait::async_trait]
impl<T: Send + Sync, A: PostGuard<T> + Send + Sync, B: PostGuard<T> + Send + Sync> PostGuard<T>
    for PostAnd<T, A, B>
{
    async fn check(&self, ctx: &Context<'_>, result: &T) -> FieldResult<()> {
        self.0.check(ctx, result).await?;
        self.1.check(ctx, result).await
    }
}
