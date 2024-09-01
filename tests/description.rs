#![allow(clippy::diverging_sub_expression)]

use async_graphql::*;

#[tokio::test]
pub async fn test_use_type_description() {
    /// Haha
    #[doc = "next line"]
    #[derive(Description, Default)]
    struct MyObj;

    #[Object(use_type_description)]
    impl MyObj {
        async fn value(&self) -> i32 {
            100
        }
    }

    #[derive(SimpleObject, Default)]
    struct Query {
        obj: MyObj,
    }

    let schema = Schema::new(Query::default(), EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Haha\nnext line" }
        })
    );
}

#[tokio::test]
pub async fn test_use_type_external() {
    /// Wow
    #[doc = include_str!("external_descriptions/desc1.md")]
    /// More
    #[derive(Description, Default)]
    struct MyObj<'a>(&'a str);

    #[Object(use_type_description)]
    impl<'a> MyObj<'a> {
        async fn value(&self) -> &str {
            self.0
        }
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj<'_> {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Wow\nextern 1\n\nextern 2\nMore" }
        })
    );
}

#[tokio::test]
pub async fn test_use_type_external_macro() {
    macro_rules! external_doc {
        ($ident:ident) => {
            include_str!(concat!("external_descriptions/", stringify!($ident), ".md"))
        };
    }

    /// Wow
    // Simple declarative macros also work
    #[doc = external_doc!(desc1)]
    /// More
    #[derive(Description, Default)]
    struct MyObj<'a>(&'a str);

    #[Object(use_type_description)]
    impl<'a> MyObj<'a> {
        async fn value(&self) -> &str {
            self.0
        }
    }

    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn obj(&self) -> MyObj<'_> {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "MyObj") { description } }"#)
            .await
            .data,
        value!({
            "__type": { "description": "Wow\nextern 1\n\nextern 2\nMore" }
        })
    );
}

#[tokio::test]
pub async fn test_fields() {
    #[derive(SimpleObject, Default)]
    #[graphql(complex)]
    struct Obj {
        #[doc = include_str!("external_descriptions/desc1.md")]
        obj: String,
    }

    #[ComplexObject]
    impl Obj {
        #[doc = "line 1"]
        #[doc = ""]
        ///
        #[doc = "line 2"]
        ///
        #[doc = include_str!("external_descriptions/desc2.md")]
        // Make sure trailing whitespace is removed
        ///
        #[doc = ""]
        async fn obj2(&self) -> i32 {
            0
        }
    }

    let schema = Schema::new(Obj::default(), EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name: "Obj") { fields { name, description } } }"#)
            .await
            .data,
        value!({
            "__type": {
                "fields": [
                    {"name": "obj", "description": "extern 1\n\nextern 2"},
                    {"name": "obj2", "description": "line 1\n\n\nline 2\n\nexternal"}
                ]
            }
        })
    );
}
