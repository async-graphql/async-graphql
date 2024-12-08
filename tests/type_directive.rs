use async_graphql::{EmptyMutation, EmptySubscription, SDLExportOptions, Schema, Subscription};
use async_graphql_derive::{
    ComplexObject, Enum, InputObject, Interface, Object, OneofObject, SimpleObject, TypeDirective,
};
use futures_util::{stream, Stream};

#[test]
pub fn test_type_directive_1() {
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

    let expected = include_str!("schemas/test_fed2_compose.schema.graphql");
    assert_eq!(expected, &sdl)
}

#[test]
fn test_type_directive_2() {
    #[TypeDirective(location = "FieldDefinition")]
    fn type_directive_field_definition(description: String) {}

    #[TypeDirective(location = "ArgumentDefinition")]
    fn type_directive_argument_definition(description: String) {}

    #[TypeDirective(location = "InputFieldDefinition")]
    fn type_directive_input_field_definition(description: String) {}

    #[TypeDirective(location = "Object")]
    fn type_directive_object(description: String) {}

    #[TypeDirective(location = "InputObject")]
    fn type_directive_input_object(description: String) {}

    #[TypeDirective(location = "Enum")]
    fn type_directive_enum(description: String) {}

    #[TypeDirective(location = "EnumValue")]
    fn type_directive_enum_value(description: String) {}

    #[TypeDirective(location = "Interface")]
    fn type_directive_interface(description: String) {}

    #[derive(InputObject)]
    #[graphql(directive = type_directive_input_object::apply("This is INPUT_OBJECT in InputObject".to_string()))]
    struct TestInput {
        #[graphql(directive = type_directive_input_field_definition::apply("This is INPUT_FIELD_DEFINITION".to_string()))]
        field: String,
    }

    #[derive(OneofObject)]
    #[graphql(directive = type_directive_input_object::apply("This is INPUT_OBJECT in OneofObject".to_string()))]
    enum TestOneOfObject {
        #[graphql(directive = type_directive_input_field_definition::apply("This is INPUT_FIELD_DEFINITION in OneofObject".to_string()))]
        Foo(String),
        #[graphql(directive = type_directive_input_field_definition::apply("This is INPUT_FIELD_DEFINITION in OneofObject".to_string()))]
        Bar(i32),
    }

    #[derive(SimpleObject)]
    #[graphql(directive = type_directive_object::apply("This is OBJECT in SimpleObject".to_string()))]
    struct TestSimpleObject {
        #[graphql(directive = type_directive_field_definition::apply("This is FIELD_DEFINITION in SimpleObject".to_string()))]
        field: String,
    }

    #[derive(SimpleObject)]
    #[graphql(complex, directive = type_directive_object::apply("This is OBJECT in (Complex / Simple)Object".to_string()))]
    struct TestComplexObject {
        #[graphql(directive = type_directive_field_definition::apply("This is FIELD_DEFINITION in (Complex / Simple)Object".to_string()))]
        field: String,
    }

    #[ComplexObject]
    impl TestComplexObject {
        #[graphql(directive = type_directive_field_definition::apply("This is FIELD_DEFINITION in ComplexObject".to_string()))]
        async fn test(
            &self,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in ComplexObject.arg1".to_string()))]
            _arg1: String,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in ComplexObject.arg2".to_string()))]
            _arg2: String,
        ) -> &'static str {
            "test"
        }
    }

    #[derive(Enum, Copy, Clone, PartialEq, Eq)]
    #[graphql(directive = type_directive_enum::apply("This is ENUM in Enum".to_string()))]
    enum TestEnum {
        #[graphql(directive = type_directive_enum_value::apply("This is ENUM_VALUE in Enum".to_string()))]
        Foo,
        #[graphql(directive = type_directive_enum_value::apply("This is ENUM_VALUE in Enum".to_string()))]
        Bar,
    }

    struct TestObjectForInterface;

    #[Object]
    impl TestObjectForInterface {
        async fn field(
            &self,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Interface.arg1".to_string()))]
            _arg1: String,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Interface.arg2".to_string()))]
            _arg2: String,
        ) -> &'static str {
            "hello"
        }
    }
    #[derive(Interface)]
    #[graphql(
        field(
            name = "field",
            ty = "String",
            directive = type_directive_field_definition::apply("This is INTERFACE in Interface".to_string()),
            arg(
                name = "_arg1",
                ty = "String",
                directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Interface.arg1".to_string())
            ),
            arg(
                name = "_arg2",
                ty = "String",
                directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Interface.arg2".to_string())
            )
        ),
        directive = type_directive_interface::apply("This is INTERFACE in Interface".to_string())
    )]
    enum TestInterface {
        TestSimpleObjectForInterface(TestObjectForInterface),
    }

    struct Query;

    #[Object]
    impl Query {
        pub async fn test_argument(
            &self,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Object.arg1".to_string()))]
            _arg1: String,
            #[graphql(directive = type_directive_argument_definition::apply("This is ARGUMENT_DEFINITION in Object.arg2".to_string()))]
            _arg2: String,
        ) -> &'static str {
            "hello"
        }

        pub async fn test_input_object(&self, _arg: TestInput) -> &'static str {
            "hello"
        }

        pub async fn test_complex_object(&self) -> TestComplexObject {
            TestComplexObject {
                field: "hello".to_string(),
            }
        }

        pub async fn test_simple_object(&self) -> TestSimpleObject {
            TestSimpleObject {
                field: "hello".to_string(),
            }
        }

        pub async fn test_one_of_object(&self, _arg: TestOneOfObject) -> &'static str {
            "hello"
        }

        pub async fn test_enum(&self, _arg: TestEnum) -> &'static str {
            "hello"
        }

        pub async fn test_interface(&self) -> TestObjectForInterface {
            TestObjectForInterface
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_output_type::<TestInterface>()
        .finish();
    let sdl = schema.sdl();
    let expected = include_str!("schemas/test_fed2_compose_2.schema.graphql");
    assert_eq!(expected, sdl);
}
