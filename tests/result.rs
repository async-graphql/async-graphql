use async_graphql::*;
use futures_util::stream::Stream;

#[tokio::test]
pub async fn test_fieldresult() {
    struct Query;

    #[Object]
    impl Query {
        async fn error(&self) -> Result<i32> {
            Err("TestError".into())
        }

        async fn opt_error(&self) -> Option<Result<i32>> {
            Some(Err("TestError".into()))
        }

        async fn vec_error(&self) -> Vec<Result<i32>> {
            vec![Ok(1), Err("TestError".into())]
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema.execute("{ error1:error error2:error }").await,
        Response {
            data: value!({ "error1": null, "error2": null }),
            extensions: Default::default(),
            cache_control: Default::default(),
            errors: vec![
                ServerError {
                    message: "TestError".to_string(),
                    locations: vec![Pos { line: 1, column: 3 }],
                    path: vec![PathSegment::Field("error1".to_owned())],
                    extensions: None,
                },
                ServerError {
                    message: "TestError".to_string(),
                    locations: vec![Pos {
                        line: 1,
                        column: 16,
                    }],
                    path: vec![PathSegment::Field("error2".to_owned())],
                    extensions: None,
                },
            ],
            http_headers: Default::default(),
        }
    );

    assert_eq!(
        schema
            .execute("{ optError }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![PathSegment::Field("optError".to_owned())],
            extensions: None,
        }]
    );

    assert_eq!(
        schema
            .execute("{ vecError }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            locations: vec![Pos { line: 1, column: 3 }],
            path: vec![
                PathSegment::Field("vecError".to_owned()),
                PathSegment::Index(1)
            ],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_custom_error() {
    #[derive(Clone)]
    struct MyError;

    impl From<MyError> for Error {
        fn from(_: MyError) -> Self {
            Error::new("custom error")
        }
    }

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        value1: i32,
    }

    #[ComplexObject]
    impl MyObj {
        async fn value2(&self) -> Result<i32, MyError> {
            Err(MyError)
        }
    }

    #[derive(Interface)]
    #[graphql(field(name = "value2", type = "i32"))]
    enum MyInterface {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> Result<i32, MyError> {
            Err(MyError)
        }
    }

    struct Subscription;

    #[Subscription]
    impl Subscription {
        async fn value1(&self) -> Result<impl Stream<Item = i32>, MyError> {
            Err::<futures_util::stream::Once<futures_util::future::Ready<i32>>, _>(MyError)
        }

        async fn value2(&self) -> impl Stream<Item = Result<i32, MyError>> {
            futures_util::stream::once(async move { Err(MyError) })
        }
    }
}
