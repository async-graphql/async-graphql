//! Integration test: validates that MergedObject with cross-crate types
//! compiles and resolves correctly.
//!
//! This test exists because deeply-nested MergedObject async dispatch can
//! overflow the compiler's recursion limit when types come from external
//! crates. See: https://github.com/async-graphql/async-graphql/issues/1647

use async_graphql::*;
use merge_recursion_merged_schema::Query;

#[tokio::test]
async fn cross_crate_merged_object_resolves() {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    // Query the first and last merged members to verify dispatch works across all
    // positions
    let query = r#"{ mutationObj { top01 { mid01 { lf01 } } top13 { mid13 { lf13 } } } }"#;
    let res = schema.execute(query).await;
    let data = res.into_result().unwrap().data;

    assert_eq!(
        data,
        value!({
            "mutationObj": {
                "top01": { "mid01": { "lf01": 0 } },
                "top13": { "mid13": { "lf13": 0 } },
            }
        })
    );
}
