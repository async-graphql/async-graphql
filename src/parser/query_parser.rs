use crate::parser::ast::*;
use crate::parser::span::Spanned;
use crate::parser::value::Value;
use crate::{Pos, Result};
use pest::iterators::Pair;
use pest::{error::Error, Parser};
use std::collections::BTreeMap;

#[derive(Parser)]
#[grammar = "parser/query.pest"]
struct QueryParser;

pub type ParseError = Error<Rule>;

pub fn parse_query<T: AsRef<str>>(input: T) -> Result<Document> {
    let document_pair: Pair<Rule> = QueryParser::parse(Rule::document, input.as_ref())?
        .next()
        .unwrap();
    let mut definitions = Vec::new();

    for pair in document_pair.into_inner() {
        match pair.as_rule() {
            Rule::named_operation_definition => definitions
                .push(parse_named_operation_definition(pair)?.pack(Definition::Operation)),
            Rule::selection_set => definitions.push(
                parse_selection_set(pair)?
                    .pack(OperationDefinition::SelectionSet)
                    .pack(Definition::Operation),
            ),
            Rule::fragment_definition => {
                definitions.push(parse_fragment_definition(pair)?.pack(Definition::Fragment))
            }
            Rule::EOI => {}
            _ => unreachable!(),
        }
    }
    Ok(Document { definitions })
}

fn parse_named_operation_definition(pair: Pair<Rule>) -> Result<Spanned<OperationDefinition>> {
    enum OperationType {
        Query,
        Mutation,
        Subscription,
    }

    let span = pair.as_span();
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
                name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span()));
            }
            Rule::variable_definitions => {
                variable_definitions = Some(parse_variable_definitions(pair)?);
            }
            Rule::directives => {
                directives = Some(parse_directives(pair)?);
            }
            Rule::selection_set => {
                selection_set = Some(parse_selection_set(pair)?);
            }
            _ => unreachable!(),
        }
    }

    Ok(match operation_type {
        OperationType::Query => Spanned::new(
            Query {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(OperationDefinition::Query),
        OperationType::Mutation => Spanned::new(
            Mutation {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(OperationDefinition::Mutation),
        OperationType::Subscription => Spanned::new(
            Subscription {
                name,
                variable_definitions: variable_definitions.unwrap_or_default(),
                directives: directives.unwrap_or_default(),
                selection_set: selection_set.unwrap(),
            },
            span,
        )
        .pack(OperationDefinition::Subscription),
    })
}

fn parse_default_value(pair: Pair<Rule>) -> Result<Value> {
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => return Ok(parse_value(pair)?),
            _ => unreachable!(),
        }
    }
    unreachable!()
}

fn parse_type(pair: Pair<Rule>) -> Result<Type> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::nonnull_type => Ok(Type::NonNull(Box::new(parse_type(pair)?))),
        Rule::list_type => Ok(Type::List(Box::new(parse_type(pair)?))),
        Rule::name => Ok(Type::Named(pair.as_str().to_string())),
        Rule::type_ => parse_type(pair),
        _ => unreachable!(),
    }
}

fn parse_variable_definition(pair: Pair<Rule>) -> Result<Spanned<VariableDefinition>> {
    let span = pair.as_span();
    let mut variable = None;
    let mut ty = None;
    let mut default_value = None;

    for pair in pair.into_inner() {
        let span = pair.as_span();
        match pair.as_rule() {
            Rule::variable => variable = Some(parse_variable(pair)?),
            Rule::type_ => ty = Some(Spanned::new(parse_type(pair)?, span)),
            Rule::default_value => {
                default_value = Some(Spanned::new(parse_default_value(pair)?, span))
            }
            _ => unreachable!(),
        }
    }
    Ok(Spanned::new(
        VariableDefinition {
            name: variable.unwrap(),
            var_type: ty.unwrap(),
            default_value,
        },
        span,
    ))
}

fn parse_variable_definitions(pair: Pair<Rule>) -> Result<Vec<Spanned<VariableDefinition>>> {
    let mut vars = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::variable_definition => vars.push(parse_variable_definition(pair)?),
            _ => unreachable!(),
        }
    }
    Ok(vars)
}

