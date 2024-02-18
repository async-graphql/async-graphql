#![allow(clippy::uninlined_format_args)]

use async_graphql::*;

#[tokio::test]
#[allow(unused_mut)] // FIXME: Clippy's bug
pub async fn test_mut_args() {
    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, mut a: i32, mut b: String) -> String {
            a += 1;
            b += "a";
            format!("{}{}", a, b)
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    assert_eq!(
        schema.execute("{ test(a: 10, b: \"abc\") }").await.data,
        value!({
            "test": "11abca"
        })
    );
}
