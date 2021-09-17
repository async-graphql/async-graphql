use async_graphql::*;

#[tokio::test]
pub async fn test_error_extensions() {
    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum {
        Create,
        Delete,
        Update,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn extend_err(&self) -> Result<i32> {
            Err("my error".extend_with(|err, e| {
                e.set("msg", err.to_string());
                e.set("code", 100);
                e.set("action", MyEnum::Create)
            }))
        }

        async fn extend_result(&self) -> Result<i32> {
            Err(Error::from("my error"))
                .extend_err(|_, e| {
                    e.set("msg", "my error");
                    e.set("code", 100);
                })
                .extend_err(|_, e| {
                    e.set("code2", 20);
                })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        serde_json::to_value(&schema.execute("{ extendErr }").await).unwrap(),
        serde_json::json!({
            "data": null,
            "errors": [{
                "message": "my error",
                "locations": [{
                    "column": 3,
                    "line": 1,
                }],
                "path": ["extendErr"],
                "extensions": {
                    "msg": "my error",
                    "code": 100,
                    "action": "CREATE",
                }
            }]
        })
    );

    assert_eq!(
        serde_json::to_value(&schema.execute("{ extendResult }").await).unwrap(),
        serde_json::json!({
            "data": null,
            "errors": [{
                "message": "my error",
                "locations": [{
                    "column": 3,
                    "line": 1,
                }],
                "path": ["extendResult"],
                "extensions": {
                    "msg": "my error",
                    "code": 100,
                    "code2": 20
                }
            }]
        })
    );
}
