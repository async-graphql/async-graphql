//! Field guards

use crate::{Context, Result};

/// Field guard
///
/// Guard is a pre-condition for a field that is resolved if `Ok(())` is returned, otherwise an error is returned.
///
/// This trait is defined through the [`async-trait`](https://crates.io/crates/async-trait) macro.
#[async_trait::async_trait]
pub trait Guard {
    /// Check whether the guard will allow access to the field.
    async fn check(&self, ctx: &Context<'_>) -> Result<()>;
}

/// An extension trait for `Guard`.
pub trait GuardExt: Guard + Sized {
    /// Perform `and` operator on two rules
    fn and<R: Guard>(self, other: R) -> And<Self, R> {
        And(self, other)
    }

    /// Perform `or` operator on two rules
    fn or<R: Guard>(self, other: R) -> Or<Self, R> {
        Or(self, other)
    }
}

impl<T: Guard> GuardExt for T {}

/// Guard for [`GuardExt::and`](trait.GuardExt.html#method.and).
pub struct And<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for And<A, B> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        self.0.check(ctx).await?;
        self.1.check(ctx).await
    }
}

/// Guard for [`GuardExt::or`](trait.GuardExt.html#method.or).
pub struct Or<A: Guard, B: Guard>(A, B);

#[async_trait::async_trait]
impl<A: Guard + Send + Sync, B: Guard + Send + Sync> Guard for Or<A, B> {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        self.0.check(ctx).await.or(self.1.check(ctx).await)
    }
}