fn parse_directive(pair: Pair<Rule>) -> Result<Spanned<Directive>> {
    let span = pair.as_span();
    let mut name = None;
    let mut arguments = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::arguments => arguments = Some(parse_arguments(pair)?),
            _ => unreachable!(),
        }
    }
    Ok(Spanned::new(
        Directive {
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
        },
        span,
    ))
}

fn parse_directives(pair: Pair<Rule>) -> Result<Vec<Spanned<Directive>>> {
    let mut directives = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::directive => directives.push(parse_directive(pair)?),
            _ => unreachable!(),
        }
    }
    Ok(directives)
}

fn parse_variable(pair: Pair<Rule>) -> Result<Spanned<String>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Ok(Spanned::new(pair.as_str().to_string(), pair.as_span()));
        }
    }
    unreachable!()
}

fn parse_value(pair: Pair<Rule>) -> Result<Value> {
    let pair = pair.into_inner().next().unwrap();
    Ok(match pair.as_rule() {
        Rule::object => parse_object_value(pair)?,
        Rule::array => parse_array_value(pair)?,
        Rule::variable => Value::Variable(parse_variable(pair)?.into_inner()),
        Rule::float => Value::Float(pair.as_str().parse().unwrap()),
        Rule::int => Value::Int(pair.as_str().parse().unwrap()),
        Rule::string => Value::String({
            let start_pos = pair.as_span().start_pos().line_col();
            unquote_string(
                pair.as_str(),
                Pos {
                    line: start_pos.0,
                    column: start_pos.1,
                },
            )?
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

fn parse_object_pair(pair: Pair<Rule>) -> Result<(String, Value)> {
    let mut name = None;
    let mut value = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(pair.as_str().to_string()),
            Rule::value => value = Some(parse_value(pair)?),
            _ => unreachable!(),
        }
    }
    Ok((name.unwrap(), value.unwrap()))
}

fn parse_object_value(pair: Pair<Rule>) -> Result<Value> {
    let mut map = BTreeMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => {
                map.extend(std::iter::once(parse_object_pair(pair)?));
            }
            _ => unreachable!(),
        }
    }
    Ok(Value::Object(map))
}

fn parse_array_value(pair: Pair<Rule>) -> Result<Value> {
    let mut array = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::value => {
                array.push(parse_value(pair)?);
            }
            _ => unreachable!(),
        }
    }
    Ok(Value::List(array))
}

fn parse_pair(pair: Pair<Rule>) -> Result<(Spanned<String>, Spanned<Value>)> {
    let mut name = None;
    let mut value = None;
    for pair in pair.into_inner() {
        let span = pair.as_span();
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), span)),
            Rule::value => value = Some(Spanned::new(parse_value(pair)?, span)),
            _ => unreachable!(),
        }
    }
    Ok((name.unwrap(), value.unwrap()))
}

fn parse_arguments(pair: Pair<Rule>) -> Result<Vec<(Spanned<String>, Spanned<Value>)>> {
    let mut arguments = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::pair => arguments.extend(std::iter::once(parse_pair(pair)?)),
            _ => unreachable!(),
        }
    }
    Ok(arguments)
}

fn parse_alias(pair: Pair<Rule>) -> Result<Spanned<String>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Ok(Spanned::new(pair.as_str().to_string(), pair.as_span()));
        }
    }
    unreachable!()
}

fn parse_field(pair: Pair<Rule>) -> Result<Spanned<Field>> {
    let span = pair.as_span();
    let mut alias = None;
    let mut name = None;
    let mut directives = None;
    let mut arguments = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::alias => alias = Some(parse_alias(pair)?),
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::arguments => arguments = Some(parse_arguments(pair)?),
            Rule::directives => directives = Some(parse_directives(pair)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)?),
            _ => unreachable!(),
        }
    }

    Ok(Spanned::new(
        Field {
            alias,
            name: name.unwrap(),
            arguments: arguments.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap_or_default(),
        },
        span,
    ))
}

fn parse_fragment_spread(pair: Pair<Rule>) -> Result<Spanned<FragmentSpread>> {
    let span = pair.as_span();
    let mut name = None;
    let mut directives = None;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::directives => directives = Some(parse_directives(pair)?),
            _ => unreachable!(),
        }
    }
    Ok(Spanned::new(
        FragmentSpread {
            fragment_name: name.unwrap(),
            directives: directives.unwrap_or_default(),
        },
        span,
    ))
}

