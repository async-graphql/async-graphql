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
    assert!(schema_sdl.contains("fieldShareableA: Int! @shareable"),);

    assert!(schema_sdl.contains(r#"MyObjShareable @key(fields: "id") @shareable"#),);
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
    assert!(schema_sdl.contains("fieldOverrideA: Int! @override(from: \"AnotherSubgraph\")"),);
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
    #[graphql(field(name = "inaccessible_interface_value", ty = "String", inaccessible))]
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
    // no trailing spaces
    assert!(!schema_sdl.contains(" \n"));

    // compare to expected schema
    let path = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/schemas/test_entity_inaccessible.schema.graphql");
    let expected_schema = std::fs::read_to_string(&path).unwrap();
    if schema_sdl != expected_schema {
        std::fs::write(path, schema_sdl).unwrap();
        panic!("schema was not up-to-date. rerun")
    }
}

#[tokio::test]
pub async fn test_link_directive() {
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

    let schema_sdl = Schema::build(Query, EmptyMutation, EmptySubscription)
        .finish()
        .sdl_with_options(SDLExportOptions::new().federation());

    let path = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/schemas/test_fed2_link.schema.graphqls");
    let expected_schema = std::fs::read_to_string(&path).unwrap();
    if schema_sdl != expected_schema {
        std::fs::write(path, schema_sdl).unwrap();
        panic!("schema was not up-to-date. verify changes and re-run if correct.")
    }
}

#[tokio::test]
pub async fn test_entity_tag() {
    struct MyCustomObjTagged;

    #[Object(
        tag = "tagged",
        tag = "object",
        tag = "with",
        tag = "multiple",
        tag = "tags"
    )]
    impl MyCustomObjTagged {
        async fn a(&self) -> i32 {
            todo!()
        }

        #[graphql(tag = "tagged_custom_object_field")]
        async fn custom_object_tagged(&self) -> i32 {
            todo!()
        }
    }

    #[derive(SimpleObject)]
    struct MyObjFieldTagged {
        #[graphql(tag = "tagged_field")]
        obj_field_tagged_a: i32,
    }

    #[derive(SimpleObject)]
    #[graphql(tag = "tagged_simple_object")]
    struct MyObjTagged {
        a: i32,
    }

    #[derive(InputObject)]
    struct MyInputObjFieldTagged {
        #[graphql(tag = "tagged_input_object_field")]
        input_field_tagged_a: i32,
    }

    #[derive(InputObject)]
    #[graphql(tag = "input_object_tag")]
    struct MyInputObjTagged {
        a: i32,
    }

    #[derive(Enum, PartialEq, Eq, Copy, Clone)]
    enum MyEnumVariantTagged {
        #[graphql(tag = "tagged_enum_option")]
        OptionATagged,
        OptionB,
        OptionC,
    }

    #[derive(Enum, PartialEq, Eq, Copy, Clone)]
    #[graphql(tag = "tagged_num")]
    enum MyEnumTagged {
        OptionA,
        OptionB,
        OptionC,
    }

    #[derive(SimpleObject)]
    struct MyInterfaceObjA {
        tagged_interface_value: String,
    }

    #[derive(SimpleObject)]
    #[graphql(tag = "interface_object")]
    struct MyInterfaceObjB {
        tagged_interface_value: String,
    }

    #[derive(Interface)]
    #[graphql(field(
        name = "tagged_interface_value",
        ty = "String",
        tag = "tagged_interface_field"
    ))]
    #[graphql(tag = "tagged_interface")]
    enum MyInterfaceTagged {
        MyInterfaceObjA(MyInterfaceObjA),
        MyInterfaceObjB(MyInterfaceObjB),
    }

    #[derive(Union)]
    #[graphql(tag = "tagged_union")]
    enum MyUnionTagged {
        MyInterfaceObjA(MyInterfaceObjA),
        MyInterfaceObjB(MyInterfaceObjB),
    }

    struct MyNumberTagged(i32);

    #[Scalar(tag = "tagged_scalar")]
    impl ScalarType for MyNumberTagged {
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
        async fn find_obj_field_tagged(&self, _id: i32) -> MyObjFieldTagged {
            todo!()
        }

        #[graphql(entity)]
        async fn find_obj_tagged(&self, _id: i32) -> MyObjTagged {
            todo!()
        }

        async fn enum_variant_tagged(&self, _id: i32) -> MyEnumVariantTagged {
            todo!()
        }

        async fn enum_tagged(&self, _id: i32) -> MyEnumTagged {
            todo!()
        }

        #[graphql(tag = "tagged_\"field\"")]
        async fn tagged_field(&self, _id: i32) -> i32 {
            todo!()
        }

        async fn tagged_argument(&self, #[graphql(tag = "tagged_argument")] _id: i32) -> i32 {
            todo!()
        }

        async fn tagged_interface(&self) -> MyInterfaceTagged {
            todo!()
        }

        async fn tagged_union(&self) -> MyUnionTagged {
            todo!()
        }

        async fn tagged_scalar(&self) -> MyNumberTagged {
            todo!()
        }

        async fn tagged_input_field(&self, _value: MyInputObjFieldTagged) -> i32 {
            todo!()
        }

        async fn tagged_input(&self, _value: MyInputObjTagged) -> i32 {
            todo!()
        }

        async fn tagged_custom_object(&self) -> MyCustomObjTagged {
            todo!()
        }
    }

    let schema_sdl = Schema::new(Query, EmptyMutation, EmptySubscription)
        .sdl_with_options(SDLExportOptions::new().federation());

    // FIELD_DEFINITION
    assert!(schema_sdl.contains(r#"taggedField(id: Int!): Int! @tag(name: "tagged_\"field\"")"#));
    assert!(schema_sdl.contains(r#"objFieldTaggedA: Int! @tag(name: "tagged_field")"#));
    assert!(schema_sdl
        .contains(r#"taggedInterfaceValue: String! @tag(name: "tagged_interface_field")"#));
    assert!(
        schema_sdl.contains(r#"customObjectTagged: Int! @tag(name: "tagged_custom_object_field")"#)
    );
    // INTERFACE
    assert!(schema_sdl.contains(r#"interface MyInterfaceTagged @tag(name: "tagged_interface")"#));
    // OBJECT
    assert!(schema_sdl.contains(r#"type MyCustomObjTagged @tag(name: "tagged") @tag(name: "object") @tag(name: "with") @tag(name: "multiple") @tag(name: "tags") {"#));
    assert!(schema_sdl
        .contains(r#"type MyObjTagged @key(fields: "id") @tag(name: "tagged_simple_object") {"#));
    assert!(schema_sdl.contains(
        r#"type MyInterfaceObjB implements MyInterfaceTagged @tag(name: "interface_object")"#
    ));
    // UNION
    assert!(schema_sdl.contains(r#"union MyUnionTagged @tag(name: "tagged_union") ="#));
    // ARGUMENT_DEFINITION
    assert!(schema_sdl.contains(r#"taggedArgument(id: Int! @tag(name: "tagged_argument")): Int!"#));
    // SCALAR
    assert!(schema_sdl.contains(r#"scalar MyNumberTagged @tag(name: "tagged_scalar")"#));
    // ENUM
    assert!(schema_sdl.contains(r#"enum MyEnumTagged @tag(name: "tagged_num")"#));
    // ENUM_VALUE
    assert!(schema_sdl.contains(r#"OPTION_A_TAGGED @tag(name: "tagged_enum_option")"#));
    // INPUT_OBJECT
    assert!(schema_sdl.contains(r#"input MyInputObjTagged @tag(name: "input_object_tag")"#));
    // INPUT_FIELD_DEFINITION
    assert!(
        schema_sdl.contains(r#"inputFieldTaggedA: Int! @tag(name: "tagged_input_object_field")"#)
    );
    // no trailing spaces
    assert!(!schema_sdl.contains(" \n"));

    // compare to expected schema
    let path = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("tests/schemas/test_entity_tag.schema.graphql");
    let expected_schema = std::fs::read_to_string(&path).unwrap();
    if schema_sdl != expected_schema {
        std::fs::write(path, schema_sdl).unwrap();
        panic!("schema was not up-to-date. rerun")
    }
}

#[tokio::test]
pub async fn test_interface_object() {
    #[derive(SimpleObject)]
    struct VariantA {
        pub id: u64,
    }

    #[derive(Interface)]
    #[graphql(field(name = "id", ty = "&u64"))]
    enum MyInterface {
        VariantA(VariantA),
    }

    #[derive(SimpleObject)]
    #[graphql(interface_object)]
    struct MyInterfaceObject1 {
        pub id: u64,
    }

    struct MyInterfaceObject2;

    #[Object(interface_object)]
    impl MyInterfaceObject2 {
        pub async fn id(&self) -> u64 {
            todo!()
        }
    }

    struct Query;

    #[Object(extends)]
    impl Query {
        #[graphql(entity)]
        async fn my_interface(&self, _id: u64) -> MyInterface {
            todo!()
        }

        #[graphql(entity)]
        async fn my_interface_object1(&self, _id: u64) -> MyInterfaceObject1 {
            todo!()
        }

        #[graphql(entity)]
        async fn my_interface_object2(&self, _id: u64) -> MyInterfaceObject2 {
            todo!()
        }
    }

    let schema_sdl = Schema::new(Query, EmptyMutation, EmptySubscription)
        .sdl_with_options(SDLExportOptions::new().federation());

    // Interface with @key directive
    assert!(schema_sdl.contains("interface MyInterface @key(fields: \"id\")"));

    // Object with @interfaceObject directive
    assert!(schema_sdl.contains("type MyInterfaceObject1 @key(fields: \"id\") @interfaceObject"));
    assert!(schema_sdl.contains("type MyInterfaceObject2 @key(fields: \"id\") @interfaceObject"));
}

#[tokio::test]
pub async fn test_unresolvable_entity() {
    #[derive(SimpleObject)]
    struct ResolvableObject {
        id: u64,
    }

    #[derive(SimpleObject)]
    #[graphql(unresolvable = "id")]
    struct SimpleExplicitUnresolvable {
        id: u64,
    }

    #[derive(SimpleObject)]
    #[graphql(unresolvable)]
    struct SimpleImplicitUnresolvable {
        a: u64,
        #[graphql(skip)]
        _skipped: bool,
    }

    struct ExplicitUnresolvable;

    #[Object(unresolvable = "id1 id2")]
    impl ExplicitUnresolvable {
        async fn id1(&self) -> u64 {
            todo!()
        }

        async fn id2(&self) -> u64 {
            todo!()
        }
    }

    struct ImplicitUnresolvable;

    #[Object(unresolvable)]
    impl ImplicitUnresolvable {
        async fn a(&self) -> &'static str {
            todo!()
        }

        async fn b(&self) -> bool {
            todo!()
        }

        #[graphql(skip)]
        async fn _skipped(&self) {}
    }

    struct Query;

    #[Object]
    impl Query {
        async fn simple_explicit_reference(&self, _id: u64) -> SimpleExplicitUnresolvable {
            todo!()
        }

        async fn simple_implicit_reference(&self, _a: u64) -> SimpleImplicitUnresolvable {
            todo!()
        }

        async fn explicit_reference(&self, _id1: u64, _id2: u64) -> ExplicitUnresolvable {
            todo!()
        }

        async fn implicit_unresolvable(&self, _a: String, _b: bool) -> ImplicitUnresolvable {
            todo!()
        }

        #[graphql(entity)]
        async fn object_entity(&self, _id: u64) -> ResolvableObject {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let schema_sdl = schema.sdl_with_options(SDLExportOptions::new().federation());

    assert!(schema_sdl.contains(r#"type ResolvableObject @key(fields: "id")"#));
    assert!(schema_sdl
        .contains(r#"type SimpleExplicitUnresolvable @key(fields: "id", resolvable: false)"#));
    assert!(schema_sdl
        .contains(r#"type SimpleImplicitUnresolvable @key(fields: "a", resolvable: false)"#));
    assert!(schema_sdl
        .contains(r#"type ExplicitUnresolvable @key(fields: "id1 id2", resolvable: false)"#));
    assert!(
        schema_sdl.contains(r#"type ImplicitUnresolvable @key(fields: "a b", resolvable: false)"#)
    );

    let query = r#"{
            __type(name: "_Entity") { possibleTypes { name } }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        value!({
            "__type": {
                "possibleTypes": [
                    {"name": "ResolvableObject"},
                ]
            }
        })
    );
}
