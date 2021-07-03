use async_graphql::*;
use core::marker::PhantomData;

#[tokio::test]
pub async fn test_complex_object() {
    #[derive(SimpleObject)]
    #[graphql(complex)]
    struct MyObj {
        a: i32,
        b: i32,
    }

    #[ComplexObject]
    impl MyObj {
        async fn c(&self) -> i32 {
            self.a + self.b
        }

        async fn d(&self, v: i32) -> i32 {
            self.a + self.b + v
        }
    }

    #[derive(Interface)]
    #[graphql(
        field(name = "a", type = "&i32"),
        field(name = "b", type = "&i32"),
        field(name = "c", type = "i32"),
        field(name = "d", type = "i32", arg(name = "v", type = "i32"))
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
        fn answer(&self) -> i64 { 42 }
    }

    struct MyQuery<D: MyData> {
        marker: PhantomData<D>,
    }

    #[Object]
    impl<D> MyQuery<D> where D: 'static + MyData {
        #[graphql(skip)]
        pub fn new() -> Self {
            Self { marker: PhantomData }
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
            Self { my_val, marker: PhantomData }
        }
    }

    let schema = Schema::build(MyQuery::<DefaultMyData>::new(), EmptyMutation, EmptySubscription)
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
