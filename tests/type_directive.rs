use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema};
use async_graphql_derive::Object;
use async_graphql_derive::TypeDirective;

#[test]
pub fn test_type_directive() {
    #[TypeDirective(location = "fielddefinition", location = "object")]
    fn myTestDirective(_string_input: String, _int_input: u32, _optional_int: Option<u64>) {}

    struct Query;

    #[Object]
    impl Query {
        #[graphql(directive = myTestDirective::apply("123".to_string(), 3 + 2, Some(4)))]
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
