#![allow(unreachable_code)]

use std::{collections::HashMap, convert::Infallible};

use async_graphql::{
    dataloader::{DataLoader, Loader},
    *,
};

#[tokio::test]
pub async fn test_nested_key() {
    #[derive(InputObject)]
    struct MyInputA {
        a: i32,
        b: i32,
        c: MyInputB,
    }

    #[derive(InputObject)]
    struct MyInputB {
        v: i32,
    }

    assert_eq!(MyInputB::federation_fields().as_deref(), Some("{ v }"));
    assert_eq!(
        MyInputA::federation_fields().as_deref(),
        Some("{ a b c { v } }")
    );

    struct Query;

    #[derive(SimpleObject)]
    struct MyObj {
        a: i32,
        b: i32,
        c: i32,
    }

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn find_obj(&self, input: MyInputA) -> MyObj {
            MyObj {
                a: input.a,
                b: input.b,
                c: input.c.v,
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "MyObj", input: {a: 1, b: 2, c: { v: 3 }}}]) {
                __typename
                ... on MyObj {
                    a b c
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "MyObj", "a": 1, "b": 2, "c": 3},
            ]
        })
    );
}

#[tokio::test]
pub async fn test_federation() {
    struct User {
        id: ID,
    }

    #[Object(extends)]
    impl User {
        #[graphql(external)]
        async fn id(&self) -> &ID {
            &self.id
        }

        async fn reviews(&self) -> Vec<Review> {
            todo!()
        }
    }

    struct Review;

    #[Object]
    impl Review {
        async fn body(&self) -> String {
            todo!()
        }

        async fn author(&self) -> User {
            todo!()
        }

        async fn product(&self) -> Product {
            todo!()
        }
    }

    struct Product {
        upc: String,
    }

    #[Object(extends)]
    impl Product {
        #[graphql(external)]
        async fn upc(&self) -> &str {
            &self.upc
        }

        async fn reviews(&self) -> Vec<Review> {
            todo!()
        }
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn find_user_by_id(&self, id: ID) -> User {
            User { id }
        }

        #[graphql(entity)]
        async fn find_product_by_upc(&self, upc: String) -> Product {
            Product { upc }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            _entities(representations: [{__typename: "Product", upc: "B00005N5PF"}]) {
                __typename
                ... on Product {
                    upc
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "Product", "upc": "B00005N5PF"},
            ]
        })
    );
}

#[tokio::test]
pub async fn test_find_entity_with_context() {
    struct MyLoader;

    #[async_trait::async_trait]
    impl Loader<ID> for MyLoader {
        type Value = MyObj;
        type Error = Infallible;

        async fn load(&self, keys: &[ID]) -> Result<HashMap<ID, Self::Value>, Self::Error> {
            Ok(keys
                .iter()
                .filter(|id| id.as_str() != "999")
                .map(|id| {
                    (
                        id.clone(),
                        MyObj {
                            id: id.clone(),
                            value: 999,
                        },
                    )
                })
                .collect())
        }
    }

    #[derive(Clone, SimpleObject)]
    struct MyObj {
        id: ID,
        value: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn find_user_by_id(&self, ctx: &Context<'_>, id: ID) -> FieldResult<MyObj> {
            let loader = ctx.data_unchecked::<DataLoader<MyLoader>>();
            loader
                .load_one(id)
                .await
                .unwrap()
                .ok_or_else(|| "Not found".into())
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(DataLoader::new(MyLoader, tokio::spawn))
        .finish();
    let query = r#"{
            _entities(representations: [
                {__typename: "MyObj", id: "1"},
                {__typename: "MyObj", id: "2"},
                {__typename: "MyObj", id: "3"},
                {__typename: "MyObj", id: "4"}
            ]) {
                __typename
                ... on MyObj {
                    id
                    value
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "_entities": [
                {"__typename": "MyObj", "id": "1", "value": 999 },
                {"__typename": "MyObj", "id": "2", "value": 999 },
                {"__typename": "MyObj", "id": "3", "value": 999 },
                {"__typename": "MyObj", "id": "4", "value": 999 },
            ]
        })
    );

    let query = r#"{
            _entities(representations: [
                {__typename: "MyObj", id: "999"}
            ]) {
                __typename
                ... on MyObj {
                    id
                    value
                }
            }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap_err(),
        vec![ServerError {
            message: "Not found".to_string(),
            source: None,
            locations: vec![Pos {
                line: 2,
                column: 13
            }],
            path: vec![PathSegment::Field("_entities".to_owned())],
            extensions: None,
        }]
    );
}

