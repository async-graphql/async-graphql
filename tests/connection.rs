use async_graphql::{connection::*, *};

#[tokio::test]
pub async fn test_connection_additional_fields() {
    struct Query;

    #[derive(SimpleObject)]
    struct ConnectionFields {
        total_count: i32,
    }

    #[derive(SimpleObject)]
    struct Diff {
        diff: i32,
    }

    #[Object]
    impl Query {
        async fn numbers(
            &self,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> Result<Connection<usize, i32, ConnectionFields, Diff>> {
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
                    connection.edges.extend((start..end).map(|n| {
                        Edge::with_additional_fields(
                            n,
                            n as i32,
                            Diff {
                                diff: (10000 - n) as i32,
                            },
                        )
                    }));
                    Ok::<_, Error>(connection)
                },
            )
            .await
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(
                "{ numbers(first: 2) { __typename totalCount edges { __typename node diff } } }"
            )
            .await
            .data,
        value!({
            "numbers": {
                "__typename": "IntConnection",
                "totalCount": 10000,
                "edges": [
                    {"__typename": "IntEdge", "node": 0, "diff": 10000},
                    {"__typename": "IntEdge", "node": 1, "diff": 9999},
                ]
            },
        })
    );

    assert_eq!(
        schema
            .execute("{ numbers(last: 2) { edges { node diff } } }")
            .await
            .data,
        value!({
            "numbers": {
                "edges": [
                    {"node": 9998, "diff": 2},
                    {"node": 9999, "diff": 1},
                ]
            },
        })
    );
}

#[tokio::test]
pub async fn test_connection_nodes() {
    struct Query;

    #[Object]
    impl Query {
        async fn numbers(
            &self,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> Result<Connection<usize, i32>> {
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
                    let mut connection = Connection::new(start > 0, end < 10000);
                    connection
                        .edges
                        .extend((start..end).map(|n| Edge::new(n, n as i32)));
                    Ok::<_, Error>(connection)
                },
            )
            .await
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ numbers(first: 2) { __typename edges { __typename node } nodes } }")
            .await
            .data,
        value!({
            "numbers": {
                "__typename": "IntConnection",
                "edges": [
                    {"__typename": "IntEdge", "node": 0},
                    {"__typename": "IntEdge", "node": 1},
                ],
                "nodes": [
                    0,
                    1,
                ],
            },
        })
    );

    assert_eq!(
        schema.execute("{ numbers(last: 2) { nodes } }").await.data,
        value!({
            "numbers": {
                "nodes": [
                    9998,
                    9999,
                ],
            },
        })
    );
}

#[tokio::test]
pub async fn test_connection_nodes_disabled() {
    struct Query;

    #[Object]
    impl Query {
        async fn numbers(
            &self,
            after: Option<String>,
            before: Option<String>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> Result<
            Connection<
                usize,
                i32,
                EmptyFields,
                EmptyFields,
                DefaultConnectionName,
                DefaultEdgeName,
                DisableNodesField,
            >,
        > {
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
                    let mut connection = Connection::new(start > 0, end < 10000);
                    connection
                        .edges
                        .extend((start..end).map(|n| Edge::new(n, n as i32)));
                    Ok::<_, Error>(connection)
                },
            )
            .await
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let r = schema.execute("{ numbers(last: 2) { nodes } }").await;

    assert_eq!(
        r.errors[0].message,
        "Unknown field \"nodes\" on type \"IntConnection\"."
    );
}
