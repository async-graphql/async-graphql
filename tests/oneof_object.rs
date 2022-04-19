use async_graphql::{
    registry::{MetaType, Registry},
    *,
};

#[tokio::test]
async fn test_oneof_object() {
    #[derive(Debug, InputObject, PartialEq)]
    struct MyInput {
        a: i32,
        b: String,
    }

    #[derive(Debug, OneofObject, PartialEq)]
    enum MyOneofObj {
        A(i32),
        B(MyInput),
    }

    assert_eq!(
        MyOneofObj::parse(Some(value!({
            "a": 100,
        })))
        .unwrap(),
        MyOneofObj::A(100)
    );

    assert_eq!(
        MyOneofObj::A(100).to_value(),
        value!({
            "a": 100,
        })
    );

    assert_eq!(
        MyOneofObj::parse(Some(value!({
            "b": {
                "a": 200,
                "b": "abc",
            },
        })))
        .unwrap(),
        MyOneofObj::B(MyInput {
            a: 200,
            b: "abc".to_string()
        })
    );

    assert_eq!(
        MyOneofObj::B(MyInput {
            a: 200,
            b: "abc".to_string()
        })
        .to_value(),
        value!({
            "b": {
                "a": 200,
                "b": "abc",
            },
        })
    );

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, obj: MyOneofObj) -> String {
            match obj {
                MyOneofObj::A(value) => format!("a:{}", value),
                MyOneofObj::B(MyInput { a, b }) => format!("b:{}/{}", a, b),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ test(obj: {a: 100}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": "a:100"
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ test(obj: {b: {a: 200, b: "abc"}}) }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": "b:200/abc"
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyOneofObj") { name oneOf } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": { "name": "MyOneofObj", "oneOf": true }
        })
    );
}

#[tokio::test]
async fn test_oneof_object_concrete() {
    #[derive(Debug, OneofObject, PartialEq)]
    #[graphql(
        concrete(name = "MyObjI32", params(i32)),
        concrete(name = "MyObjString", params(String))
    )]
    enum MyObj<T: InputType> {
        A(i32),
        B(T),
    }

    assert_eq!(MyObj::<i32>::type_name(), "MyObjI32");
    assert_eq!(MyObj::<String>::type_name(), "MyObjString");

    assert_eq!(
        MyObj::<String>::parse(Some(value!({
            "a": 100,
        })))
        .unwrap(),
        MyObj::A(100)
    );

    assert_eq!(
        MyObj::<i32>::A(100).to_value(),
        value!({
            "a": 100,
        })
    );

    assert_eq!(
        MyObj::<String>::B("abc".to_string()).to_value(),
        value!({
            "b": "abc",
        })
    );

    struct Query;

    #[Object]
    impl Query {
        async fn test(&self, obj: MyObj<String>) -> String {
            match obj {
                MyObj::A(value) => format!("a:{}", value),
                MyObj::B(value) => format!("b:{}", value),
            }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        schema
            .execute("{ test(obj: {a: 100}) }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": "a:100"
        })
    );

    assert_eq!(
        schema
            .execute(r#"{ test(obj: {b: "abc"}) }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "test": "b:abc"
        })
    );
}

#[tokio::test]
async fn test_oneof_object_rename_fields() {
    #[derive(OneofObject)]
    #[graphql(rename_fields = "lowercase")]
    enum MyInput {
        Name(i32),
        CreateAt(String),
    }

    let mut registry = Registry::default();
    MyInput::create_type_info(&mut registry);

    let ty: &MetaType = registry.types.get("MyInput").unwrap();
    match ty {
        MetaType::InputObject { input_fields, .. } => {
            assert_eq!(
                input_fields.keys().collect::<Vec<_>>(),
                vec!["name", "createat"]
            );
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn test_oneof_object_rename_field() {
    #[derive(OneofObject)]
    enum MyInput {
        Name(i32),
        #[graphql(name = "create_At")]
        CreateAt(String),
    }

    let mut registry = Registry::default();
    MyInput::create_type_info(&mut registry);

    let ty: &MetaType = registry.types.get("MyInput").unwrap();
    match ty {
        MetaType::InputObject { input_fields, .. } => {
            assert_eq!(
                input_fields.keys().collect::<Vec<_>>(),
                vec!["name", "create_At"]
            );
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
async fn test_oneof_object_validation() {
    #[derive(Debug, OneofObject, PartialEq)]
    enum MyOneofObj {
        #[graphql(validator(maximum = 10))]
        A(i32),
        #[graphql(validator(max_length = 3))]
        B(String),
    }

    assert_eq!(
        MyOneofObj::parse(Some(value!({
            "a": 5,
        })))
        .unwrap(),
        MyOneofObj::A(5)
    );

    assert_eq!(
        MyOneofObj::parse(Some(value!({
            "a": 20,
        })))
        .unwrap_err()
        .into_server_error(Default::default())
        .message,
        r#"Failed to parse "Int": the value is 20, must be less than or equal to 10 (occurred while parsing "MyOneofObj")"#
    );
}
