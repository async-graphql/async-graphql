use async_graphql::*;
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
