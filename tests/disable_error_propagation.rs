use async_graphql::*;
use futures_util::stream::{Stream, StreamExt};

#[derive(SimpleObject, Clone)]
struct Profile {
    bio: String,
}

struct Viewer;

#[Object]
impl Viewer {
    async fn id(&self) -> ID {
        ID::from("1")
    }

    async fn display_name(&self) -> Result<String> {
        Err(Error::new("Could not fetch display name."))
    }

    async fn nickname(&self) -> String {
        "Ada".to_string()
    }

    async fn profile(&self) -> Profile {
        Profile {
            bio: "hello".to_string(),
        }
    }

    async fn broken_profile(&self) -> BrokenProfile {
        BrokenProfile
    }

    async fn items(&self) -> Vec<Item> {
        vec![Item(1), Item(2), Item(3)]
    }
}

struct BrokenProfile;

#[Object]
impl BrokenProfile {
    async fn bio(&self) -> Result<String> {
        Err(Error::new("Could not fetch bio."))
    }
}

struct Item(i32);

#[Object]
impl Item {
    async fn value(&self) -> Result<i32> {
        if self.0 == 2 {
            Err(Error::new("Item 2 failed."))
        } else {
            Ok(self.0)
        }
    }
}

struct Query;

#[Object]
impl Query {
    async fn viewer(&self) -> Viewer {
        Viewer
    }

    async fn optional_viewer(&self) -> Option<Viewer> {
        Some(Viewer)
    }

    async fn value(&self) -> i32 {
        10
    }
}

struct Mutation;

#[Object]
impl Mutation {
    async fn update(&self) -> Result<i32> {
        Err(Error::new("Update failed."))
    }

    async fn other(&self) -> i32 {
        7
    }
}

struct Subscription;

#[Subscription]
impl Subscription {
    async fn events(&self) -> impl Stream<Item = Viewer> {
        futures_util::stream::iter(vec![Viewer, Viewer])
    }

    async fn numbers(&self) -> impl Stream<Item = Result<i32>> {
        futures_util::stream::iter(vec![Ok(1), Err(Error::new("boom")), Ok(3)])
    }
}

fn schema() -> Schema<Query, Mutation, Subscription> {
    Schema::build(Query, Mutation, Subscription)
        .enable_experimental_disable_error_propagation()
        .finish()
}

#[tokio::test]
async fn test_error_propagation_default_behaviour() {
    let schema = schema();
    let resp = schema
        .execute("{ viewer { id displayName nickname } }")
        .await;
    assert_eq!(resp.data, Value::Null);
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "Could not fetch display name.".to_string(),
            source: None,
            locations: vec![Pos {
                line: 1,
                column: 15
            }],
            path: vec![
                PathSegment::Field("viewer".to_owned()),
                PathSegment::Field("displayName".to_owned())
            ],
            extensions: None,
        }]
    );

    // A nullable parent absorbs the error.
    let resp = schema
        .execute("{ optionalViewer { id displayName nickname } }")
        .await;
    assert_eq!(resp.data, value!({ "optionalViewer": null }));
    assert_eq!(resp.errors.len(), 1);
}

#[tokio::test]
async fn test_disable_error_propagation_on_query() {
    let schema = schema();
    let resp = schema
        .execute(
            r#"query @experimental_disableErrorPropagation {
                viewer { id displayName nickname }
            }"#,
        )
        .await;
    assert_eq!(
        resp.data,
        value!({
            "viewer": {
                "id": "1",
                "displayName": null,
                "nickname": "Ada",
            }
        })
    );
    assert_eq!(
        resp.errors,
        vec![ServerError {
            message: "Could not fetch display name.".to_string(),
            source: None,
            locations: vec![Pos {
                line: 2,
                column: 29
            }],
            path: vec![
                PathSegment::Field("viewer".to_owned()),
                PathSegment::Field("displayName".to_owned())
            ],
            extensions: None,
        }]
    );
}

#[tokio::test]
async fn test_disable_error_propagation_nested_non_null() {
    let schema = schema();
    let resp = schema
        .execute(
            r#"query @experimental_disableErrorPropagation {
                viewer { nickname brokenProfile { bio } profile { bio } }
            }"#,
        )
        .await;
    assert_eq!(
        resp.data,
        value!({
            "viewer": {
                "nickname": "Ada",
                "brokenProfile": { "bio": null },
                "profile": { "bio": "hello" },
            }
        })
    );
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(resp.errors[0].message, "Could not fetch bio.");
    assert_eq!(
        resp.errors[0].path,
        vec![
            PathSegment::Field("viewer".to_owned()),
            PathSegment::Field("brokenProfile".to_owned()),
            PathSegment::Field("bio".to_owned()),
        ]
    );
}

