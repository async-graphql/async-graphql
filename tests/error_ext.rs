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

#[tokio::test]
pub async fn test_failure() {
    #[derive(thiserror::Error, Debug, PartialEq)]
    enum MyError {
        #[error("error1")]
        Error1,

        #[error("error2")]
        Error2,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn failure(&self) -> Result<i32> {
            Err(Error::new_with_source(MyError::Error1))
        }

        async fn failure2(&self) -> Result<i32> {
            Err(Error::new_with_source(MyError::Error2))
        }

        async fn failure3(&self) -> Result<i32> {
            Err(Error::new_with_source(MyError::Error1)
                .extend_with(|_, values| values.set("a", 1))
                .extend_with(|_, values| values.set("b", 2)))
        }

        async fn failure4(&self) -> Result<i32> {
            Err(Error::new_with_source(MyError::Error2))
                .extend_err(|_, values| values.set("a", 1))
                .extend_err(|_, values| values.set("b", 2))?;
            Ok(1)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let err = schema
        .execute("{ failure }")
        .await
        .into_result()
        .unwrap_err()
        .remove(0);
    assert_eq!(err.source::<MyError>().unwrap(), &MyError::Error1);

    let err = schema
        .execute("{ failure2 }")
        .await
        .into_result()
        .unwrap_err()
        .remove(0);
    assert_eq!(err.source::<MyError>().unwrap(), &MyError::Error2);

    let err = schema
        .execute("{ failure3 }")
        .await
        .into_result()
        .unwrap_err()
        .remove(0);
    assert_eq!(err.source::<MyError>().unwrap(), &MyError::Error1);
    assert_eq!(
        err.extensions,
        Some({
            let mut values = ErrorExtensionValues::default();
            values.set("a", 1);
            values.set("b", 2);
            values
        })
    );

    let err = schema
        .execute("{ failure4 }")
        .await
        .into_result()
        .unwrap_err()
        .remove(0);
    assert_eq!(err.source::<MyError>().unwrap(), &MyError::Error2);
    assert_eq!(
        err.extensions,
        Some({
            let mut values = ErrorExtensionValues::default();
            values.set("a", 1);
            values.set("b", 2);
            values
        })
    );
}

#[tokio::test]
pub async fn test_failure2() {
    #[derive(thiserror::Error, Debug, PartialEq)]
    enum MyError {
        #[error("error1")]
        Error1,
    }

    #[cfg(feature = "custom-error-conversion")]
    impl From<MyError> for Error {
        fn from(e: MyError) -> Self {
            Self::new_with_source(e)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn failure(&self) -> Result<i32, MyError> {
            Err(MyError::Error1)
        }
    }
}
