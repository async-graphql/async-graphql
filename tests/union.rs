use async_graphql::*;

#[tokio::test]
pub async fn test_union_simple_object() {
    #[derive(SimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(Union)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[tokio::test]
pub async fn test_union_simple_object2() {
    #[derive(SimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(Union)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[tokio::test]
pub async fn test_multiple_unions() {
    struct MyObj;

    #[Object]
    impl MyObj {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    #[derive(Union)]
    enum UnionA {
        MyObj(MyObj),
    }

    #[derive(Union)]
    enum UnionB {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn union_a(&self) -> UnionA {
            MyObj.into()
        }
        async fn union_b(&self) -> UnionB {
            MyObj.into()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_output_type::<UnionA>() // `UnionA` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            unionA {
               ... on MyObj {
                valueA
                valueB
                valueC
              }
            }
            unionB {
                ... on MyObj {
                 valueA
                 valueB
                 valueC
               }
             }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "unionA": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            },
            "unionB": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }
        })
    );
}

#[tokio::test]
pub async fn test_multiple_objects_in_multiple_unions() {
    struct MyObjOne;

    #[Object]
    impl MyObjOne {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    struct MyObjTwo;

    #[Object]
    impl MyObjTwo {
        async fn value_a(&self) -> i32 {
            1
        }
    }

    #[derive(Union)]
    enum UnionA {
        MyObjOne(MyObjOne),
        MyObjTwo(MyObjTwo),
    }

    #[derive(Union)]
    enum UnionB {
        MyObjOne(MyObjOne),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn my_obj(&self) -> Vec<UnionA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_output_type::<UnionB>() // `UnionB` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            myObj {
                ... on MyObjTwo {
                    valueA
                }
                ... on MyObjOne {
                    valueA
                    valueB
                    valueC
                }
            }
         }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "myObj": [{
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }, {
                "valueA": 1
            }]
        })
    );
}

#[tokio::test]
pub async fn test_union_field_result() {
    struct MyObj;

    #[Object]
    impl MyObj {
        async fn value(&self) -> Result<i32> {
            Ok(10)
        }
    }

    #[derive(Union)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self) -> Node {
            MyObj.into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    value
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "node": {
                "value": 10,
            }
        })
    );
}

#[tokio::test]
pub async fn test_union_flatten() {
    #[derive(SimpleObject)]
    struct MyObj1 {
        value1: i32,
    }

    #[derive(SimpleObject)]
    struct MyObj2 {
        value2: i32,
    }

    #[derive(Union)]
    enum InnerUnion1 {
        A(MyObj1),
    }

    #[derive(Union)]
    enum InnerUnion2 {
        B(MyObj2),
    }

    #[derive(Union)]
    enum MyUnion {
        #[graphql(flatten)]
        Inner1(InnerUnion1),

        #[graphql(flatten)]
        Inner2(InnerUnion2),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value1(&self) -> MyUnion {
            InnerUnion1::A(MyObj1 { value1: 99 }).into()
        }

        async fn value2(&self) -> MyUnion {
            InnerUnion2::B(MyObj2 { value2: 88 }).into()
        }

        async fn value3(&self) -> InnerUnion1 {
            InnerUnion1::A(MyObj1 { value1: 77 })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"
    {
        value1 {
            ... on MyObj1 {
                value1
            }
        }
        value2 {
            ... on MyObj2 {
                value2
            }
        }
        value3 {
            ... on MyObj1 {
                value1
            }
        }
    }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "value1": {
                "value1": 99,
            },
            "value2": {
                "value2": 88,
            },
            "value3": {
                "value1": 77,
            }
        })
    );
}

#[tokio::test]
pub async fn test_trait_object_in_union() {
    pub trait ProductTrait: Send + Sync {
        fn id(&self) -> &str;
    }

    #[Object]
    impl dyn ProductTrait {
        #[graphql(name = "id")]
        async fn gql_id(&self, _ctx: &Context<'_>) -> &str {
            self.id()
        }
    }

    struct MyProduct;

    impl ProductTrait for MyProduct {
        fn id(&self) -> &str {
            "abc"
        }
    }

    #[derive(Union)]
    pub enum Content {
        Product(Box<dyn ProductTrait>),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> Content {
            Content::Product(Box::new(MyProduct))
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value { ... on ProductTrait { id } } }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "value": {
                "id": "abc"
            }
        })
    );
}

