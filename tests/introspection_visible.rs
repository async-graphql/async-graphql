use async_graphql::*;
use futures_util::Stream;
use serde::Deserialize;

#[tokio::test]
pub async fn test_type_visible() {
    #[derive(SimpleObject)]
    #[graphql(visible = false)]
    struct MyObj {
        a: i32,
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": null,
        })
    );

    #[derive(Deserialize)]
    struct QueryResponse {
        #[serde(rename = "__schema")]
        schema: SchemaResponse,
    }

    #[derive(Deserialize)]
    struct SchemaResponse {
        types: Vec<TypeResponse>,
    }

    #[derive(Deserialize)]
    struct TypeResponse {
        name: String,
    }

    let resp: QueryResponse = from_value(
        schema
            .execute(r#"{ __schema { types { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
    )
    .unwrap();

    assert!(!resp.schema.types.into_iter().any(|ty| ty.name == "MyObj"));
}

#[tokio::test]
pub async fn test_field_visible() {
    #[derive(SimpleObject)]
    struct MyObj {
        a: i32,
        #[graphql(visible = false)]
        b: i32,
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj {
            todo!()
        }

        #[graphql(visible = false)]
        async fn c(&self) -> i32 {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    #[derive(Debug, Deserialize)]
    struct QueryResponse {
        #[serde(rename = "__type")]
        ty: TypeResponse,
    }

    #[derive(Debug, Deserialize)]
    struct TypeResponse {
        fields: Vec<FieldResposne>,
    }

    #[derive(Debug, Deserialize)]
    struct FieldResposne {
        name: String,
    }

    let resp: QueryResponse = from_value(
        schema
            .execute(r#"{ __type(name: "MyObj") { fields { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
    )
    .unwrap();
    assert_eq!(
        resp.ty
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        vec!["a"]
    );

    let resp: QueryResponse = from_value(
        schema
            .execute(r#"{ __type(name: "Query") { fields { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
    )
    .unwrap();
    assert_eq!(
        resp.ty
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        vec!["obj"]
    );
}

#[tokio::test]
pub async fn test_enum_value_visible() {
    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum {
        A,
        B,
        #[graphql(visible = false)]
        C,
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn e(&self) -> MyEnum {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    #[derive(Debug, Deserialize)]
    struct QueryResponse {
        #[serde(rename = "__type")]
        ty: TypeResponse,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TypeResponse {
        enum_values: Vec<EnumValueResponse>,
    }

    #[derive(Debug, Deserialize)]
    struct EnumValueResponse {
        name: String,
    }

    let resp: QueryResponse = from_value(
        schema
            .execute(r#"{ __type(name: "MyEnum") { enumValues { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
    )
    .unwrap();
    assert_eq!(
        resp.ty
            .enum_values
            .iter()
            .map(|value| value.name.as_str())
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
}

#[tokio::test]
pub async fn test_visible_fn() {
    mod nested {
        use async_graphql::Context;

        pub struct IsAdmin(pub bool);

        pub fn is_admin(ctx: &Context<'_>) -> bool {
            ctx.data_unchecked::<IsAdmin>().0
        }
    }

    use nested::IsAdmin;

    #[derive(SimpleObject)]
    #[graphql(visible = "nested::is_admin")]
    struct MyObj {
        a: i32,
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute(Request::new(r#"{ __type(name: "MyObj") { name } }"#).data(IsAdmin(false)))
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": null,
        })
    );

    assert_eq!(
        schema
            .execute(Request::new(r#"{ __type(name: "MyObj") { name } }"#).data(IsAdmin(true)))
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "name": "MyObj",
            },
        })
    );
}

#[tokio::test]
pub async fn test_indirect_hiding_type() {
    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum1 {
        A,
    }

    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum2 {
        A,
    }

    struct MyDirective;

    impl CustomDirective for MyDirective {}

    #[Directive(location = "Field")]
    fn my_directive1(_a: MyEnum1) -> impl CustomDirective {
        MyDirective
    }

    #[Directive(location = "Field", visible = false)]
    fn my_directive2(_a: MyEnum2) -> impl CustomDirective {
        MyDirective
    }

    #[derive(SimpleObject)]
    struct MyObj1 {
        a: i32,
        b: MyObj2,
        c: MyObj3,
    }

    #[derive(SimpleObject)]
    struct MyObj2 {
        a: i32,
    }

    #[derive(SimpleObject)]
    struct MyObj3 {
        a: i32,
        #[graphql(visible = false)]
        b: MyObj5,
    }

    #[derive(SimpleObject)]
    #[graphql(visible = false)]
    struct MyObj4 {
        a: i32,
    }

    #[derive(SimpleObject)]
    struct MyObj5 {
        a: i32,
    }

    #[derive(InputObject)]
    struct MyInputObj1 {
        a: i32,
        b: MyInputObj2,
        c: MyInputObj3,
    }

    #[derive(InputObject)]
    struct MyInputObj2 {
        a: i32,
    }

    #[derive(InputObject)]
    struct MyInputObj3 {
        a: i32,
        #[graphql(visible = false)]
        b: MyInputObj4,
    }

    #[derive(InputObject)]
    struct MyInputObj4 {
        a: i32,
    }

    #[derive(InputObject)]
    struct MyInputObj5 {
        a: i32,
    }

    #[derive(Union)]
    enum MyUnion {
        MyObj3(MyObj3),
        MyObj4(MyObj4),
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", ty = "&i32"))]
    enum MyInterface {
        MyObj3(MyObj3),
        MyObj4(MyObj4),
    }

    #[derive(Interface)]
    #[graphql(visible = false, field(name = "a", ty = "&i32"))]
    enum MyInterface2 {
        MyObj3(MyObj3),
        MyObj4(MyObj4),
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        #[graphql(visible = false)]
        async fn obj1(&self) -> MyObj1 {
            todo!()
        }

        async fn obj3(&self) -> MyObj3 {
            todo!()
        }

        #[graphql(visible = false)]
        async fn input_obj1(&self, _obj: MyInputObj1) -> i32 {
            todo!()
        }

        async fn input_obj3(&self, _obj: MyInputObj3) -> i32 {
            todo!()
        }

        async fn input_obj5(&self, #[graphql(visible = false)] _obj: Option<MyInputObj5>) -> i32 {
            todo!()
        }

        async fn union1(&self) -> MyUnion {
            todo!()
        }

        async fn interface1(&self) -> MyInterface {
            todo!()
        }

        async fn interface2(&self) -> MyInterface2 {
            todo!()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .directive(my_directive1)
        .directive(my_directive2)
        .finish();
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj1") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj2") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj3") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "name": "MyObj3" } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj1") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj2") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj3") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "name": "MyInputObj3" } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyUnion") { possibleTypes { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "possibleTypes": [{ "name": "MyObj3" }] } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInterface") { possibleTypes { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "possibleTypes": [{ "name": "MyObj3" }] } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInterface2") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj3") { interfaces { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "interfaces": [{ "name": "MyInterface" }] } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj3") { fields { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "fields": [
            { "name": "a" },
        ]}})
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj5") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj3") { inputFields { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "inputFields": [
            { "name": "a" },
        ]}})
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj4") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyInputObj5") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyEnum1") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": { "name": "MyEnum1" } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyEnum2") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );
}

#[tokio::test]
pub async fn root() {
    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn value(&self) -> i32 {
            todo!()
        }
    }

    struct Mutation;

    #[Object(visible = false)]
    #[allow(unreachable_code)]
    impl Mutation {
        async fn value(&self) -> i32 {
            todo!()
        }
    }

    struct Subscription;

    #[Subscription(visible = false)]
    #[allow(unreachable_code)]
    impl Subscription {
        async fn value(&self) -> impl Stream<Item = i32> {
            futures_util::stream::iter(vec![1, 2, 3])
        }
    }

    let schema = Schema::new(Query, Mutation, Subscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Mutation") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    let schema = Schema::new(Query, Mutation, Subscription);
    assert_eq!(
        schema
            .execute(r#"{ __schema { mutationType { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__schema": { "mutationType": null } })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Subscription") { name } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__type": null })
    );

    let schema = Schema::new(Query, Mutation, Subscription);
    assert_eq!(
        schema
            .execute(r#"{ __schema { subscriptionType { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__schema": { "subscriptionType": null } })
    );

    let schema = Schema::new(Query, Mutation, Subscription);
    assert_eq!(
        schema
            .execute(r#"{ __schema { queryType { name } } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "__schema": { "queryType": { "name": "Query" } } })
    );
}
