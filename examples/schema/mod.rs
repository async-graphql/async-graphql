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

pub struct MyObj {
    pub value: i32,
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
impl MyObj {
    #[field]
    async fn a(&self) -> &i32 {
        &self.value
    }

    #[field]
    async fn b(&self, #[arg(default = "123")] v: i32) -> i32 {
        v
    }

    #[field]
    async fn c(&self) -> Option<String> {
        Some(format!("**{}**", self.value))
    }
}
