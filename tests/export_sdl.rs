use async_graphql::*;

#[tokio::test]
async fn test_spaces() {
    #[derive(SimpleObject)]
    struct A {
        a: i32,
        b: i32,
    }

    #[derive(SimpleObject)]
    struct B {
        a: A,
        b: i32,
    }

    struct Query;

    #[Object]
    impl Query {
        async fn obj(&self) -> B {
            B {
                a: A { a: 100, b: 200 },
                b: 300,
            }
        }

        async fn a(&self) -> A {
            A { a: 100, b: 200 }
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    let sdl = schema.sdl_with_options(
        SDLExportOptions::new()
            .use_space_ident()
            .indent_width(2)
            .sorted_fields()
            .sorted_enum_items(),
    );
    std::fs::write("./test_space_schema.graphql", &sdl).unwrap();

    let expected = include_str!("schemas/test_space_schema.graphql");
    assert_eq!(sdl, expected);
}
