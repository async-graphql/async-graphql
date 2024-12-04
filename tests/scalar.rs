#![allow(clippy::diverging_sub_expression)]

use async_graphql::*;

mod test_mod {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct MyValue {
        a: i32,
    }
}

scalar!(
    test_mod::MyValue,
    "MV",
    "DESC",
    "https://tools.ietf.org/html/rfc4122"
);

#[tokio::test]
pub async fn test_scalar_macro() {
    struct Query;

    #[Object]
    #[allow(unreachable_code)]
    impl Query {
        async fn value(&self) -> test_mod::MyValue {
            todo!()
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute(r#"{ __type(name:"MV") { name description specifiedByURL } }"#)
            .await
            .into_result()
            .unwrap()
            .data,
        value!({
            "__type": {
                "name": "MV",
                "description": "DESC",
                "specifiedByURL": "https://tools.ietf.org/html/rfc4122",
            }
        })
    );
}

#[tokio::test]
pub async fn test_float_inf() {
    struct Query;

    #[Object]
    impl Query {
        async fn value(&self) -> f32 {
            f32::INFINITY
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema
            .execute("{ value }")
            .await
            .into_result()
            .unwrap()
            .data,
        value!({ "value": null })
    );
}
