pub mod stream {
    use async_graphql::*;
    use futures::StreamExt;
    use serde_json::json;

    struct Root;

    #[Object]
    impl Root {
        async fn stream_connection(
            &self,
            ctx: &Context<'_>,
            after: Option<Cursor>,
            before: Option<Cursor>,
            first: Option<i32>,
            last: Option<i32>,
        ) -> FieldResult<Connection<String>> {
            futures::stream::iter(vec![
                "a".to_owned(),
                "b".to_owned(),
                "c".to_owned(),
                "d".to_owned(),
                "e".to_owned(),
                "f".to_owned(),
            ])
            .map(|node| {
                let cursor: Cursor = node.clone().into();
                (cursor, EmptyEdgeFields, node)
            })
            .boxed()
            .query(ctx, after, before, first, last)
            .await
        }
    }

    static QUERY: &str = r#"
        query(
            $after: Cursor
            $before: Cursor
            $first: Int
            $last: Int
        ) {
            streamConnection(
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
                    cursor
                    node
                }
            }
        }
    "#;

    async fn do_test(
        vars: serde_json::Value,
        (has_prev_page, has_next_page, edges): (bool, bool, &[&str]),
    ) {
        let schema = Schema::new(Root, EmptyMutation, EmptySubscription);

        let query = QueryBuilder::new(QUERY).variables(Variables::parse_from_json(vars).unwrap());

        let edges = edges
            .iter()
            .map(|s| json!({ "cursor": s.to_owned(), "node": s.to_owned() }))
            .collect::<Vec<_>>();

        assert_eq!(
            query.execute(&schema).await.unwrap().data,
            serde_json::json!({
                "streamConnection": {
                    "totalCount": 6,
                    "pageInfo": {
                        "hasPreviousPage": has_prev_page,
                        "hasNextPage": has_next_page,
                    },
                    "edges": edges,
                },
            })
        );
    }

    #[async_std::test]
    pub async fn test_connection_stream_none() {
        do_test(
            json!({ "after": null, "before": null, "first": null, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_after() {
        do_test(
            json!({ "after": "b", "before": null, "first": null, "last": null }),
            (true, false, &["c", "d", "e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": "x", "before": null, "first": null, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_before() {
        do_test(
            json!({ "after": null, "before": "e", "first": null, "last": null }),
            (false, true, &["a", "b", "c", "d"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": "x", "first": null, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_between() {
        do_test(
            json!({ "after": "b", "before": "e", "first": null, "last": null }),
            (true, true, &["c", "d"]),
        )
        .await;

        do_test(
            json!({ "after": "x", "before": "e", "first": null, "last": null }),
            (false, true, &["a", "b", "c", "d"]),
        )
        .await;

        do_test(
            json!({ "after": "b", "before": "x", "first": null, "last": null }),
            (true, false, &["c", "d", "e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": "x", "before": "x", "first": null, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_first() {
        do_test(
            json!({ "after": null, "before": null, "first": 2, "last": null }),
            (false, true, &["a", "b"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": null, "first": 6, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": null, "first": 10, "last": null }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_first_after() {
        do_test(
            json!({ "after": "b", "before": null, "first": 2, "last": null }),
            (true, true, &["c", "d"]),
        )
        .await;

        do_test(
            json!({ "after": "b", "before": null, "first": 6, "last": null }),
            (true, false, &["c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_first_before() {
        do_test(
            json!({ "after": null, "before": "e", "first": 2, "last": null }),
            (false, true, &["a", "b"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": "e", "first": 6, "last": null }),
            (false, true, &["a", "b", "c", "d"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_first_between() {
        do_test(
            json!({ "after": "a", "before": "f", "first": 2, "last": null }),
            (true, true, &["b", "c"]),
        )
        .await;

        do_test(
            json!({ "after": "a", "before": "f", "first": 6, "last": null }),
            (true, true, &["b", "c", "d", "e"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_last() {
        do_test(
            json!({ "after": null, "before": null, "first": null, "last": 2 }),
            (true, false, &["e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": null, "first": null, "last": 6 }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": null, "first": null, "last": 10 }),
            (false, false, &["a", "b", "c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_last_after() {
        do_test(
            json!({ "after": "b", "before": null, "first": null, "last": 2 }),
            (true, false, &["e", "f"]),
        )
        .await;

        do_test(
            json!({ "after": "b", "before": null, "first": null, "last": 6 }),
            (true, false, &["c", "d", "e", "f"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_last_before() {
        do_test(
            json!({ "after": null, "before": "e", "first": null, "last": 2 }),
            (true, true, &["c", "d"]),
        )
        .await;

        do_test(
            json!({ "after": null, "before": "e", "first": null, "last": 6 }),
            (false, true, &["a", "b", "c", "d"]),
        )
        .await;
    }

    #[async_std::test]
    pub async fn test_connection_stream_last_between() {
        do_test(
            json!({ "after": "a", "before": "f", "first": null, "last": 2 }),
            (true, true, &["d", "e"]),
        )
        .await;

        do_test(
            json!({ "after": "a", "before": "f", "first": null, "last": 6 }),
            (true, true, &["b", "c", "d", "e"]),
        )
        .await;
    }
}
