use async_graphql::connection::*;
use async_graphql::*;

#[async_std::test]
pub async fn test_connection_additional_fields() {
    struct QueryRoot;

    #[SimpleObject]
    struct ConnectionFields {
        total_count: i32,
    }

    #[SimpleObject]
    struct Diff {
        diff: i32,
    }

    #[Object]
    impl QueryRoot {
        async fn numbers(
            &self,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> FieldResult<Connection<usize, i32, ConnectionFields, Diff>> {
            connection::query(
                after,
                before,
                first,
                last,
                |after, before, first, last| async move {
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
                    connection.append((start..end).map(|n| {
                        Edge::with_additional_fields(
                            n,
                            n as i32,
                            Diff {
                                diff: (10000 - n) as i32,
                            },
                        )
                    }));
                    Ok(connection)
                },
            )
            .await
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