fn parse_type_condition(pair: Pair<Rule>) -> Result<Spanned<TypeCondition>> {
    for pair in pair.into_inner() {
        if let Rule::name = pair.as_rule() {
            return Ok(Spanned::new(
                TypeCondition::On(Spanned::new(pair.as_str().to_string(), pair.as_span())),
                pair.as_span(),
            ));
        }
    }
    unreachable!()
}

fn parse_inline_fragment(pair: Pair<Rule>) -> Result<Spanned<InlineFragment>> {
    let span = pair.as_span();
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::type_condition => type_condition = Some(parse_type_condition(pair)?),
            Rule::directives => directives = Some(parse_directives(pair)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)?),
            _ => unreachable!(),
        }
    }

    Ok(Spanned::new(
        InlineFragment {
            type_condition,
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
        },
        span,
    ))
}

fn parse_selection_set(pair: Pair<Rule>) -> Result<Spanned<SelectionSet>> {
    let span = pair.as_span();
    let mut items = Vec::new();
    for pair in pair.into_inner().map(|pair| pair.into_inner()).flatten() {
        match pair.as_rule() {
            Rule::field => items.push(parse_field(pair)?.pack(Selection::Field)),
            Rule::fragment_spread => {
                items.push(parse_fragment_spread(pair)?.pack(Selection::FragmentSpread))
            }
            Rule::inline_fragment => {
                items.push(parse_inline_fragment(pair)?.pack(Selection::InlineFragment))
            }
            _ => unreachable!(),
        }
    }
    Ok(Spanned::new(SelectionSet { items }, span))
}

fn parse_fragment_definition(pair: Pair<Rule>) -> Result<Spanned<FragmentDefinition>> {
    let span = pair.as_span();
    let mut name = None;
    let mut type_condition = None;
    let mut directives = None;
    let mut selection_set = None;

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::name => name = Some(Spanned::new(pair.as_str().to_string(), pair.as_span())),
            Rule::type_condition => type_condition = Some(parse_type_condition(pair)?),
            Rule::directives => directives = Some(parse_directives(pair)?),
            Rule::selection_set => selection_set = Some(parse_selection_set(pair)?),
            _ => unreachable!(),
        }
    }

    Ok(Spanned::new(
        FragmentDefinition {
            name: name.unwrap(),
            type_condition: type_condition.unwrap(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap(),
        },
        span,
    ))
}

fn unquote_string(s: &str, pos: Pos) -> Result<String> {
    let mut res = String::with_capacity(s.len());
    debug_assert!(s.starts_with('"') && s.ends_with('"'));
    let mut chars = s[1..s.len() - 1].chars();
    let mut temp_code_point = String::with_capacity(4);
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                match chars.next().expect("slash cant be at the end") {
                    c @ '"' | c @ '\\' | c @ '/' => res.push(c),
                    'b' => res.push('\u{0010}'),
                    'f' => res.push('\u{000C}'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => {
                        temp_code_point.clear();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(inner_c) => temp_code_point.push(inner_c),
                                None => {
                                    return Err(crate::Error::Parse {
                                        line: pos.line,
                                        column: pos.column,
                                        message: format!(
                                            "\\u must have 4 characters after it, only found '{}'",
                                            temp_code_point
                                        ),
                                    });
                                }
                            }
                        }

                        // convert our hex string into a u32, then convert that into a char
                        match u32::from_str_radix(&temp_code_point, 16).map(std::char::from_u32) {
                            Ok(Some(unicode_char)) => res.push(unicode_char),
                            _ => {
                                return Err(crate::Error::Parse {
                                    line: pos.line,
                                    column: pos.column,
                                    message: format!(
                                        "{} is not a valid unicode code point",
                                        temp_code_point
                                    ),
                                });
                            }
                        }
                    }
                    c => {
                        return Err(crate::Error::Parse {
                            line: pos.line,
                            column: pos.column,
                            message: format!("bad escaped char {:?}", c),
                        });
                    }
                }
            }
            c => res.push(c),
        }
    }

    Ok(res)
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
