use async_graphql::*;
use std::fmt::{self, Display, Formatter};

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

#[tokio::test]
pub async fn test_failure() {
    #[derive(Debug)]
    struct MyError;

    impl Display for MyError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "my error")
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn failure(&self) -> Result<i32> {
            Err(Failure(MyError).into())
        }

        async fn failure2(&self) -> Result<i32> {
            Err(Failure(MyError))?;
            Ok(1)
        }

        async fn failure3(&self) -> Result<i32> {
            Err(Failure(MyError)
                .extend_with(|_, values| values.set("a", 1))
                .extend_with(|_, values| values.set("b", 2)))?;
            Ok(1)
        }

        async fn failure4(&self) -> Result<i32> {
            Err(Failure(MyError))
                .extend_err(|_, values| values.set("a", 1))
                .extend_err(|_, values| values.set("b", 2))?;
            Ok(1)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ failure }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "my error".to_string(),
            debug_message: Some("MyError".to_string()),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("failure".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ failure2 }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "my error".to_string(),
            debug_message: Some("MyError".to_string()),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("failure2".to_string())],
            extensions: None
        }]
    );

    assert_eq!(
        schema
            .execute("{ failure3 }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "my error".to_string(),
            debug_message: Some("MyError".to_string()),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("failure3".to_string())],
            extensions: Some({
                let mut values = ErrorExtensionValues::default();
                values.set("a", 1);
                values.set("b", 2);
                values
            })
        }]
    );

    assert_eq!(
        schema
            .execute("{ failure4 }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "my error".to_string(),
            debug_message: Some("MyError".to_string()),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("failure4".to_string())],
            extensions: Some({
                let mut values = ErrorExtensionValues::default();
                values.set("a", 1);
                values.set("b", 2);
                values
            })
        }]
    );
}
