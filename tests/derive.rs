use async_graphql::*;

#[async_std::test]
pub async fn test_derive() {
    #[Enum(name = "MyEnum1")]
    enum MyEnum {
        #[cfg_attr(feature = "bson", item(name = "A1"))]
        A,
    }

    // Infers the name based on Rust name
    #[derive(GQLEnum, Eq, Copy, PartialEq, Clone)]
    enum MyEnumDerive {
        #[cfg_attr(feature = "bson", item(name = "A1"))]
        A,
    }

    // Can be renamed with graphql(name = ..) attribute
    #[derive(GQLEnum, Eq, Copy, PartialEq, Clone)]
    #[graphql(name = "MyEnumDerive")]
    enum MyEnumDeriveRenamed {
        #[cfg_attr(feature = "bson", item(name = "A1"))]
        A,
    }

    #[InputObject(name = "MyInputObj1")]
    struct MyInputObj {
        #[cfg_attr(feature = "bson", field(default))]
        value: i32,
    }

    // Infers the name based on Rust name
    #[derive(GQLInputObject)]
    struct MyInputObjDerive {
        #[cfg_attr(feature = "bson", field(default))]
        value: i32,
    }

    // Can be renamed with graphql(name = ..) attribute
    #[derive(GQLInputObject)]
    #[graphql(name = "MyInputObjDerive")]
    struct MyInputObjDeriveRenamed {
        #[cfg_attr(feature = "bson", field(default))]
        value: i32,
    }

    #[InputObject(name = "MySimpleObj1")]
    struct MySimpleObj {
        #[cfg_attr(feature = "bson", field(name = "value1"))]
        value: i32,
    }

    // Infers the name based on Rust name
    #[derive(GQLInputObject)]
    struct MySimpleObjDerive {
        #[cfg_attr(feature = "bson", field(name = "value1"))]
        value: i32,
    }

    // Can be renamed with graphql(name = ..) attribute
    #[derive(GQLInputObject)]
    #[graphql(name = "MySimpleObjDerive")]
    struct MySimpleObjDeriveRenamed {
        #[cfg_attr(feature = "bson", field(name = "value1"))]
        value: i32,
    }
}
