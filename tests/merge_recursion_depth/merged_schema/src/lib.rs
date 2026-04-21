#![recursion_limit = "64"]
//! Merges 13 cross-crate Object types into a single MergedObject.
//!
//! This reproduces the structure that triggers compiler recursion limit errors
//! when MergedObject dispatch uses nested async delegation instead of flat
//! iteration. The cross-crate boundary forces the compiler to fully resolve
//! type layouts during monomorphization, amplifying the recursion depth.

use async_graphql::*;
use merge_recursion_member_types::*;

#[rustfmt::skip] // otherwise many lines
#[derive(MergedObject, Default)]
pub struct MergedMutation(
    Top01, Top02, Top03, Top04, Top05, Top06, Top07,
    Top08, Top09, Top10, Top11, Top12, Top13,
);

pub struct Query;

#[Object]
impl Query {
    async fn mutation_obj(&self) -> MergedMutation {
        MergedMutation::default()
    }
}
