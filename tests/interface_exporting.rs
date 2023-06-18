use async_graphql::*;
use serde::Deserialize;

#[derive(SimpleObject)]
struct ObjectA {
    id: i32,
    title: String,
}

#[derive(SimpleObject)]
struct ObjectB {
    id: i32,
    title: String,
}

#[derive(Interface)]
#[graphql(field(name = "id", ty = "&i32"))]
enum ImplicitInterface {
    ObjectA(ObjectA),
    ObjectB(ObjectB),
}

#[derive(Interface)]
#[graphql(field(name = "title", ty = "String"))]
enum ExplicitInterface {
    ObjectA(ObjectA),
    ObjectB(ObjectB),
}

#[derive(Interface)]
#[graphql(visible = false)]
#[graphql(field(name = "title", ty = "String"))]
enum InvisibleInterface {
    ObjectA(ObjectA),
    ObjectB(ObjectB),
}

#[derive(SimpleObject)]
struct ObjectC {
    id: i32,
    title: String,
}

#[derive(Interface)]
#[graphql(field(name = "id", ty = "&i32"))]
enum UnreferencedInterface {
    ObjectC(ObjectC),
}

#[derive(Union)]
enum ObjectUnion {
    ObjectA(ObjectA),
    ObjectB(ObjectB),
}

struct Query;

#[Object]
impl Query {
    async fn implicit(&self) -> ObjectUnion {
        ObjectA {
            id: 33,
            title: "haha".to_string(),
        }
        .into()
    }

    async fn explicit(&self) -> ExplicitInterface {
        ObjectA {
            id: 40,
            title: "explicit".to_string(),
        }
        .into()
    }
}

fn build_schema() -> Schema<Query, EmptyMutation, EmptySubscription> {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_output_type::<ImplicitInterface>()
        .register_output_type::<InvisibleInterface>()
        .register_output_type::<UnreferencedInterface>()
        .finish()
}

#[tokio::test]
pub async fn test_interface_exports_interfaces_on_object_type() {
    #[derive(Deserialize)]
    struct QueryResponse {
        #[serde(rename = "__type")]
        ty: TypeResponse,
    }

    #[derive(Deserialize)]
    struct TypeResponse {
        name: String,
        kind: String,
        interfaces: Vec<InterfaceResponse>,
    }

    #[derive(Deserialize)]
    struct InterfaceResponse {
        name: String,
    }

    let schema = build_schema();

    let resp: QueryResponse = from_value(
        schema
            .execute(r#"{ __type(name: "ObjectA") { name kind interfaces { name }} }"#)
            .await
            .into_result()
            .unwrap()
            .data,
    )
    .unwrap();

    assert_eq!(resp.ty.name, "ObjectA");
    assert_eq!(resp.ty.kind, "OBJECT");
    assert!(resp
        .ty
        .interfaces
        .iter()
        .any(|i| i.name == "ExplicitInterface"));
    assert!(resp
        .ty
        .interfaces
        .iter()
        .any(|i| i.name == "ImplicitInterface"));
    assert!(!resp
        .ty
        .interfaces
        .iter()
        .any(|i| i.name == "InvisibleInterface"));
}

#[tokio::test]
pub async fn test_interface_exports_explicit_interface_type() {
    let schema = build_schema();

    let data = schema
        .execute(r#"{ __type(name: "ExplicitInterface") { name kind } }"#)
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
        data,
        value!({
            "__type": {
                "name": "ExplicitInterface",
                "kind": "INTERFACE",
            }
        })
    );
}

#[tokio::test]
pub async fn test_interface_exports_implicit_interface_type() {
    let schema = build_schema();

    let data = schema
        .execute(r#"{ __type(name: "ImplicitInterface") { name kind } }"#)
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
        data,
        value!({
            "__type": {
                "name": "ImplicitInterface",
                "kind": "INTERFACE",
            }
        })
    );
}

#[tokio::test]
pub async fn test_interface_no_export_invisible_interface_type() {
    let schema = build_schema();

    let data = schema
        .execute(r#"{ __type(name: "InvisibleInterface") { name } }"#)
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
        data,
        value!({
            "__type": null,
        })
    );
}

#[tokio::test]
pub async fn test_interface_no_export_unreferenced_interface_type() {
    let schema = build_schema();

    let data = schema
        .execute(r#"{ __type(name: "UnreferencedInterface") { name } }"#)
        .await
        .into_result()
        .unwrap()
        .data;

    assert_eq!(
        data,
        value!({
            "__type": null,
        })
    );
}
