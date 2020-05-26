use crate::pos::Positioned;
use crate::query::*;
use crate::utils::{unquote_string, PositionCalculator};
use crate::value::Value;
use crate::Result;
use pest::iterators::Pair;
use pest::Parser;
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "query.pest"]
struct QueryParser;

/// Parse a GraphQL query.
pub fn parse_query<T: AsRef<str>>(input: T) -> Result<Document> {
    let document_pair: Pair<Rule> = QueryParser::parse(Rule::document, input.as_ref())?
        .next()
        .unwrap();
    let mut definitions = Vec::new();
    let mut pc = PositionCalculator::new(input.as_ref());

    for pair in document_pair.into_inner() {
        match pair.as_rule() {
            Rule::named_operation_definition => definitions
                .push(parse_named_operation_definition(pair, &mut pc)?.pack(Definition::Operation)),
            Rule::selection_set => definitions.push(
                parse_selection_set(pair, &mut pc)?
                    .pack(OperationDefinition::SelectionSet)
                    .pack(Definition::Operation),
            ),
            Rule::fragment_definition => definitions
                .push(parse_fragment_definition(pair, &mut pc)?.pack(Definition::Fragment)),
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }

    Ok(Document {
        definitions,
        fragments: Default::default(),
        current_operation: None,
    })
}

fn parse_named_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<OperationDefinition>> {
    enum OperationType {
        Query,
        Mutation,
        Subscription,
    }

    let pos = pc.step(&pair);
    let mut operation_type = OperationType::Query;
    let mut name = None;
    let mut variable_definitions = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::operation_type => {
                operation_type = match pair.as_str() {
                    "query" => OperationType::Query,
                    "mutation" => OperationType::Mutation,
                    "subscription" => OperationType::Subscription,
                    _ => unreachable!(),
                };
            }
            Rule::name => {
                name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair)));
            }
            Rule::variable_definitions => {
                variable_definitions = Some(parse_variable_definitions(pair, pc)?);
            }
            Rule::directives => {
                directives = Some(parse_directives(pair, pc)?);
            }
            Rule::selection_set => {
                selection_set = Some(parse_selection_set(pair, pc)?);
            }
            _ => unreachable!(),
        }
    }

    Ok(match operation_type {
        OperationType::Query => Positioned::new(
            Query {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            pos,
        )
        .pack(OperationDefinition::Query),
        OperationType::Mutation => Positioned::new(
            Mutation {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            pos,
        )
        .pack(OperationDefinition::Mutation),
        OperationType::Subscription => Positioned::new(
            Subscription {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            pos,
        )
        .pack(OperationDefinition::Subscription),
    })
}

fn parse_default_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Value> {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => return Ok(parse_value2(pair, pc)?),
            _ => unreachable!(),
        }
    }
    unreachable!()
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

fn parse_variable_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<VariableDefinition>> {
    let pos = pc.step(&pair);
    let mut variable = None;
    let mut ty = None;
    let mut default_value = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::variable => variable = Some(parse_variable(pair, pc)?),
            Rule::type_ => {
                ty = {
                    let pos = pc.step(&pair);
                    Some(Positioned::new(parse_type(pair, pc)?, pos))
                }
            }
            Rule::default_value => {
                let pos = pc.step(&pair);
                default_value = Some(Positioned::new(parse_default_value(pair, pc)?, pos))
            }
            _ => unreachable!(),
        }
    }
    Ok(Positioned::new(
        VariableDefinition {
            name: variable.unwrap(),
            var_type: ty.unwrap(),
            default_value,
        },
        pos,
    ))
}

fn parse_variable_definitions(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<VariableDefinition>>> {
    let mut vars = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::variable_definition => vars.push(parse_variable_definition(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(vars)
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

fn parse_variable(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<String>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Ok(Positioned::new(pair.as_str().to_string(), pc.step(&pair)));
        }
    }
    unreachable!()
}

fn parse_value2(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Value> {
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::object => parse_object_value(pair, pc)?,
        Rule::array => parse_array_value(pair, pc)?,
        Rule::variable => Value::Variable(parse_variable(pair, pc)?.into_inner()),
        Rule::float => Value::Float(pair.as_str().parse().unwrap()),
        Rule::int => Value::Int(pair.as_str().parse().unwrap()),
        Rule::string => Value::String({
            let pos = pc.step(&pair);
            unquote_string(pair.as_str(), pos)?
        }),
        Rule::name => Value::Enum(pair.as_str().to_string()),
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
            Rule::value => value = Some(parse_value2(pair, pc)?),
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
            Rule::value => array.push(parse_value2(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(Value::List(array))
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
                    Some(Positioned::new(parse_value2(pair, pc)?, pos))
                }
            }
            _ => unreachable!(),
        }
    }
    Ok((name.unwrap(), value.unwrap()))
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

fn parse_alias(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<String>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Ok(Positioned::new(pair.as_str().to_string(), pc.step(&pair)));
        }
    }
    unreachable!()
}

fn parse_field(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Field>> {
    let pos = pc.step(&pair);
    let mut alias = None;
    let mut name = None;
    let mut directives = None;
    let mut arguments = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::alias => alias = Some(parse_alias(pair, pc)?),
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::arguments => arguments = Some(parse_arguments(pair, pc)?),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        Field {
            alias,
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_fragment_spread(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FragmentSpread>> {
    let pos = pc.step(&pair);
    let mut name = None;
    let mut directives = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            _ => unreachable!(),
        }
    }
    Ok(Positioned::new(
        FragmentSpread {
            fragment_name: name.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_type_condition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<TypeCondition>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            let pos = pc.step(&pair);
            return Ok(Positioned::new(
                TypeCondition::On(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
                pos,
            ));
        }
    }
    unreachable!()
}

fn parse_inline_fragment(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<InlineFragment>> {
    let pos = pc.step(&pair);
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_condition => type_condition = Some(parse_type_condition(pair, pc)?),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        InlineFragment {
            type_condition,
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
        },
        pos,
    ))
}

fn parse_selection_set(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<SelectionSet>> {
    let pos = pc.step(&pair);
    let mut items = Vec::new();
    for pair in pair.into_inner().map(|pair| pair.into_inner()).flatten() {
        match pair.as_rule() {
            Rule::field => items.push(parse_field(pair, pc)?.pack(Selection::Field)),
            Rule::fragment_spread => {
                items.push(parse_fragment_spread(pair, pc)?.pack(Selection::FragmentSpread))
            }
            Rule::inline_fragment => {
                items.push(parse_inline_fragment(pair, pc)?.pack(Selection::InlineFragment))
            }
            _ => unreachable!(),
        }
    }
    Ok(Positioned::new(SelectionSet { items }, pos))
}

fn parse_fragment_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FragmentDefinition>> {
    let pos = pc.step(&pair);
    let mut name = None;
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Positioned::new(pair.as_str().to_string(), pc.step(&pair))),
            Rule::type_condition => type_condition = Some(parse_type_condition(pair, pc)?),
            Rule::directives => directives = Some(parse_directives(pair, pc)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair, pc)?),
            _ => unreachable!(),
        }
    }

    Ok(Positioned::new(
        FragmentDefinition {
            name: name.unwrap(),
            type_condition: type_condition.unwrap(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
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
        for entry in fs::read_dir("tests/queries").unwrap() {
            if let Ok(entry) = entry {
                QueryParser::parse(Rule::document, &fs::read_to_string(entry.path()).unwrap())
                    .unwrap();
            }
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/queries").unwrap() {
            if let Ok(entry) = entry {
                parse_query(fs::read_to_string(entry.path()).unwrap()).unwrap();
            }
        }
    }
}
