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
        json!({ "after": 1usize.encode_cursor().unwrap(), "before": null, "first": 2, "last": null }),
        (true, true, &["c", "d"]),
    )
    .await;

    do_test(
        json!({ "after": 1usize.encode_cursor().unwrap(), "before": null, "first": 6, "last": null }),
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
        json!({ "after": null, "before": 1usize.encode_cursor().unwrap(), "first": 10, "last": null }),
        (false, true, &["a"]),
    )
    .await;

    do_test(
        json!({ "after": null, "before": 1usize.encode_cursor().unwrap(), "first": null, "last": 10 }),
        (false, true, &["a"]),
    )
    .await;
}

#[async_std::test]
pub async fn test_datasource_additional_fields() {
    struct QueryRoot;

    struct Numbers;

    #[SimpleObject]
    struct ConnectionFields {
        total_count: i32,
    }

    #[SimpleObject]
    struct Diff {
        diff: i32,
    }

    #[DataSource]
    impl DataSource for Numbers {
        type CursorType = usize;
        type NodeType = i32;
        type ConnectionFieldsType = ConnectionFields;
        type EdgeFieldsType = Diff;

        async fn execute_query(
            &self,
            _ctx: &Context<'_>,
            after: Option<usize>,
            before: Option<usize>,
            first: Option<usize>,
            last: Option<usize>,
        ) -> FieldResult<
            Connection<
                Self::CursorType,
                Self::NodeType,
                Self::ConnectionFieldsType,
                Self::EdgeFieldsType,
            >,
        > {
            let mut start = after.map(|after| after + 1).unwrap_or(0);
            let mut end = before.unwrap_or(10000);
            if let Some(first) = first {
                end = (start + first).min(end);
            }
            if let Some(last) = last {
                start = if last > end - start { end } else { end - last };
            }
            let mut connection = Connection::with_additional_fields(
                start > 0,
                end < 10000,
                ConnectionFields { total_count: 10000 },
            );
            connection.append((start..end).into_iter().map(|n| {
                Edge::with_additional_fields(
                    n,
                    n as i32,
                    Diff {
                        diff: (10000 - n) as i32,
                    },
                )
            }));
            Ok(connection)
        }
    }

    #[Object]
    impl QueryRoot {
        async fn numbers(
            &self,
            ctx: &Context<'_>,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> FieldResult<Connection<usize, i32, ConnectionFields, Diff>> {
            Numbers.query(ctx, after, before, first, last).await
        }
    }

    let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ numbers(first: 2) { totalCount edges { node diff } } }")
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "numbers": {
                "totalCount": 10000,
                "edges": [
                    {"node": 0, "diff": 10000},
                    {"node": 1, "diff": 9999},
                ]
            },
        })
    );

    assert_eq!(
        schema
            .execute("{ numbers(last: 2) { edges { node diff } } }")
            .await
            .unwrap()
            .data,
        serde_json::json!({
            "numbers": {
                "edges": [
                    {"node": 9998, "diff": 2},
                    {"node": 9999, "diff": 1},
                ]
            },
        })
    );
}
