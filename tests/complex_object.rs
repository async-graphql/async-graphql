use core::marker::PhantomData;

use async_graphql::*;

#[tokio::test]
async fn test_complex_object_process_with_method_field() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
    }

    #[ComplexObject]
    impl MyObj {
        async fn test(
            &self,
            #[graphql(process_with = "str::make_ascii_uppercase")] processed_complex_arg: String,
        ) -> String {
            processed_complex_arg
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj { a: 10 }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ obj { test(processedComplexArg: \"smol\") } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "obj": {
                "test": "SMOL"
            }
        })
    );
}

#[tokio::test]
pub async fn test_complex_object() {
    /// A complex object.
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
        b: i32,
    }

    #[ComplexObject]
    impl MyObj {
        /// A field named `c`.
        async fn c(&self) -> i32 {
            self.a + self.b
        }

        /// A field named `d`.
        async fn d(&self, #[graphql(desc = "An argument named `v`.")] v: i32) -> i32 {
            self.a + self.b + v
        }
    }

    #[allow(clippy::duplicated_attributes)]
    #[derive(Interface)]
    #[graphql(
        field(name = "a", ty = "&i32"),
        field(name = "b", ty = "&i32"),
        field(name = "c", ty = "i32"),
        field(name = "d", ty = "i32", arg(name = "v", ty = "i32"))
    )]
    enum ObjInterface {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> MyObj {
            MyObj { a: 10, b: 20 }
        }

        async fn obj2(&self) -> ObjInterface {
            MyObj { a: 10, b: 20 }.into()
        }
    }

    let query = "{ obj { a b c d(v:100) } obj2 { a b c d(v:200) } }";
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "obj": {
                "a": 10,
                "b": 20,
                "c": 30,
                "d": 130,
            },
            "obj2": {
                "a": 10,
                "b": 20,
                "c": 30,
                "d": 230,
            }
        })
    );
}

#[tokio::test]
pub async fn test_complex_object_with_generic_context_data() {
    trait MyData: Send + Sync {
        fn answer(&self) -> i64;
    }

    struct DefaultMyData {}

    impl MyData for DefaultMyData {
        fn answer(&self) -> i64 {
            42
        }
    }

    struct MyQuery<D: MyData> {
        marker: PhantomData<D>,
    }

    #[Object]
    impl<D> MyQuery<D>
    where
        D: 'static + MyData,
    {
        #[graphql(skip)]
        pub fn new() -> Self {
            Self {
                marker: PhantomData,
            }
        }

        async fn obj(&self, ctx: &Context<'_>) -> MyObject<D> {
            MyObject::new(ctx.data_unchecked::<D>().answer())
        }
    }

    #[derive(SimpleObject, Debug, Clone, Hash, Eq, PartialEq)]
    #[graphql(complex)]
    struct MyObject<D: MyData> {
        my_val: i64,
        #[graphql(skip)]
        marker: PhantomData<D>,
    }

    #[ComplexObject]
    impl<D: MyData> MyObject<D> {
        #[graphql(skip)]
        pub fn new(my_val: i64) -> Self {
            Self {
                my_val,
                marker: PhantomData,
            }
        }
    }

    let schema = Schema::build(
        MyQuery::<DefaultMyData>::new(),
        EmptyMutation,
        EmptySubscription,
    )
    .data(DefaultMyData {})
    .finish();

    assert_eq!(
        schema.execute("{ obj { myVal } }").await.data,
        value!({
            "obj": {
                "myVal": 42,
            }
        })
    );
}

#[tokio::test]
pub async fn test_complex_object_with_generic_concrete_type() {
    #[derive(SimpleObject)]
    #[graphql(concrete(name = "MyObjIntString", params(i32, String)))]
    #[graphql(concrete(name = "MyObji64f32", params(i64, u8)))]
    #[graphql(complex)]
    struct MyObj<A, B> where A: OutputType + OutputTypeMarker, B: OutputType + OutputTypeMarker, MyObj<A, B>: async_graphql::OutputTypeMarker {
        a: A,
        b: B,
    }

    #[ComplexObject]
    impl MyObj<i32, String> {
        async fn value_a(&self) -> String {
            format!("i32,String {},{}", self.a, self.b)
        }
    }

    #[ComplexObject]
    impl MyObj<i64, u8> {
        async fn value_b(&self) -> String {
            format!("i64,u8 {},{}", self.a, self.b)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn q1(&self) -> MyObj<i32, String> {
            MyObj {
                a: 100,
                b: "abc".to_string(),
            }
        }

        async fn q2(&self) -> MyObj<i64, u8> {
            MyObj { a: 100, b: 28 }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ q1 { a b valueA } q2 { a b valueB } }";
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "q1": {
                "a": 100,
                "b": "abc",
                "valueA": "i32,String 100,abc",
            },
            "q2": {
                "a": 100,
                "b": 28,
                "valueB": "i64,u8 100,28",
            }
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObjIntString") { fields { name type { kind ofType { name } } } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "a",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "Int" },
                        },
                    },
                    {
                        "name": "b",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "String" },
                        },
                    },
                    {
                        "name": "valueA",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "String" },
                        },
                    },
                ]
            }
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObji64f32") { fields { name type { kind ofType { name } } } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "a",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "Int" },
                        },
                    },
                    {
                        "name": "b",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "Int" },
                        },
                    },
                    {
                        "name": "valueB",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "String" },
                        },
                    },
                ]
            }
        })
    );

    assert_eq!(
        schema
            .execute(
                r#"{ __type(name: "Query") { fields { name type { kind ofType { name } } } } }"#
            )
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "fields": [
                    {
                        "name": "q1",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "MyObjIntString" },
                        },
                    },
                    {
                        "name": "q2",
                        "type": {
                            "kind": "NON_NULL",
                            "ofType": { "name": "MyObji64f32" },
                        },
                    },
                ]
            }
        })
    );
}

#[tokio::test]
async fn test_flatten() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct B {
        #[graphql(skip)]
        a: A,
        c: i32,
    }

    #[ComplexObject]
    impl B {
        #[graphql(flatten)]
        async fn a(&self) -> &A {
            &self.a
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B {
                a: A { a: 100, b: 200 },
                c: 300,
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "c"},
                    {"name": "a"},
                    {"name": "b"}
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

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct B {
        #[graphql(skip)]
        a: A,
        c: i32,
    }

    #[ComplexObject]
    impl B {
        #[graphql(flatten)]
        async fn a(&self, _ctx: &Context<'_>) -> &A {
            &self.a
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B {
                a: A { a: 100, b: 200 },
                c: 300,
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "c"},
                    {"name": "a"},
                    {"name": "b"}
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
async fn test_flatten_with_result() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct B {
        #[graphql(skip)]
        a: A,
        c: i32,
    }

    #[ComplexObject]
    impl B {
        #[graphql(flatten)]
        async fn a(&self) -> FieldResult<&A> {
            Ok(&self.a)
        }
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B {
                a: A { a: 100, b: 200 },
                c: 300,
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = "{ __type(name: \"B\") { fields { name } } }";
    assert_eq!(
        schema.execute(query).await.data,
        value!({
            "__type": {
                "fields": [
                    {"name": "c"},
                    {"name": "a"},
                    {"name": "b"}
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
