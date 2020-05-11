use async_graphql::prelude::*;
use async_graphql::{EmptyMutation, EmptySubscription};

/// Is SimpleObject
#[GqlSimpleObject]
struct SimpleObject {
    /// Value a
    a: i32,

    #[field(desc = "Value b")]
    b: String,

    #[field(deprecation = "Test deprecated")]
    c: bool,
}

struct Query;

#[GqlObject]
impl Query {
    /// Get a simple object
    async fn simple_object(&self) -> SimpleObject {
        unimplemented!()
    }
}

#[async_std::test]
pub async fn test_introspection() {
    let schema = GqlSchema::new(Query, EmptyMutation, EmptySubscription);

    let res = schema
        .execute(
            r#"
    {
        __type(name: "SimpleObject") {
            kind
            name
            description
            fields(includeDeprecated: true) {
                name
                description
                args { name }
                type { name kind ofType { name kind } }
                isDeprecated
                deprecationReason
            }
            interfaces { name }
            possibleTypes { name }
            enumValues { name }
            inputFields { name }
            ofType { name }
        }
    }"#,
        )
        .await
        .unwrap()
        .data;
    assert_eq!(
        res,
        serde_json::json!({
            "__type": {
                "kind": "OBJECT",
                "name": "SimpleObject",
                "description": "Is SimpleObject",
                "fields": [
                    {
                        "name": "a",
                        "description": "Value a",
                        "args": [],
                        "type": { "name": null, "kind": "NON_NULL", "ofType": { "name": "Int", "kind": "SCALAR" } },
                        "isDeprecated": false,
                        "deprecationReason": null,
                    },
                    {
                        "name": "b",
                        "description": "Value b",
                        "args": [],
                        "type": { "name": null, "kind": "NON_NULL", "ofType": { "name": "String", "kind": "SCALAR" } },
                        "isDeprecated": false,
                        "deprecationReason": null,
                    },
                    {
                        "name": "c",
                        "description": null,
                        "args": [],
                        "type": { "name": null, "kind": "NON_NULL", "ofType": { "name": "Boolean", "kind": "SCALAR" } },
                        "isDeprecated": true,
                        "deprecationReason": "Test deprecated",
                    },
                ],
                "interfaces": [],
                "possibleTypes": null,
                "enumValues": null,
                "inputFields": null,
                "ofType": null,
            }
        })
    )
}
