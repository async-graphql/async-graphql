use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema, Subscription};
use async_graphql_derive::{Object, SimpleObject, TypeDirective};
use futures_util::{stream, Stream};

#[test]
pub fn test_type_directive() {
    #[TypeDirective(
        location = "FieldDefinition",
        location = "Object",
        composable = "https://custom.spec.dev/extension/v1.0"
    )]
    fn testDirective(scope: String, input: u32, opt: Option<u64>) {}

    #[TypeDirective(
        location = "FieldDefinition",
        composable = "https://custom.spec.dev/extension/v1.0"
    )]
    pub fn noArgsDirective() {}

    struct Query;

    #[derive(SimpleObject)]
    #[graphql(
        directive = testDirective::apply("simple object type".to_string(), 1, Some(3))
    )]
    struct SimpleValue {
        #[graphql(
            directive = testDirective::apply("field and param with \" symbol".to_string(), 2, Some(3))
        )]
        some_data: String,
    }

    #[Object(
        directive = testDirective::apply("object type".to_string(), 3, None),
    )]
    impl Query {
        #[graphql(
        directive = testDirective::apply("object field".to_string(), 4, None),
        directive = noArgsDirective::apply())
        ]
        async fn value(&self) -> &'static str {
            "abc"
        }

        async fn another_value(&self) -> SimpleValue {
            SimpleValue {
                some_data: "data".to_string(),
            }
        }
    }

    struct Subscription;

    #[Subscription(
        directive = testDirective::apply("object type".to_string(), 3, None),
    )]
    impl Subscription {
        #[graphql(
        directive = testDirective::apply("object field".to_string(), 4, None),
        directive = noArgsDirective::apply())
        ]
        async fn value(&self) -> impl Stream<Item = &'static str> {
            stream::iter(vec!["abc"])
        }

        async fn another_value(&self) -> impl Stream<Item = SimpleValue> {
            stream::iter(vec![SimpleValue {
                some_data: "data".to_string(),
            }])
        }
    }

    let schema = Schema::build(Query, EmptyMutation, Subscription)
        .enable_subscription_in_federation()
        .finish();

    let sdl = schema.sdl_with_options(SDLExportOptions::new().federation().compose_directive());

    println!("----");
    println!("{}", sdl);

    let expected = include_str!("schemas/test_fed2_compose.schema.graphql");
    assert_eq!(expected, &sdl)
}