#[tokio::test]
pub async fn test_entity_union() {
    #[derive(SimpleObject)]
    struct MyObj {
        a: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        #[graphql(entity)]
        async fn find_obj(&self, _id: i32) -> MyObj {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"{
            __type(name: "_Entity") { possibleTypes { name } }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "__type": {
                "possibleTypes": [
                    {"name": "MyObj"},
                ]
            }
        })
    );
}

#[tokio::test]
pub async fn test_entity_shareable() {
    #[derive(SimpleObject)]
    struct MyObjFieldShareable {
        #[graphql(shareable)]
        field_shareable_a: i32,
    }

    #[derive(SimpleObject)]
    #[graphql(shareable)]
    struct MyObjShareable {
        a: i32,
    }

    struct Query;

    #[Object(extends)]
    impl Query {
        #[graphql(entity)]
        async fn find_obj_field_shareable(&self, _id: i32) -> MyObjFieldShareable {
            todo!()
        }
        #[graphql(entity)]
        async fn find_obj_shareable(&self, _id: i32) -> MyObjShareable {
            todo!()
        }
    }

    let schema_sdl = Schema::new(Query, EmptyMutation, EmptySubscription)
        .sdl_with_options(SDLExportOptions::new().federation());
    assert_eq!(
        schema_sdl.contains("fieldShareableA: Int! @shareable"),
        true
    );

    assert_eq!(
        schema_sdl.contains(r#"MyObjShareable @key(fields: "id") @shareable"#),
        true
    );
}

#[tokio::test]
pub async fn test_field_override_directive() {
    #[derive(SimpleObject)]
    struct MyObjFieldOverride {
        #[graphql(override_from = "AnotherSubgraph")]
        field_override_a: i32,
    }

    struct Query;

    #[Object(extends)]
    impl Query {
        #[graphql(entity)]
        async fn find_obj_field_override(&self, _id: i32) -> MyObjFieldOverride {
            todo!()
        }
    }

    let schema_sdl = Schema::new(Query, EmptyMutation, EmptySubscription)
        .sdl_with_options(SDLExportOptions::new().federation());
    assert_eq!(
        schema_sdl.contains("fieldOverrideA: Int! @override(from: \"AnotherSubgraph\")"),
        true
    );
}

#[tokio::test]
pub async fn test_entity_inaccessible() {
    struct MyCustomObjInaccessible;

    #[Object(inaccessible)]
    impl MyCustomObjInaccessible {
        async fn a(&self) -> i32 {
            todo!()
        }

        #[graphql(inaccessible)]
        async fn custom_object_inaccessible(&self) -> i32 {
            todo!()
        }
    }

    #[derive(SimpleObject)]
    struct MyObjFieldInaccessible {
        #[graphql(inaccessible)]
        obj_field_inaccessible_a: i32,
    }

    #[derive(SimpleObject)]
    #[graphql(inaccessible)]
    struct MyObjInaccessible {
        a: i32,
    }

    #[derive(InputObject)]
    struct MyInputObjFieldInaccessible {
        #[graphql(inaccessible)]
        input_field_inaccessible_a: i32,
    }

    #[derive(InputObject)]
    #[graphql(inaccessible)]
    struct MyInputObjInaccessible {
        a: i32,
    }

    #[derive(Enum, PartialEq, Eq, Copy, Clone)]
    enum MyEnumVariantInaccessible {
        #[graphql(inaccessible)]
        OptionAInaccessible,
        OptionB,
        OptionC,
    }

    #[derive(Enum, PartialEq, Eq, Copy, Clone)]
    #[graphql(inaccessible)]
    enum MyEnumInaccessible {
        OptionA,
        OptionB,
        OptionC,
    }

    #[derive(SimpleObject)]
    struct MyInterfaceObjA {
        inaccessible_interface_value: String,
    }

    #[derive(SimpleObject)]
    #[graphql(inaccessible)]
    struct MyInterfaceObjB {
        inaccessible_interface_value: String,
    }

    #[derive(Interface)]
    #[graphql(field(name = "inaccessible_interface_value", type = "String", inaccessible))]
    #[graphql(inaccessible)]
    enum MyInterfaceInaccessible {
        MyInterfaceObjA(MyInterfaceObjA),
        MyInterfaceObjB(MyInterfaceObjB),
    }

    #[derive(Union)]
    #[graphql(inaccessible)]
    enum MyUnionInaccessible {
        MyInterfaceObjA(MyInterfaceObjA),
        MyInterfaceObjB(MyInterfaceObjB),
    }

    struct MyNumberInaccessible(i32);

    #[Scalar(inaccessible)]
    impl ScalarType for MyNumberInaccessible {
        fn parse(_value: Value) -> InputValueResult<Self> {
            todo!()
        }

        fn to_value(&self) -> Value {
            todo!()
        }
    }

    struct Query;

    #[Object(extends)]
    impl Query {
        #[graphql(entity)]
        async fn find_obj_field_inaccessible(&self, _id: i32) -> MyObjFieldInaccessible {
            todo!()
        }

        #[graphql(entity)]
        async fn find_obj_inaccessible(&self, _id: i32) -> MyObjInaccessible {
            todo!()
        }

        async fn enum_variant_inaccessible(&self, _id: i32) -> MyEnumVariantInaccessible {
            todo!()
        }

        async fn enum_inaccessible(&self, _id: i32) -> MyEnumInaccessible {
            todo!()
        }

        #[graphql(inaccessible)]
        async fn inaccessible_field(&self, _id: i32) -> i32 {
            todo!()
        }

        async fn inaccessible_argument(&self, #[graphql(inaccessible)] _id: i32) -> i32 {
            todo!()
        }

        async fn inaccessible_interface(&self) -> MyInterfaceInaccessible {
            todo!()
        }

        async fn inaccessible_union(&self) -> MyUnionInaccessible {
            todo!()
        }

        async fn inaccessible_scalar(&self) -> MyNumberInaccessible {
            todo!()
        }

        async fn inaccessible_input_field(&self, _value: MyInputObjFieldInaccessible) -> i32 {
            todo!()
        }

        async fn inaccessible_input(&self, _value: MyInputObjInaccessible) -> i32 {
            todo!()
        }

        async fn inaccessible_custom_object(&self) -> MyCustomObjInaccessible {
            todo!()
        }
    }

    let schema_sdl = Schema::new(Query, EmptyMutation, EmptySubscription)
        .sdl_with_options(SDLExportOptions::new().federation());

    // FIELD_DEFINITION
    assert!(schema_sdl.contains("inaccessibleField(id: Int!): Int! @inaccessible"));
    assert!(schema_sdl.contains("objFieldInaccessibleA: Int! @inaccessible"));
    assert!(schema_sdl.contains("inaccessibleInterfaceValue: String! @inaccessible"));
    assert!(schema_sdl.contains("customObjectInaccessible: Int! @inaccessible"));
    // INTERFACE
    assert!(schema_sdl.contains("interface MyInterfaceInaccessible @inaccessible"));
    // OBJECT
    assert!(schema_sdl.contains("type MyCustomObjInaccessible @inaccessible"));
    assert!(schema_sdl.contains(r#"type MyObjInaccessible @key(fields: "id") @inaccessible"#));
    assert!(schema_sdl
        .contains("type MyInterfaceObjB implements MyInterfaceInaccessible @inaccessible"));
    // UNION
    assert!(schema_sdl.contains("union MyUnionInaccessible @inaccessible ="));
    // ARGUMENT_DEFINITION
    assert!(schema_sdl.contains("inaccessibleArgument(id: Int! @inaccessible): Int!"));
    // SCALAR
    assert!(schema_sdl.contains("scalar MyNumberInaccessible @inaccessible"));
    // ENUM
    assert!(schema_sdl.contains("enum MyEnumInaccessible @inaccessible"));
    // ENUM_VALUE
    assert!(schema_sdl.contains("OPTION_A_INACCESSIBLE @inaccessible"));
    // INPUT_OBJECT
    assert!(schema_sdl.contains("input MyInputObjInaccessible @inaccessible"));
    // INPUT_FIELD_DEFINITION
    assert!(schema_sdl.contains("inputFieldInaccessibleA: Int! @inaccessible"));
}