#[tokio::test]
async fn test_disable_error_propagation_list_items() {
    let schema = schema();

    // Default: an error in one item of `[Item!]!` nulls out the whole `viewer`.
    let resp = schema.execute("{ viewer { items { value } } }").await;
    assert_eq!(resp.data, Value::Null);
    assert_eq!(resp.errors.len(), 1);

    // Disabled: only the errored item becomes null.
    let resp = schema
        .execute(
            r#"query @experimental_disableErrorPropagation {
                viewer { items { value } }
            }"#,
        )
        .await;
    assert_eq!(
        resp.data,
        value!({
            "viewer": {
                "items": [{ "value": 1 }, { "value": null }, { "value": 3 }],
            }
        })
    );
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(
        resp.errors[0].path,
        vec![
            PathSegment::Field("viewer".to_owned()),
            PathSegment::Field("items".to_owned()),
            PathSegment::Index(1),
            PathSegment::Field("value".to_owned()),
        ]
    );
}

#[tokio::test]
async fn test_disable_error_propagation_on_mutation() {
    let schema = schema();

    let resp = schema.execute("mutation { update other }").await;
    assert_eq!(resp.data, Value::Null);
    assert_eq!(resp.errors.len(), 1);

    let resp = schema
        .execute("mutation @experimental_disableErrorPropagation { update other }")
        .await;
    assert_eq!(resp.data, value!({ "update": null, "other": 7 }));
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(resp.errors[0].message, "Update failed.");
    assert_eq!(
        resp.errors[0].path,
        vec![PathSegment::Field("update".to_owned())]
    );
}

#[tokio::test]
async fn test_disable_error_propagation_on_subscription() {
    let schema = schema();

    // Nested non-null field errors inside an event.
    let mut stream = schema.execute_stream(
        r#"subscription @experimental_disableErrorPropagation {
            events { nickname displayName }
        }"#,
    );
    for _ in 0..2 {
        let resp = stream.next().await.unwrap();
        assert_eq!(
            resp.data,
            value!({ "events": { "nickname": "Ada", "displayName": null } })
        );
        assert_eq!(resp.errors.len(), 1);
        assert_eq!(
            resp.errors[0].path,
            vec![
                PathSegment::Field("events".to_owned()),
                PathSegment::Field("displayName".to_owned()),
            ]
        );
    }
    assert!(stream.next().await.is_none());

    // An error at the subscription root field nulls the root field itself.
    let mut stream =
        schema.execute_stream("subscription @experimental_disableErrorPropagation { numbers }");
    let resp = stream.next().await.unwrap();
    assert_eq!(resp.data, value!({ "numbers": 1 }));
    assert!(resp.errors.is_empty());

    let resp = stream.next().await.unwrap();
    assert_eq!(resp.data, value!({ "numbers": null }));
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(resp.errors[0].message, "boom");
    assert_eq!(
        resp.errors[0].path,
        vec![PathSegment::Field("numbers".to_owned())]
    );

    let resp = stream.next().await.unwrap();
    assert_eq!(resp.data, value!({ "numbers": 3 }));
    assert!(stream.next().await.is_none());

    // Without the directive, the root error yields a data-less response.
    let mut stream = schema.execute_stream("subscription { numbers }");
    stream.next().await.unwrap();
    let resp = stream.next().await.unwrap();
    assert_eq!(resp.data, Value::Null);
    assert_eq!(resp.errors.len(), 1);
}

#[tokio::test]
async fn test_disable_error_propagation_requires_opt_in() {
    let schema = Schema::new(Query, Mutation, Subscription);

    let resp = schema
        .execute("query @experimental_disableErrorPropagation { value }")
        .await;
    assert_eq!(resp.data, Value::Null);
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(
        resp.errors[0].message,
        r#"Unknown directive "experimental_disableErrorPropagation""#
    );

    // Not exposed in introspection or SDL either.
    assert!(
        !schema
            .sdl()
            .contains("experimental_disableErrorPropagation")
    );
}

#[tokio::test]
async fn test_disable_error_propagation_wrong_location() {
    let schema = schema();
    let resp = schema
        .execute("{ value @experimental_disableErrorPropagation }")
        .await;
    assert_eq!(resp.data, Value::Null);
    assert_eq!(resp.errors.len(), 1);
    assert_eq!(
        resp.errors[0].message,
        r#"Directive "experimental_disableErrorPropagation" may not be used on "FIELD""#
    );
}

#[tokio::test]
async fn test_disable_error_propagation_introspection_and_sdl() {
    let schema = schema();

    assert!(schema.sdl().contains(
        "directive @experimental_disableErrorPropagation on QUERY | MUTATION | SUBSCRIPTION"
    ));

    let resp = schema
        .execute(
            r#"{
                __schema {
                    directives { name locations }
                }
            }"#,
        )
        .await
        .into_result()
        .unwrap();
    let directives = match resp.data {
        Value::Object(mut obj) => match obj.swap_remove("__schema") {
            Some(Value::Object(mut schema)) => schema.swap_remove("directives").unwrap(),
            _ => unreachable!(),
        },
        _ => unreachable!(),
    };
    let Value::List(directives) = directives else {
        unreachable!()
    };
    assert!(directives.contains(&value!({
        "name": "experimental_disableErrorPropagation",
        "locations": ["QUERY", "MUTATION", "SUBSCRIPTION"],
    })));
}

