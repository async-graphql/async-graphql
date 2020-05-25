use async_graphql::connection::*;
use async_graphql::*;
use serde_json::json;

#[async_std::test]
pub async fn test_slice_datasource() {
    struct Query;

    #[Object]
    impl Query {
        async fn values(
            &self,
            ctx: &Context<'_>,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> FieldResult<Connection<usize, &&str>> {
            const ROWS: &[&str] = &[
                "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p",
                "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            ];
            ROWS.query(ctx, after, before, first, last).await
        }
    }

    static QUERY: &str = r#"
        query(
            $after: String
            $before: String
            $first: Int
            $last: Int
        ) {
            values(
                after: $after
                before: $before
                first: $first
                last: $last
            ) {
                totalCount
                pageInfo {
                    hasPreviousPage
                    hasNextPage
                }
                edges {
                    node
                }
            }
        }
    "#;

    async fn do_test(
        vars: serde_json::Value,
        (has_prev_page, has_next_page, nodes): (bool, bool, &[&str]),
    ) {
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
        let query = QueryBuilder::new(QUERY).variables(Variables::parse_from_json(vars).unwrap());
        let edges = nodes
            .iter()
            .map(|s| json!({ "node": s.to_owned() }))
            .collect::<Vec<_>>();

        assert_eq!(
            query.execute(&schema).await.unwrap().data,
            serde_json::json!({
                "values": {
                    "totalCount": 26,
                    "pageInfo": {
                        "hasPreviousPage": has_prev_page,
                        "hasNextPage": has_next_page,
                    },
                    "edges": edges,
                },
            })
        );
    }

    do_test(
        json!({ "after": null, "before": null, "first": 2, "last": null }),
        (false, true, &["a", "b"]),
    )
    .await;

    do_test(
        json!({ "after": 1usize.encode_cursor(), "before": null, "first": 2, "last": null }),
        (true, true, &["c", "d"]),
    )
    .await;

    do_test(
        json!({ "after": 1usize.encode_cursor(), "before": null, "first": 6, "last": null }),
        (true, true, &["c", "d", "e", "f", "g", "h"]),
    )
    .await;

    do_test(
        json!({ "after": null, "before": null, "first": 3, "last": null }),
        (false, true, &["a", "b", "c"]),
    )
    .await;

    do_test(
        json!({ "after": null, "before": null, "first": null, "last": 3 }),
        (true, false, &["x", "y", "z"]),
    )
    .await;

    do_test(
        json!({ "after": null, "before": 1usize.encode_cursor(), "first": 10, "last": null }),
        (false, true, &["a"]),
    )
    .await;

    do_test(
        json!({ "after": null, "before": 1usize.encode_cursor(), "first": null, "last": 10 }),
        (false, true, &["a"]),
    )
    .await;
}
