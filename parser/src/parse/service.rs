use super::*;

/// Parse a GraphQL schema document.
///
/// # Errors
///
/// Fails if the schema is not a valid GraphQL document.
pub fn parse_schema<T: AsRef<str>>(input: T) -> Result<ServiceDocument> {
    let mut pc = PositionCalculator::new(input.as_ref());
    Ok(parse_service_document(
        exactly_one(GraphQLParser::parse(
            Rule::service_document,
            input.as_ref(),
        )?),
        &mut pc,
    )?)
}

fn parse_service_document(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<ServiceDocument> {
    debug_assert_eq!(pair.as_rule(), Rule::service_document);

    Ok(ServiceDocument {
        definitions: pair
            .into_inner()
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| parse_type_system_definition(pair, pc))
            .collect::<Result<_>>()?,
    })
}

fn parse_type_system_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<TypeSystemDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::type_system_definition);

    let pair = exactly_one(pair.into_inner());
    Ok(match pair.as_rule() {
        Rule::schema_definition => TypeSystemDefinition::Schema(parse_schema_definition(pair, pc)?),
        Rule::type_definition => TypeSystemDefinition::Type(parse_type_definition(pair, pc)?),
        Rule::directive_definition => {
            TypeSystemDefinition::Directive(parse_directive_definition(pair, pc)?)
        }
        _ => unreachable!(),
    })
}

fn parse_schema_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<SchemaDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::schema_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let extend = next_if_rule(&mut pairs, Rule::extend).is_some();
    let directives = parse_opt_const_directives(&mut pairs, pc)?;

    let mut query = None;
    let mut mutation = None;
    let mut subscription = None;

    for pair in pairs {
        debug_assert_eq!(pair.as_rule(), Rule::operation_type_definition);

        let mut pairs = pair.into_inner();

        let operation_type = parse_operation_type(pairs.next().unwrap(), pc)?;
        let name = parse_name(pairs.next().unwrap(), pc)?;

        match operation_type.node {
            OperationType::Query => {
                if query.is_some() {
                    return Err(operation_type.error_here("multiple query roots"));
                }
                query = Some(name);
            }
            OperationType::Mutation => {
                if mutation.is_some() {
                    return Err(operation_type.error_here("multiple mutation roots"));
                }
                mutation = Some(name);
            }
            OperationType::Subscription => {
                if subscription.is_some() {
                    return Err(operation_type.error_here("multiple subscription roots"));
                }
                subscription = Some(name);
            }
        }

        debug_assert_eq!(pairs.next(), None);
    }

    if !extend && query.is_none() {
        return Err(Error::new("missing query root", pos));
    }

    Ok(Positioned::new(
        SchemaDefinition {
            extend,
            directives,
            query,
            mutation,
            subscription,
        },
        pos,
    ))
}

fn parse_type_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<TypeDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::type_definition);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());
    let rule = pair.as_rule();
    let mut pairs = pair.into_inner();

    let description = parse_if_rule(&mut pairs, Rule::string, |pair| parse_string(pair, pc))?;
    let extend = next_if_rule(&mut pairs, Rule::extend).is_some();
    let name = parse_name(pairs.next().unwrap(), pc)?;

    let (directives, kind) = match rule {
        Rule::scalar_type => {
            let directives = parse_opt_const_directives(&mut pairs, pc)?;
            (directives, TypeKind::Scalar)
        }
        Rule::object_type => {
            let implements = parse_if_rule(&mut pairs, Rule::implements_interfaces, |pair| {
                debug_assert_eq!(pair.as_rule(), Rule::implements_interfaces);

                pair.into_inner()
                    .map(|pair| parse_name(pair, pc))
                    .collect::<Result<_>>()
            })?;

            let directives = parse_opt_const_directives(&mut pairs, pc)?;

            let fields = parse_if_rule(&mut pairs, Rule::fields_definition, |pair| {
                parse_fields_definition(pair, pc)
            })?
            .unwrap_or_default();

            (
                directives,
                TypeKind::Object(ObjectType {
                    implements: implements.unwrap_or_default(),
                    fields,
                }),
            )
        }
        Rule::interface_type => {
            let directives = parse_opt_const_directives(&mut pairs, pc)?;
            let fields = parse_if_rule(&mut pairs, Rule::fields_definition, |pair| {
                parse_fields_definition(pair, pc)
            })?
            .unwrap_or_default();
            (directives, TypeKind::Interface(InterfaceType { fields }))
        }
        Rule::union_type => {
            let directives = parse_opt_const_directives(&mut pairs, pc)?;
            let members = parse_if_rule(&mut pairs, Rule::union_member_types, |pair| {
                debug_assert_eq!(pair.as_rule(), Rule::union_member_types);

                pair.into_inner().map(|pair| parse_name(pair, pc)).collect()
            })?
            .unwrap_or_default();
            (directives, TypeKind::Union(UnionType { members }))
        }
        Rule::enum_type => {
            let directives = parse_opt_const_directives(&mut pairs, pc)?;
            let values = parse_if_rule(&mut pairs, Rule::enum_values, |pair| {
                debug_assert_eq!(pair.as_rule(), Rule::enum_values);

                pair.into_inner()
                    .map(|pair| {
                        debug_assert_eq!(pair.as_rule(), Rule::enum_value_definition);

                        let pos = pc.step(&pair);
                        let mut pairs = pair.into_inner();

                        let description =
                            parse_if_rule(&mut pairs, Rule::string, |pair| parse_string(pair, pc))?;
                        let value = parse_enum_value(pairs.next().unwrap(), pc)?;
                        let directives = parse_opt_const_directives(&mut pairs, pc)?;

                        debug_assert_eq!(pairs.next(), None);

                        Ok(Positioned::new(
                            EnumValueDefinition {
                                description,
                                value,
                                directives,
                            },
                            pos,
                        ))
                    })
                    .collect()
            })?
            .unwrap_or_default();
            (directives, TypeKind::Enum(EnumType { values }))
        }
        Rule::input_object_type => {
            let directives = parse_opt_const_directives(&mut pairs, pc)?;
            let fields = parse_if_rule(&mut pairs, Rule::input_fields_definition, |pair| {
                debug_assert_eq!(pair.as_rule(), Rule::input_fields_definition);

                pair.into_inner()
                    .map(|pair| parse_input_value_definition(pair, pc))
                    .collect()
            })?
            .unwrap_or_default();

            (
                directives,
                TypeKind::InputObject(InputObjectType { fields }),
            )
        }
        _ => unreachable!(),
    };

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        TypeDefinition {
            extend,
            description,
            name,
            directives,
            kind,
        },
        pos,
    ))
}

