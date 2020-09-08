use crate::schema::*;
use crate::utils::{unquote_string, PositionCalculator};
use crate::{Positioned, Result};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "schema.pest"]
struct SchemaParser;

/// Parse a GraphQL schema.
pub fn parse_schema<T: AsRef<str>>(input: T) -> Result<Document> {
    let document_pair: Pair<Rule> = SchemaParser::parse(Rule::document, input.as_ref())?
        .next()
        .unwrap();
    let mut definitions = Vec::new();
    let mut pc = PositionCalculator::new(input.as_ref());

    for pair in document_pair.into_inner() {
        match pair.as_rule() {
            Rule::definition => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::schema_definition => {
                            definitions.push(
                                parse_schema_definition(pair, &mut pc)?
                                    .pack(Definition::SchemaDefinition),
                            );
                        }
                        Rule::sclar_type_definition => definitions.push(
                            parse_scalar_type(pair, &mut pc)?
                                .pack(TypeDefinition::Scalar)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::object_type_definition => definitions.push(
                            parse_object_type(pair, &mut pc)?
                                .pack(TypeDefinition::Object)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::interface_type_definition => definitions.push(
                            parse_interface_type(pair, &mut pc)?
                                .pack(TypeDefinition::Interface)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::union_type_definition => definitions.push(
                            parse_union_type(pair, &mut pc)?
                                .pack(TypeDefinition::Union)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::enum_type_definition => definitions.push(
                            parse_enum_type(pair, &mut pc)?
                                .pack(TypeDefinition::Enum)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::input_type_definition => definitions.push(
                            parse_input_type(pair, &mut pc)?
                                .pack(TypeDefinition::InputObject)
                                .pack(Definition::TypeDefinition),
                        ),
                        Rule::directive_definition => definitions.push(
                            parse_directive_definition(pair, &mut pc)?
                                .pack(Definition::DirectiveDefinition),
                        ),
                        _ => unreachable!(),
                    }
                }
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    Ok(Document { definitions })
}

fn parse_schema_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<SchemaDefinition>> {
    let pos = pc.step(&pair);
    let mut extend = false;
    let mut directives = None;
    let mut query = None;
    let mut mutation = None;
    let mut subscription = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::extend => extend = true,
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::root_operation_type_definition => {
                let mut op_name = None;
                let mut ty_name = None;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::operation_type => op_name = Some(pair.as_str().to_string()),
                        Rule::name => {
                            ty_name =
                                Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
                        }
                        _ => unreachable!(),
                    }
                }
                match op_name.as_deref() {
                    Some("query") => query = ty_name,
                    Some("mutation") => mutation = ty_name,
                    Some("subscription") => subscription = ty_name,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        SchemaDefinition {
            extend,
            directives: directives.unwrap_or_default(),
            query,
            mutation,
            subscription,
        },
        pos,
    ))
}

fn parse_directive(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Directive>> {
    let pos = pc.step(&pair);
    let mut name = None;
    let mut arguments = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => {
                let pos = pc.step(&pair);
                name = Some(Positioned::new(pair.as_str().to_string(), pos))
            }
            Rule::arguments => arguments = Some(parse_arguments(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(Positioned::new(
        Directive {
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_directives(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<Directive>>> {
    let mut directives = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::directive => directives.push(parse_directive(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(directives)
}

fn parse_arguments(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<(Positioned<String>, Positioned<Value>)>> {
    let mut arguments = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => arguments.extend(std::iter::once(parse_pair(pair, pc)?)),
            _ => unreachable!(),
        }
    }
    Ok(arguments)
}

fn parse_pair(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<(Positioned<String>, Positioned<Value>)> {
    let mut name = None;
    let mut value = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::value => {
                value = {
                    let pos = pc.step(&pair);
                    Some(Positioned::new(parse_value(pair, pc)?, pos))
                }
            }
            _ => unreachable!(),
        }
    }
    Ok((name.unwrap(), value.unwrap()))
}

fn parse_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Value> {
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::object => parse_object_value(pair, pc)?,
        Rule::array => parse_array_value(pair, pc)?,
        Rule::float => Value::Float(pair.as_str().parse().unwrap()),
        Rule::int => Value::Int(pair.as_str().parse().unwrap()),
        Rule::string => Value::String({
            let pos = pc.step(&pair);
            unquote_string(pair.as_str(), pos)?
        }),
        Rule::name => Value::Enum(pair.to_string()),
        Rule::boolean => Value::Boolean(match pair.as_str() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        }),
        Rule::null => Value::Null,
        _ => unreachable!(),
    })
}

fn parse_object_pair(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<(String, Value)> {
    let mut name = None;
    let mut value = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(pair.as_str().to_string()),
            Rule::value => value = Some(parse_value(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok((name.unwrap(), value.unwrap()))
}

fn parse_object_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Value> {
    let mut map = BTreeMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => map.extend(std::iter::once(parse_object_pair(pair, pc)?)),
            _ => unreachable!(),
        }
    }
    Ok(Value::Object(map))
}

fn parse_array_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Value> {
    let mut array = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => array.push(parse_value(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(Value::List(array))
}

fn parse_scalar_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<ScalarType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut directives = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        ScalarType {
            extend,
            description,
            name: name.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_implements_interfaces(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<String>>> {
    let mut interfaces = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => {
                interfaces.push(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            _ => unreachable!(),
        }
    }
    Ok(interfaces)
}

fn parse_type(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Type> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::nonnull_type => Ok(Type::NonNull(Box::new(parse_type(pair, pc)?))),
        Rule::list_type => Ok(Type::List(Box::new(parse_type(pair, pc)?))),
        Rule::name => Ok(Type::Named(pair.as_str().to_string())),
        Rule::type_ => parse_type(pair, pc),
        _ => unreachable!(),
    }
}

fn parse_input_value_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<InputValue>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut name = None;
    let mut type_ = None;
    let mut default_value = None;
    let mut directives = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::type_ => {
                let pos = pc.step(&pair);
                type_ = Some(Positioned::new(parse_type(pair, pc)?, pos));
            }
            Rule::default_value => {
                let pos = pc.step(&pair);
                default_value = Some(Positioned::new(
                    parse_value(pair.into_inner().next().unwrap(), pc)?,
                    pos,
                ));
            }
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        InputValue {
            description,
            name: name.unwrap(),
            default_value,
            ty: type_.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_arguments_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<InputValue>>> {
    let mut arguments = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::input_value_definition => arguments.push(parse_input_value_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(arguments)
}

fn parse_field_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<Field>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut name = None;
    let mut arguments = None;
    let mut type_ = None;
    let mut directives = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::arguments_definition => arguments = Some(parse_arguments_definition(pair, pc)?),
            Rule::type_ => {
                let pos = pc.step(&pair);
                type_ = Some(Positioned::new(parse_type(pair, pc)?, pos));
            }
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        Field {
            description,
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
            ty: type_.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_fields_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<Field>>> {
    let mut fields = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::field_definition => fields.push(parse_field_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(fields)
}

fn parse_object_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<ObjectType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut implements_interfaces = None;
    let mut directives = None;
    let mut fields = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::implements_interfaces => {
                implements_interfaces = Some(parse_implements_interfaces(pair, pc)?)
            }
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::fields_definition => fields = Some(parse_fields_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        ObjectType {
            extend,
            description,
            name: name.unwrap(),
            implements_interfaces: implements_interfaces.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            fields: fields.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_interface_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<InterfaceType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut implements_interfaces = None;
    let mut directives = None;
    let mut fields = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::implements_interfaces => {
                implements_interfaces = Some(parse_implements_interfaces(pair, pc)?)
            }
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::fields_definition => fields = Some(parse_fields_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        InterfaceType {
            extend,
            description,
            name: name.unwrap(),
            implements_interfaces: implements_interfaces.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            fields: fields.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_union_members(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<String>>> {
    let mut members = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => members.push(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            _ => unreachable!(),
        }
    }
    Ok(members)
}

fn parse_union_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<UnionType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut directives = None;
    let mut members = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::union_member_types => members = Some(parse_union_members(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        UnionType {
            extend,
            description,
            name: name.unwrap(),
            directives: directives.unwrap_or_default(),
            members: members.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_enum_value(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<EnumValue>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut name = None;
    let mut directives = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(Positioned::new(
        EnumValue {
            description,
            name: name.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_enum_values(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<EnumValue>>> {
    let mut values = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::enum_value_definition => values.push(parse_enum_value(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(values)
}

fn parse_enum_type(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<EnumType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut directives = None;
    let mut values = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::enum_values_definition => values = Some(parse_enum_values(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        EnumType {
            extend,
            description,
            name: name.unwrap(),
            directives: directives.unwrap_or_default(),
            values: values.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_input_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<InputObjectType>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut extend = false;
    let mut name = None;
    let mut directives = None;
    let mut fields = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::extend => extend = true,
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::input_fields_definition => fields = Some(parse_arguments_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        InputObjectType {
            extend,
            description,
            name: name.unwrap(),
            directives: directives.unwrap_or_default(),
            fields: fields.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_directive_locations(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<DirectiveLocation>>> {
    let mut locations = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::directive_location => {
                let pos = pc.step(&pair);
                let loc = match pair.as_str() {
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
                };
                locations.push(Positioned::new(loc, pos));
            }
            _ => unreachable!(),
        }
    }
    Ok(locations)
}

fn parse_directive_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<DirectiveDefinition>> {
    let pos = pc.step(&pair);
    let mut description = None;
    let mut name = None;
    let mut arguments = None;
    let mut locations = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::string => {
                description = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)))
            }
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::arguments_definition => arguments = Some(parse_arguments_definition(pair, pc)?),
            Rule::directive_locations => locations = Some(parse_directive_locations(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        DirectiveDefinition {
            description,
            name: name.unwrap(),
            arguments: arguments.unwrap(),
            locations: locations.unwrap(),
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
        for entry in fs::read_dir("tests/schemas").unwrap() {
            if let Ok(entry) = entry {
                SchemaParser::parse(Rule::document, &fs::read_to_string(entry.path()).unwrap())
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/schemas").unwrap() {
            if let Ok(entry) = entry {
                parse_schema(fs::read_to_string(entry.path()).unwrap()).unwrap();
            }
        }
    }
}
