use async_graphql::*;

mod test_mod {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct MyValue {
        a: i32,
    }
}

#[tokio::test]
pub async fn test_scalar_macro() {
    scalar!(
        test_mod::MyValue,
        "MV",
        "DESC",
        "https://tools.ietf.org/html/rfc4122"
    );

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