macro_rules! generate_union {
    ($name:ident, $variant_ty:ty) => {
        #[derive(Union)]
        pub enum $name {
            Val($variant_ty),
        }
    };
}

#[test]
pub fn test_macro_generated_union() {
    #[derive(SimpleObject)]
    pub struct IntObj {
        pub val: i32,
    }

    generate_union!(MyEnum, IntObj);

    let _ = MyEnum::Val(IntObj { val: 1 });
}

#[tokio::test]
pub async fn test_union_with_oneof_object() {
    #[derive(SimpleObject, InputObject)]
    #[graphql(input_name = "MyObjInput")]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(OneofObject, Union)]
    #[graphql(input_name = "NodeInput")]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn node(&self, input: Node) -> Node {
            input
        }
    }

    let query = r#"{
            node(input: { myObj: { id: 10, title: "abc" } }) {
                ... on MyObj {
                    id title
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "node": {
                "id": 10,
                "title": "abc",
            }
        })
    );
}

#[tokio::test]
pub async fn test_union_with_generic() {
    struct MyObj<T> {
        value: T,
    }

    #[Object(
        concrete(name = "MyObjString", params(String)),
        concrete(name = "MyObjInt", params(i64))
    )]
    impl<T> MyObj<T>
    where
        T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker,
        MyObj<T>: async_graphql::OutputTypeMarker,
    {
        async fn id(&self) -> i32 {
            10
        }

        async fn title(&self) -> String {
            "abc".to_string()
        }

        async fn value(&self) -> &T {
            &self.value
        }
    }

    #[derive(Union)]
    #[graphql(concrete(name = "NodeInt", params(i64)))]
    #[graphql(concrete(name = "NodeString", params(String)))]
    enum Node<T>
    where
        T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker,
        Node<T>: async_graphql::OutputTypeMarker,
        MyObj<T>: async_graphql::OutputTypeMarker,
    {
        MyObj(MyObj<T>),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn node_int(&self) -> Node<i64> {
            Node::MyObj(MyObj { value: 10 })
        }

        async fn node_str(&self) -> Node<String> {
            Node::MyObj(MyObj {
                value: "abc".to_string(),
            })
        }
    }

    let query = r#"{
            nodeInt {
                ... on MyObjInt {
                    value
                }
            }
            nodeStr {
                ... on MyObjString {
                    value
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "nodeInt": {
                "value": 10,
            },
            "nodeStr": {
                "value": "abc",
            }
        })
    );

    println!("{}", schema.sdl());
}

#[tokio::test]
pub async fn test_union_with_sub_generic() {
    struct MyObj<G> {
        _marker: std::marker::PhantomData<G>,
    }

    #[Object]
    impl<G: Send + Sync> MyObj<G> {
        async fn id(&self) -> i32 {
            10
        }
    }

    struct MyObj2<G> {
        _marker: std::marker::PhantomData<G>,
    }

    #[Object]
    impl<G: Send + Sync> MyObj2<G> {
        async fn id(&self) -> i32 {
            10
        }
    }

    #[derive(Union)]
    #[graphql(concrete(name = "NodeMyObj", params("MyObj<G>"), bounds("G: Send + Sync")))]
    enum Node<T> where T: Send + Sync + async_graphql::OutputType + async_graphql::OutputTypeMarker {
        Nested(MyObj2<T>),
        NotNested(T),
    }

    struct Query;

    #[Object]
    impl Query {
        async fn nested(&self) -> Node<MyObj<()>> {
            Node::Nested(MyObj2 {
                _marker: std::marker::PhantomData,
            })
        }

        async fn not_nested(&self) -> Node<MyObj<()>> {
            Node::NotNested(MyObj {
                _marker: std::marker::PhantomData,
            })
        }
    }

    let query = r#"{
            nested {
                ... on MyObj {
                    id
                }
                ... on MyObj2 {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "nested": {
                "id": 10,
            },
        })
    );

    println!("{}", schema.sdl());
}
