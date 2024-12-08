#[cfg(feature = "dynamic-schema")]
mod tests {
    use async_graphql::{
        dynamic::{
            Directive, Enum, EnumItem, Field, FieldFuture, InputObject, InputValue, Interface,
            InterfaceField, Object, ResolverContext, Scalar, Schema, SchemaError, TypeRef, Union,
        },
        Value,
    };

    fn mock_resolver_fn(_ctx: ResolverContext) -> FieldFuture {
        FieldFuture::Value(None)
    }

    pub fn schema() -> Result<Schema, SchemaError> {
        let test_enum = Enum::new("TestEnum")
            .item(EnumItem::new("A"))
            .item(EnumItem::new("B").directive(Directive::new("default")))
            .item(EnumItem::new("C"))
            .directive(Directive::new("oneOf"));

        let interface = Interface::new("TestInterface")
            .field(
                InterfaceField::new("id", TypeRef::named_nn(TypeRef::STRING))
                    .directive(Directive::new("id")),
            )
            .field(InterfaceField::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
            ))
            .directive(
                Directive::new("test")
                    .argument("a", Value::from(5))
                    .argument("b", Value::from(true))
                    .argument("c", Value::from("str")),
            );

        let output_type = Object::new("OutputType")
            .implement(interface.type_name())
            .field(
                Field::new("id", TypeRef::named_nn(TypeRef::STRING), mock_resolver_fn)
                    .directive(Directive::new("test")),
            )
            .field(Field::new(
                "name",
                TypeRef::named_nn(TypeRef::STRING),
                mock_resolver_fn,
            ))
            .field(Field::new(
                "body",
                TypeRef::named(TypeRef::STRING),
                mock_resolver_fn,
            ))
            .directive(Directive::new("type"));

        let output_type_2 = Object::new("OutputType2").field(Field::new(
            "a",
            TypeRef::named_nn_list_nn(TypeRef::INT),
            mock_resolver_fn,
        ));

        let union_type = Union::new("TestUnion")
            .possible_type(output_type.type_name())
            .possible_type(output_type_2.type_name())
            .directive(Directive::new("wrap"));

        let input_type = InputObject::new("InputType")
            .field(
                InputValue::new("a", TypeRef::named_nn(TypeRef::STRING))
                    .directive(Directive::new("input_a").argument("test", Value::from(5))),
            )
            .directive(Directive::new("a"))
            .directive(Directive::new("b"));

        let scalar = Scalar::new("TestScalar").directive(Directive::new("json"));

        let query = Object::new("Query")
            .field(
                Field::new(
                    "interface",
                    TypeRef::named_nn(interface.type_name()),
                    mock_resolver_fn,
                )
                .argument(
                    InputValue::new("x", TypeRef::named(test_enum.type_name()))
                        .directive(Directive::new("validate")),
                ),
            )
            .field(
                Field::new(
                    "output_type",
                    TypeRef::named(output_type.type_name()),
                    mock_resolver_fn,
                )
                .argument(InputValue::new(
                    "input",
                    TypeRef::named_nn(input_type.type_name()),
                )),
            )
            .field(
                Field::new(
                    "enum",
                    TypeRef::named(test_enum.type_name()),
                    mock_resolver_fn,
                )
                .argument(InputValue::new(
                    "input",
                    TypeRef::named_list_nn(test_enum.type_name()),
                ))
                .directive(Directive::new("pin")),
            )
            .field(Field::new(
                "union",
                TypeRef::named_nn(union_type.type_name()),
                mock_resolver_fn,
            ))
            .field(Field::new(
                "scalar",
                TypeRef::named(scalar.type_name()),
                mock_resolver_fn,
            ));

        Schema::build(query.type_name(), None, None)
            .register(test_enum)
            .register(interface)
            .register(input_type)
            .register(output_type)
            .register(output_type_2)
            .register(union_type)
            .register(scalar)
            .register(query)
            .finish()
    }

    #[test]
    fn test_schema_sdl() {
        let schema = schema().unwrap();
        let sdl = schema.sdl();

        let expected = include_str!("schemas/test_dynamic_schema.graphql");

        assert_eq!(sdl, expected);
    }
}
