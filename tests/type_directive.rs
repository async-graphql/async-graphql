use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema};
use async_graphql_derive::Object;
use async_graphql_derive::TypeDirective;

#[test]
pub fn test_type_directive() {
    #[TypeDirective(
        location = "fielddefinition",
        location = "object",
        composable = "https://custom.spec.dev/extension/v1.0"
    )]
    fn myTestDirective(string_input: String, int_input: u32, optional_int: Option<u64>) {}

    #[TypeDirective(
        location = "fielddefinition",
        composable = "https://custom.spec.dev/extension/v1.0"
    )]
    pub fn noArgsDirective() {}

    struct Query;

    #[Object]
    impl Query {
        #[graphql(directive = myTestDirective::apply("123".to_string(), 3 + 2, None))]
        pub async fn value(&self) -> &'static str {
            "abc"
        }

        #[graphql(directive = noArgsDirective::apply())]
        pub async fn value2(&self) -> u32 {
            123
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .type_directive(myTestDirective)
        .type_directive(noArgsDirective)
        .finish();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());
    println!("{}", sdl)
}
