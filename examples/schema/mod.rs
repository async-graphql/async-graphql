#[async_graphql::Enum]
pub enum MyEnum {
    A,
    B,
}

#[async_graphql::InputObject]
pub struct MyInputObj {
    #[field(default = "\"hehe\"")]
    a: i32,
    b: i32,
}

#[async_graphql::Object(
field(name = "a", type = "i32"),
field(
owned,
name = "b",
type = "i32",
arg(name = "v", type = "i32", default = "123")
),
field(owned, name = "c", type = "Option<String>")
)]
pub struct MyObj {
    pub value: i32,
}

#[async_trait::async_trait]
impl MyObjFields for MyObj {
    async fn a<'a>(&'a self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<&'a i32> {
        Ok(&self.value)
    }

    async fn b(&self, ctx: &async_graphql::Context<'_>, v: i32) -> async_graphql::Result<i32> {
        Ok(v)
    }

    async fn c(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<Option<String>> {
        Ok(Some(format!("**{}**", self.value)))
    }
}