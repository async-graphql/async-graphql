use async_graphql::*;

#[test]
#[should_panic]
fn object() {
    mod t {
        use async_graphql::*;

        pub struct MyObj;

        #[Object]
        impl MyObj {
            async fn a(&self) -> i32 {
                1
            }
        }
    }

    struct MyObj;

    #[Object]
    impl MyObj {
        async fn b(&self) -> i32 {
            1
        }
    }

    #[derive(SimpleObject)]
    struct Query {
        a: MyObj,
        b: t::MyObj,
    }

    Schema::new(
        Query {
            a: MyObj,
            b: t::MyObj,
        },
        EmptyMutation,
        EmptySubscription,
    );
}

#[test]
#[should_panic]
fn simple_object() {
    mod t {
        use async_graphql::*;

        #[derive(SimpleObject, Default)]
        pub struct MyObj {
            a: i32,
        }
    }

    #[derive(SimpleObject, Default)]
    struct MyObj {
        a: i32,
    }

    #[derive(SimpleObject)]
    struct Query {
        a: MyObj,
        b: t::MyObj,
    }

    Schema::new(
        Query {
            a: MyObj::default(),
            b: t::MyObj::default(),
        },
        EmptyMutation,
        EmptySubscription,
    );
}

#[test]
#[should_panic]
fn merged_object() {
    mod t {
        use async_graphql::*;

        #[derive(SimpleObject, Default)]
        pub struct Query {
            a: i32,
        }
    }

    #[derive(SimpleObject, Default)]
    struct Query {
        a: i32,
    }

    #[derive(MergedObject)]
    struct QueryRoot(Query, t::Query);

    Schema::new(
        QueryRoot(Query::default(), t::Query::default()),
        EmptyMutation,
        EmptySubscription,
    );
}

#[test]
#[should_panic]
fn merged_object_root() {
    mod example {
        use async_graphql::{MergedObject, SimpleObject};

        #[derive(SimpleObject, Default)]
        pub struct Obj {
            i: i32,
        }

        #[derive(MergedObject, Default)]
        pub struct Merged(Obj);
    }

    #[derive(SimpleObject, Default)]
    pub struct Obj2 {
        j: i32,
    }

    #[derive(MergedObject, Default)]
    pub struct Merged(example::Merged, Obj2);

    Schema::new(Merged::default(), EmptyMutation, EmptySubscription);
}

#[test]
#[should_panic]
fn enum_type() {
    mod t {
        use async_graphql::*;

        #[derive(Enum, Eq, PartialEq, Copy, Clone)]
        pub enum MyEnum {
            A,
        }
    }

    #[derive(Enum, Eq, PartialEq, Copy, Clone)]
    enum MyEnum {
        B,
    }

    #[derive(SimpleObject)]
    struct Query {
        a: MyEnum,
        b: t::MyEnum,
    }

    Schema::new(
        Query {
            a: MyEnum::B,
            b: t::MyEnum::A,
        },
        EmptyMutation,
        EmptySubscription,
    );
}

#[test]
#[should_panic]
fn union() {
    mod t {
        use async_graphql::*;

        #[derive(SimpleObject, Default)]
        pub struct ObjA {
            a: i32,
        }

        #[derive(SimpleObject, Default)]
        pub struct ObjB {
            a: i32,
        }

        #[derive(SimpleObject, Default)]
        pub struct ObjC {
            a: i32,
        }

        #[derive(Union)]
        pub enum MyUnion {
            ObjA(ObjA),
            ObjB(ObjB),
        }
    }

    #[derive(Union)]
    pub enum MyUnion {
        ObjA(t::ObjA),
        ObjB(t::ObjB),
        ObjC(t::ObjC),
    }

    #[derive(SimpleObject)]
    struct Query {
        a: MyUnion,
        b: t::MyUnion,
    }

    Schema::new(
        Query {
            a: MyUnion::ObjA(t::ObjA::default()),
            b: t::MyUnion::ObjB(t::ObjB::default()),
        },
        EmptyMutation,
        EmptySubscription,
    );
}

#[test]
#[should_panic]
fn interface() {
    mod t {
        use async_graphql::*;

        #[derive(SimpleObject, Default)]
        pub struct ObjA {
            pub a: i32,
        }

        #[derive(SimpleObject, Default)]
        pub struct ObjB {
            pub a: i32,
        }

        #[derive(SimpleObject, Default)]
        pub struct ObjC {
            pub a: i32,
        }

        #[derive(Interface)]
        #[graphql(field(name = "a", ty = "&i32"))]
        pub enum MyInterface {
            ObjA(ObjA),
            ObjB(ObjB),
            ObjC(ObjC),
        }
    }

    #[derive(Interface)]
    #[graphql(field(name = "a", ty = "&i32"))]
    enum MyInterface {
        ObjA(t::ObjA),
        ObjB(t::ObjB),
    }

    #[derive(SimpleObject)]
    struct Query {
        a: MyInterface,
        b: t::MyInterface,
    }

    Schema::new(
        Query {
            a: MyInterface::ObjA(t::ObjA::default()),
            b: t::MyInterface::ObjB(t::ObjB::default()),
        },
        EmptyMutation,
        EmptySubscription,
    );
}
