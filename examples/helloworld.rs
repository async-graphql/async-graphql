use async_graphql::*;

#[async_graphql::Enum(name = "haha", desc = "hehe")]
enum MyEnum {
    A,
    B,
}

#[async_graphql::InputObject]
struct MyInputObj {
    a: i32,
    b: i32,
}

#[async_graphql::Object(auto_impl)]
struct MyObj2 {
    #[field(attr)]
    a: i32,

    #[field(attr)]
    b: i32,
}

#[async_graphql::Object(name = "haha", desc = "hehe")]
struct MyObj {
    #[field(
        attr,
        name = "a",
        arg(name = "a1", type = "i32"),
        arg(name = "a2", type = "f32")
    )]
    a: i32,

    #[field(desc = "hehe")]
    b: i32,

    #[field(arg(name = "input", type = "MyEnum"))]
    c: MyEnum,

    #[field(arg(name = "input", type = "MyInputObj"))]
    d: i32,

    #[field]
    e: &'static str,

    #[field]
    f: &'static [i32],

    #[field]
    g: &'static MyObj2,

    #[field]
    child: ChildObj,
}

#[async_graphql::Object]
struct ChildObj {
    #[field(attr)]
    value: f32,
}

#[async_trait::async_trait]
impl MyObjFields for MyObj {
    async fn a(&self, ctx: &ContextField<'_>, a1: i32, a2: f32) -> Result<i32> {
        Ok(a1 + a2 as i32)
    }

    async fn b(&self, ctx: &ContextField<'_>) -> Result<i32> {
        Ok(999)
    }

    async fn c(&self, ctx: &ContextField<'_>, input: MyEnum) -> Result<MyEnum> {
        Ok(input)
    }

    async fn d(&self, ctx: &ContextField<'_>, input: MyInputObj) -> Result<i32> {
        Ok(input.a + input.b)
    }

    async fn e(&self, ctx: &ContextField<'_>) -> Result<&'static str> {
        Ok("hehehe")
    }

    async fn f(&self, ctx: &ContextField<'_>) -> Result<&'static [i32]> {
        Ok(&[1, 2, 3, 4, 5])
    }

    async fn g(&self, ctx: &ContextField<'_>) -> Result<&'static MyObj2> {
        Ok(&MyObj2 { a: 10, b: 20 })
    }

    async fn child(&self, ctx: &ContextField<'_>) -> async_graphql::Result<ChildObj> {
        Ok(ChildObj { value: 10.0 })
    }
}

#[async_trait::async_trait]
impl ChildObjFields for ChildObj {
    async fn value(&self, ctx: &ContextField<'_>) -> Result<f32> {
        Ok(self.value)
    }
}

#[async_std::main]
async fn main() {
    let res = QueryBuilder::new(
        MyObj { a: 10 },
        GQLEmptyMutation,
        "{ b c(input:B) d(input:{a:10 b:20}) e f g { a b } }",
    )
    .execute()
    .await
    .unwrap();
    serde_json::to_writer_pretty(std::io::stdout(), &res).unwrap();
}