#[cfg(feature = "dynamic-schema")]
mod dynamic {
    use async_graphql::{
        PathSegment, Value,
        dynamic::{Field, FieldFuture, FieldValue, Object, Schema, TypeRef},
        value,
    };

    fn schema(enable: bool) -> Schema {
        let item = Object::new("Item").field(Field::new(
            "value",
            TypeRef::named_nn(TypeRef::INT),
            |ctx| {
                FieldFuture::new(async move {
                    let n = *ctx.parent_value.try_downcast_ref::<i32>()?;
                    if n == 2 {
                        Err(async_graphql::Error::new("Item 2 failed."))
                    } else {
                        Ok(Some(Value::from(n)))
                    }
                })
            },
        ));

        let viewer = Object::new("Viewer")
            .field(Field::new("id", TypeRef::named_nn(TypeRef::ID), |_| {
                FieldFuture::new(async { Ok(Some(Value::from("1"))) })
            }))
            .field(Field::new(
                "displayName",
                TypeRef::named_nn(TypeRef::STRING),
                |_| {
                    FieldFuture::new(async {
                        Err::<Option<Value>, _>(async_graphql::Error::new(
                            "Could not fetch display name.",
                        ))
                    })
                },
            ))
            .field(Field::new(
                "missing",
                TypeRef::named_nn(TypeRef::STRING),
                |_| FieldFuture::new(async { Ok(None::<Value>) }),
            ))
            .field(Field::new(
                "nickname",
                TypeRef::named_nn(TypeRef::STRING),
                |_| FieldFuture::new(async { Ok(Some(Value::from("Ada"))) }),
            ))
            .field(Field::new(
                "items",
                TypeRef::named_nn_list_nn("Item"),
                |_| {
                    FieldFuture::new(async {
                        Ok(Some(FieldValue::list(
                            [1, 2, 3].into_iter().map(FieldValue::owned_any),
                        )))
                    })
                },
            ));

        let query =
            Object::new("Query").field(Field::new("viewer", TypeRef::named_nn("Viewer"), |_| {
                FieldFuture::new(async { Ok(Some(FieldValue::NULL)) })
            }));

        let builder = Schema::build(query.type_name(), None, None)
            .register(item)
            .register(viewer)
            .register(query);
        let builder = if enable {
            builder.enable_experimental_disable_error_propagation()
        } else {
            builder
        };
        builder.finish().unwrap()
    }

    #[tokio::test]
    async fn test_dynamic_disable_error_propagation() {
        let schema = schema(true);

        let resp = schema
            .execute("{ viewer { id displayName nickname } }")
            .await;
        assert_eq!(resp.data, Value::Null);
        assert_eq!(resp.errors.len(), 1);

        let resp = schema
            .execute(
                r#"query @experimental_disableErrorPropagation {
                    viewer { id displayName missing nickname items { value } }
                }"#,
            )
            .await;
        assert_eq!(
            resp.data,
            value!({
                "viewer": {
                    "id": "1",
                    "displayName": null,
                    "missing": null,
                    "nickname": "Ada",
                    "items": [{ "value": 1 }, { "value": null }, { "value": 3 }],
                }
            })
        );
        assert_eq!(resp.errors.len(), 3);
        assert_eq!(resp.errors[0].message, "Could not fetch display name.");
        assert_eq!(
            resp.errors[0].path,
            vec![
                PathSegment::Field("viewer".to_owned()),
                PathSegment::Field("displayName".to_owned()),
            ]
        );
        assert_eq!(
            resp.errors[1].path,
            vec![
                PathSegment::Field("viewer".to_owned()),
                PathSegment::Field("missing".to_owned()),
            ]
        );
        assert_eq!(resp.errors[2].message, "Item 2 failed.");
        assert_eq!(
            resp.errors[2].path,
            vec![
                PathSegment::Field("viewer".to_owned()),
                PathSegment::Field("items".to_owned()),
                PathSegment::Index(1),
                PathSegment::Field("value".to_owned()),
            ]
        );

        assert!(schema.sdl().contains(
            "directive @experimental_disableErrorPropagation on QUERY | MUTATION | SUBSCRIPTION"
        ));
    }

    #[tokio::test]
    async fn test_dynamic_disable_error_propagation_requires_opt_in() {
        let schema = schema(false);
        let resp = schema
            .execute("query @experimental_disableErrorPropagation { viewer { id } }")
            .await;
        assert_eq!(resp.data, Value::Null);
        assert_eq!(
            resp.errors[0].message,
            r#"Unknown directive "experimental_disableErrorPropagation""#
        );
        assert!(
            !schema
                .sdl()
                .contains("experimental_disableErrorPropagation")
        );
    }
}
