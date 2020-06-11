use async_graphql::*;

#[async_std::test]
pub async fn test_derive() {
    #[Enum(name = "MyEnum1")]
    enum MyEnum {
        #[cfg_attr(feature = "bson", item(name = "A1"))]
        A,
    }

    #[derive(GQLEnum, Eq, Copy, PartialEq, Clone)]
    #[graphql(name = "MyEnumDerive1")]
    enum MyEnumDerive {
        #[cfg_attr(feature = "bson", item(name = "A1"))]
        A,
    }

    #[InputObject(name = "MyInputObj1")]
    struct MyInputObj {
        #[cfg_attr(feature = "bson", field(default))]
        value: i32,
    }

    #[derive(GQLInputObject)]
    #[graphql(name = "MyInputObjDerive1")]
    struct MyInputObjDerive {
        #[cfg_attr(feature = "bson", field(default))]
        value: i32,
    }

    #[InputObject(name = "MySimpleObj1")]
    struct MySimpleObj {
        #[cfg_attr(feature = "bson", field(name = "value1"))]
        value: i32,
    }

    #[derive(GQLInputObject)]
    #[graphql(name = "MySimpleObjDerive1")]
    struct MySimpleObjDerive {
        #[cfg_attr(feature = "bson", field(name = "value1"))]
        value: i32,
    }
}
