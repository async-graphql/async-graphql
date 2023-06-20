use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema};
use async_graphql_derive::Object;
use async_graphql_derive::TypeDirective;

#[test]
fn test_type_directive() {
    #[TypeDirective(location = "fielddefinition")]
    fn my_test_directive(string_input: String, int_input: u32) {}

    struct Query;

    #[Object]
    impl Query {
        pub async fn value(&self) -> &'static str {
            "abc"
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .type_directive(my_test_directive)
        .finish();

    let sdl = schema.sdl_with_options(
        SDLExportOptions::new()
            .federation()
            .enable_compose_directive(),
    );
    println!("{}", sdl)
}
