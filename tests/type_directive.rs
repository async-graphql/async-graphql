use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema};
use async_graphql_derive::Object;
use async_graphql_derive::TypeDirective;

#[test]
fn test_type_directive() {
    #[TypeDirective(location = "fielddefinition", location = "object")]
    fn myTestDirective(string_input: String, int_input: u32, optional_int: Option<u64>) {}

    struct Query;

    #[Object]
    impl Query {
        #[graphql(directive = "@myTestDirective(stringInput: \"abc\", intInput: 2345)")]
        pub async fn value(&self) -> &'static str {
            "abc"
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .type_directive(myTestDirective)
        .finish();

    let sdl = schema.sdl_with_options(
        SDLExportOptions::new()
            .federation()
            .enable_compose_directive(),
    );
    println!("{}", sdl)
}
