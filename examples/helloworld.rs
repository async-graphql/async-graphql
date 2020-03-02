#[async_graphql::Object(
    field(name = "a", type = "i32"),
    field(owned, name = "b", type = "i32"),
    field(owned, name = "c", type = "Option<String>")
)]
struct MyObj {
    value: i32,
}

#[async_trait::async_trait]
impl MyObjFields for MyObj {
    async fn a<'a>(
        &'a self,
        ctx: &async_graphql::ContextField<'_>,
    ) -> async_graphql::Result<&'a i32> {
        Ok(&self.value)
    }

    async fn b(&self, ctx: &async_graphql::ContextField<'_>) -> async_graphql::Result<i32> {
        Ok(999)
    }

    async fn c(
        &self,
        ctx: &async_graphql::ContextField<'_>,
    ) -> async_graphql::Result<Option<String>> {
        Ok(Some(format!("**{}**", self.value)))
    }
}

#[async_std::main]
async fn main() {
    let res = async_graphql::QueryBuilder::new(
        MyObj { value: 100 },
        async_graphql::GQLEmptyMutation,
        "{ a b c }",
    )
    .execute()
    .await
    .unwrap();
    serde_json::to_writer_pretty(std::io::stdout(), &res).unwrap();
}