fn parse_fields_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<FieldDefinition>>> {
    debug_assert_eq!(pair.as_rule(), Rule::fields_definition);

    pair.into_inner()
        .map(|pair| parse_field_definition(pair, pc))
        .collect()
}

fn parse_field_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FieldDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::field_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let description = parse_if_rule(&mut pairs, Rule::string, |pair| parse_string(pair, pc))?;
    let name = parse_name(pairs.next().unwrap(), pc)?;
    let arguments = parse_if_rule(&mut pairs, Rule::arguments_definition, |pair| {
        parse_arguments_definition(pair, pc)
    })?
    .unwrap_or_default();
    let ty = parse_type(pairs.next().unwrap(), pc)?;
    let directives = parse_opt_const_directives(&mut pairs, pc)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        FieldDefinition {
            description,
            name,
            arguments,
            ty,
            directives,
        },
        pos,
    ))
}

fn parse_directive_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<DirectiveDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::directive_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let description = parse_if_rule(&mut pairs, Rule::string, |pair| parse_string(pair, pc))?;
    let name = parse_name(pairs.next().unwrap(), pc)?;
    let arguments = parse_if_rule(&mut pairs, Rule::arguments_definition, |pair| {
        debug_assert_eq!(pair.as_rule(), Rule::arguments_definition);
        pair.into_inner()
            .map(|pair| parse_input_value_definition(pair, pc))
            .collect()
    })?
    .unwrap_or_default();
    let locations = {
        let pair = pairs.next().unwrap();
        debug_assert_eq!(pair.as_rule(), Rule::directive_locations);
        pair.into_inner()
            .map(|pair| {
                let pos = pc.step(&pair);
                debug_assert_eq!(pair.as_rule(), Rule::directive_location);
                Positioned::new(
                    match pair.as_str() {
                        "QUERY" => DirectiveLocation::Query,
                        "MUTATION" => DirectiveLocation::Mutation,
                        "SUBSCRIPTION" => DirectiveLocation::Subscription,
                        "FIELD" => DirectiveLocation::Field,
                        "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
                        "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
                        "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
                        "SCHEMA" => DirectiveLocation::Schema,
                        "SCALAR" => DirectiveLocation::Scalar,
                        "OBJECT" => DirectiveLocation::Object,
                        "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
                        "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
                        "INTERFACE" => DirectiveLocation::Interface,
                        "UNION" => DirectiveLocation::Union,
                        "ENUM" => DirectiveLocation::Enum,
                        "ENUM_VALUE" => DirectiveLocation::EnumValue,
                        "INPUT_OBJECT" => DirectiveLocation::InputObject,
                        "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
                        _ => unreachable!(),
                    },
                    pos,
                )
            })
            .collect()
    };

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        DirectiveDefinition {
            description,
            name,
            arguments,
            locations,
        },
        pos,
    ))
}

fn parse_arguments_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<InputValueDefinition>>> {
    debug_assert_eq!(pair.as_rule(), Rule::arguments_definition);

    pair.into_inner()
        .map(|pair| parse_input_value_definition(pair, pc))
        .collect()
}

fn parse_input_value_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<InputValueDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::input_value_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let description = parse_if_rule(&mut pairs, Rule::string, |pair| parse_string(pair, pc))?;
    let name = parse_name(pairs.next().unwrap(), pc)?;
    let ty = parse_type(pairs.next().unwrap(), pc)?;
    let default_value = parse_if_rule(&mut pairs, Rule::default_value, |pair| {
        parse_default_value(pair, pc)
    })?;
    let directives = parse_opt_const_directives(&mut pairs, pc)?;

    Ok(Positioned::new(
        InputValueDefinition {
            description,
            name,
            ty,
            default_value,
            directives,
        },
        pos,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parser() {
        for entry in fs::read_dir("tests/services").unwrap() {
            if let Ok(entry) = entry {
                GraphQLParser::parse(
                    Rule::service_document,
                    &fs::read_to_string(entry.path()).unwrap(),
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/services").unwrap() {
            if let Ok(entry) = entry {
                parse_schema(fs::read_to_string(entry.path()).unwrap()).unwrap();
            }
        }
    }
}
