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

    let resp = schema.execute("{ error1:optError error2:optError }").await;
    assert_eq!(
        resp.data,
        value!({
            "error1": null,
            "error2": null,
        })
    );
    assert_eq!(
        resp.errors,
        vec![
            ServerError {
                message: "TestError".to_string(),
                source: None,
                locations: vec![Pos { line: 1, column: 3 }],
                path: vec![PathSegment::Field("error1".to_owned())],
                extensions: None,
            },
            ServerError {
                message: "TestError".to_string(),
                source: None,
                locations: vec![Pos {
                    line: 1,
                    column: 19,
                }],
                path: vec![PathSegment::Field("error2".to_owned())],
                extensions: None,
            },
        ]
    );

    assert_eq!(
        schema
            .execute("{ optError }")
            .await
            .into_result()
            .unwrap_err(),
        vec![ServerError {
            message: "TestError".to_string(),
            source: None,
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
            source: None,
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
    #[graphql(field(name = "value2", ty = "i32"))]
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

#[tokio::test]
pub async fn test_error_propagation() {
    struct ParentObject;

    #[Object]
    impl ParentObject {
        async fn child(&self) -> ChildObject {
            ChildObject
        }

        async fn child_opt(&self) -> Option<ChildObject> {
            Some(ChildObject)
        }
    }

    struct ChildObject;

    #[Object]
    impl ChildObject {
        async fn name(&self) -> Result<i32> {
            Err("myerror".into())
        }

        async fn name_opt(&self) -> Option<Result<i32>> {
            Some(Err("myerror".into()))
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn parent(&self) -> ParentObject {
            ParentObject
        }

        async fn parent_opt(&self) -> Option<ParentObject> {
            Some(ParentObject)
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let resp = schema.execute("{ parent { child { name } } }").await;

    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "myerror".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 20,
            }],
            path: vec![
                PathSegment::Field("parent".to_owned()),
                PathSegment::Field("child".to_owned()),
                PathSegment::Field("name".to_owned()),
            ],
            extensions: None,
        }]
    );

    let resp = schema.execute("{ parent { childOpt { name } } }").await;
    assert_eq!(
        resp.data,
        value!({
            "parent": {
                "childOpt": null,
            }
        })
    );
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "myerror".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 23,
            }],
            path: vec![
                PathSegment::Field("parent".to_owned()),
                PathSegment::Field("childOpt".to_owned()),
                PathSegment::Field("name".to_owned()),
            ],
            extensions: None,
        }]
    );

    let resp = schema.execute("{ parentOpt { child { name } } }").await;
    assert_eq!(resp.data, value!({ "parentOpt": null }));
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "myerror".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 23,
            }],
            path: vec![
                PathSegment::Field("parentOpt".to_owned()),
                PathSegment::Field("child".to_owned()),
                PathSegment::Field("name".to_owned()),
            ],
            extensions: None,
        }]
    );

    let resp = schema.execute("{ parentOpt { child { nameOpt } } }").await;
    assert_eq!(
        resp.data,
        value!({
            "parentOpt": {
                "child": {
                    "nameOpt": null,
                }
            },
        })
    );
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "myerror".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 23,
            }],
            path: vec![
                PathSegment::Field("parentOpt".to_owned()),
                PathSegment::Field("child".to_owned()),
                PathSegment::Field("nameOpt".to_owned()),
            ],
            extensions: None,
        }]
    );
}
