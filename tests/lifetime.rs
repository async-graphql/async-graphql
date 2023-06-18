use async_graphql::*;
use static_assertions::_core::marker::PhantomData;

#[derive(SimpleObject)]
struct ObjA<'a> {
    value: &'a i32,
}

struct ObjB<'a>(PhantomData<&'a i32>);

#[Object]
#[allow(unreachable_code)]
impl<'a> ObjB<'a> {
    async fn value(&self) -> &'a i32 {
        todo!()
    }
}

#[derive(Union)]
enum MyUnion1<'a> {
    ObjA(ObjA<'a>),
}

#[derive(Interface)]
#[graphql(field(name = "value", ty = "&&'a i32"))]
enum MyInterface<'a> {
    ObjA(ObjA<'a>),
}
