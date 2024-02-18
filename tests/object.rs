use async_graphql::*;

#[tokio::test]
async fn test_flatten() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_flatten_with_context() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    struct B;

    #[Object]
    impl B {
        #[graphql(flatten)]
        async fn a(&self, _ctx: &Context<'_>) -> A {
            A { a: 100, b: 200 }
        }

        async fn c(&self) -> i32 {
            300
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "a"},
                    {"name": "b"},
                    {"name": "c"}
                ]
            }
        })
    );

    let query = "{ obj { a b c } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 100,
                "b": 200,
                "c": 300,
            }
        })
    );
}

#[tokio::test]
async fn test_object_process_with_field() {
    struct Query;

    #[Object]
    impl Query {
        async fn test(
            &self,
            #[graphql(process_with = "str::make_ascii_uppercase")] processed_arg: String,
        ) -> String {
            processed_arg
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ test(processedArg: \"smol\") }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "test": "SMOL"
        })
    );
}

#[tokio::test]
async fn ignore_name_conflicts() {
    #[derive(SimpleObject)]
    #[graphql(name = "MyObj")]
    struct MyObj {
        name: String,
    }

    #[derive(SimpleObject)]
    #[graphql(name = "MyObj")]
    struct MyObjRef<'a> {
        name: &'a str,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj_owned(&self) -> MyObj {
            MyObj {
                name: "a".to_string(),
            }
        }

        async fn obj_ref(&self) -> MyObjRef<'_> {
            MyObjRef { name: "b" }
        }
    }

    let schema = Schema::build_with_ignore_name_conflicts(
        Query,
        EmptyMutation,
        EmptySubscription,
        ["MyObj"],
    )
    .finish();

    let query = r#"
    {
        objOwned { name }
        objRef { name }
    }
    "#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "objOwned": { "name": "a" },
            "objRef": { "name": "b" },
        })
    );
}

#[tokio::test]
async fn test_impl_dyn_trait() {
    use async_graphql::*;

    trait MyTrait: Send + Sync {
        fn name(&self) -> &str;
    }

    #[Object]
    impl dyn MyTrait + '_ {
        #[graphql(name = "name")]
        async fn gql_name(&self) -> &str {
            self.name()
        }
    }

    struct MyObj(String);

    impl MyTrait for MyObj {
        fn name(&self) -> &str {
            &self.0
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn objs(&self) -> Vec<Box<dyn MyTrait>> {
            vec![
                Box::new(MyObj("a".to_string())),
                Box::new(MyObj("b".to_string())),
            ]
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let res = schema
        .execute("{ objs { name } }")
        .await
        .into_result()
        .unwrap()
        .data;
    assert_eq!(
        res,
        value!({
            "objs": [
                { "name": "a" },
                { "name": "b" },
            ]
        })
    );
}

#[tokio::test]
async fn test_optional_output_with_try() {
    struct B {
        some: Option<bool>,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self, ctx: &Context<'_>) -> Option<u32> {
            let x = ctx.data_unchecked::<B>();

            if x.some? {
                Some(300)
            } else {
                None
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let res = schema
        .execute(Request::from("{ obj }").data(B { some: Some(true) }))
        .await
        .into_result()
        .unwrap()
        .data;
    assert_eq!(
        res,
        value!({
            "obj": 300 ,
        })
    );

    let res = schema
        .execute(Request::from("{ obj }").data(B { some: None }))
        .await
        .into_result()
        .unwrap()
        .data;
    assert_eq!(
        res,
        value!({
            "obj": null,
        })
    );
}
