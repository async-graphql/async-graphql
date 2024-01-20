use async_graphql::*;

#[tokio::test]
pub async fn test_schema_defaults_sdl_export() {
    struct Query;

    #[Object]
    impl Query {
        async fn sample(&self, _input: SampleInput) -> i32 {
            1
        }
    }

    #[derive(InputObject)]
    struct SampleNestedInputObject {
        #[graphql(default = "DEFAULT STRING")]
        nested_input_1: String,
        #[graphql(default = true)]
        nested_input_2: bool,
    }

    fn default_input_object() -> SampleNestedInputObject {
        SampleNestedInputObject {
            nested_input_1: "Hello".to_string(),
            nested_input_2: false,
        }
    }

    #[derive(InputObject)]
    struct SampleInput {
        required_input: i32,

        #[graphql(default = 1)]
        optional_input_defaulted: i32,

        truly_optional_input: Option<i32>,

        #[graphql(default_with = "default_input_object()")]
        optional_object_defaulted: SampleNestedInputObject,

        truly_optional_object: Option<SampleNestedInputObject>,
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    let sdl_opts = SDLExportOptions::new().sorted_arguments().sorted_fields();

    let sdl_output = schema.sdl_with_options(sdl_opts);

    let expected_output = r#"



type Query {
    sample(input: SampleInput!): Int!
}

input SampleInput {
    optionalInputDefaulted: Int = 1
    optionalObjectDefaulted: SampleNestedInputObject = {nestedInput1: "Hello",nestedInput2: false}
    requiredInput: Int!
    trulyOptionalInput: Int
    trulyOptionalObject: SampleNestedInputObject
}

input SampleNestedInputObject {
    nestedInput1: String = "DEFAULT STRING"
    nestedInput2: Boolean = true
}


schema {
    query: Query
}
"#
    .replace("    ", "\t");

    assert_eq!(sdl_output, expected_output);
}
