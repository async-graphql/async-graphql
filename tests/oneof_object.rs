#![allow(clippy::uninlined_format_args)]

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
            .execute(r#"{ __type(name: "MyOneofObj") { name isOneOf } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": { "name": "MyOneofObj", "isOneOf": true }
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

#[tokio::test]
async fn test_oneof_object_vec() {
    use async_graphql::*;

    #[derive(SimpleObject)]
    pub struct User {
        name: String,
    }

    #[derive(OneofObject)]
    pub enum UserBy {
        Email(String),
        RegistrationNumber(i64),
    }

    pub struct Query;

    #[Object]
    impl Query {
        async fn search_users(&self, by: Vec<UserBy>) -> Vec<String> {
            by.into_iter()
                .map(|user| match user {
                    UserBy::Email(email) => email,
                    UserBy::RegistrationNumber(id) => format!("{}", id),
                })
                .collect()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let query = r#"
    {
        searchUsers(by: [
            { email: "a@a.com" },
            { registrationNumber: 100 },
            { registrationNumber: 200 },
        ])
    }
    "#;
    let data = schema.execute(query).await.into_result().unwrap().data;
    assert_eq!(
        data,
        value!({
            "searchUsers": [
                "a@a.com", "100", "200"
            ]
        })
    );
}

#[tokio::test]
async fn test_issue_923() {
    #[derive(OneofObject)]
    enum Filter {
        Any(Vec<String>),
        All(Vec<String>),
    }

    pub struct Query;

    #[Object]
    impl Query {
        async fn query(&self, filter: Filter) -> bool {
            match filter {
                Filter::Any(values) => assert_eq!(values, vec!["a".to_string(), "b".to_string()]),
                Filter::All(values) => assert_eq!(values, vec!["c".to_string(), "d".to_string()]),
            }
            true
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let query = r#"{ query(filter: {any: ["a", "b"]}) }"#;
    schema.execute(query).await.into_result().unwrap();

    let query = r#"{ query(filter: {all: ["c", "d"]}) }"#;
    schema.execute(query).await.into_result().unwrap();
}
