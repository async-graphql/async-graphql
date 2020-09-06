use crate::pos::Positioned;
use crate::types::*;
use crate::utils::{block_string_value, string_value, PositionCalculator};
use crate::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "graphql.pest"]
struct GraphQLParser;

/// Parse a GraphQL query.
///
/// # Errors
///
/// Fails if the query is not a valid GraphQL document.
pub fn parse_query<T: AsRef<str>>(input: T) -> Result<Document> {
    let mut pc = PositionCalculator::new(input.as_ref());
    Ok(parse_document(
        exactly_one(GraphQLParser::parse(Rule::document, input.as_ref())?),
        &mut pc,
    ))
}

fn parse_document(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Document {
    debug_assert_eq!(pair.as_rule(), Rule::document);

    Document {
        definitions: pair
            .into_inner()
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| parse_definition(pair, pc))
            .collect(),
    }
}

fn parse_definition(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Definition {
    debug_assert_eq!(pair.as_rule(), Rule::definition);

    let pair = exactly_one(pair.into_inner());
    match pair.as_rule() {
        Rule::operation_definition => Definition::Operation(parse_operation_definition(pair, pc)),
        Rule::fragment_definition => Definition::Fragment(parse_fragment_definition(pair, pc)),
        _ => unreachable!(),
    }
}

fn parse_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<OperationDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::operation_definition);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());
    Positioned::new(
        match pair.as_rule() {
            Rule::named_operation_definition => parse_named_operation_definition(pair, pc),
            Rule::selection_set => OperationDefinition {
                ty: OperationType::Query,
                name: None,
                variable_definitions: Vec::new(),
                directives: Vec::new(),
                selection_set: parse_selection_set(pair, pc),
            },
            _ => unreachable!(),
        },
        pos,
    )
}

fn parse_named_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> OperationDefinition {
    debug_assert_eq!(pair.as_rule(), Rule::named_operation_definition);

    let mut pairs = pair.into_inner();

    let ty = parse_operation_type(&pairs.next().unwrap(), pc);
    let name = next_if_rule(&mut pairs, Rule::name).map(|pair| parse_name(&pair, pc));
    let variable_definitions = next_if_rule(&mut pairs, Rule::variable_definitions)
        .map(|pair| parse_variable_definitions(pair, pc));
    let directives =
        next_if_rule(&mut pairs, Rule::directives).map(|pair| parse_directives(pair, pc));
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc);

    debug_assert_eq!(pairs.next(), None);

    OperationDefinition {
        ty: ty.node,
        name,
        variable_definitions: variable_definitions.unwrap_or_default(),
        directives: directives.unwrap_or_default(),
        selection_set,
    }
}

fn parse_operation_type(
    pair: &Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<OperationType> {
    debug_assert_eq!(pair.as_rule(), Rule::operation_type);

    let pos = pc.step(&pair);

    Positioned::new(
        match pair.as_str() {
            "query" => OperationType::Query,
            "mutation" => OperationType::Mutation,
            "subscription" => OperationType::Subscription,
            _ => unreachable!(),
        },
        pos,
    )
}

fn parse_variable_definitions(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Vec<Positioned<VariableDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::variable_definitions);

    pair.into_inner()
        .map(|pair| parse_variable_definition(pair, pc))
        .collect()
}

fn parse_variable_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<VariableDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::variable_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let variable = parse_variable(pairs.next().unwrap(), pc);
    let var_type = parse_type(&pairs.next().unwrap(), pc);
    let default_value = pairs.next().map(|pair| parse_default_value(pair, pc));

    debug_assert_eq!(pairs.peek(), None);

    Positioned::new(
        VariableDefinition {
            name: variable,
            var_type,
            default_value,
        },
        pos,
    )
}

fn parse_variable(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<String> {
    debug_assert_eq!(pair.as_rule(), Rule::variable);

    parse_name(&exactly_one(pair.into_inner()), pc)
}

fn parse_default_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Value> {
    debug_assert_eq!(pair.as_rule(), Rule::default_value);

    parse_value(exactly_one(pair.into_inner()), pc)
}

fn parse_type(pair: &Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Type> {
    debug_assert_eq!(pair.as_rule(), Rule::type_);

    Positioned::new(Type::new(pair.as_str()), pc.step(&pair))
}

fn parse_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Value> {
    debug_assert_eq!(pair.as_rule(), Rule::value);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Positioned::new(
        match pair.as_rule() {
            Rule::variable => Value::Variable(parse_variable(pair, pc).node),
            Rule::float | Rule::int => {
                Value::Number(pair.as_str().parse().expect("failed to parse number"))
            }
            Rule::string => Value::String({
                let pair = exactly_one(pair.into_inner());
                match pair.as_rule() {
                    Rule::block_string_content => block_string_value(pair.as_str()),
                    Rule::string_content => string_value(pair.as_str()),
                    _ => unreachable!(),
                }
            }),
            Rule::boolean => Value::Boolean(match pair.as_str() {
                "true" => true,
                "false" => false,
                _ => unreachable!(),
            }),
            Rule::null => Value::Null,
            Rule::enum_ => Value::Enum(parse_name(&exactly_one(pair.into_inner()), pc).node),
            Rule::list => Value::List(
                pair.into_inner()
                    .map(|pair| parse_value(pair, pc).node)
                    .collect(),
            ),
            Rule::object => Value::Object(
                pair.into_inner()
                    .map(|pair| {
                        let (name, value) = parse_name_value(pair, pc);
                        (name.node, value.node)
                    })
                    .collect(),
            ),
            _ => unreachable!(),
        },
        pos,
    )
}

fn parse_name_value(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> (Positioned<String>, Positioned<Value>) {
    debug_assert_eq!(pair.as_rule(), Rule::name_value);

    let mut pairs = pair.into_inner();

    let name = parse_name(&pairs.next().unwrap(), pc);
    let value = parse_value(pairs.next().unwrap(), pc);

    debug_assert_eq!(pairs.next(), None);

    (name, value)
}

fn parse_selection_set(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<SelectionSet> {
    debug_assert_eq!(pair.as_rule(), Rule::selection_set);

    let pos = pc.step(&pair);

    Positioned::new(
        SelectionSet {
            items: pair
                .into_inner()
                .map(|pair| parse_selection(pair, pc))
                .collect(),
        },
        pos,
    )
}

fn parse_selection(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Selection> {
    debug_assert_eq!(pair.as_rule(), Rule::selection);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Positioned::new(
        match pair.as_rule() {
            Rule::field => Selection::Field(parse_field(pair, pc)),
            Rule::fragment_spread => Selection::FragmentSpread(parse_fragment_spread(pair, pc)),
            Rule::inline_fragment => Selection::InlineFragment(parse_inline_fragment(pair, pc)),
            _ => unreachable!(),
        },
        pos,
    )
}

fn parse_field(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Field> {
    debug_assert_eq!(pair.as_rule(), Rule::field);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let alias = next_if_rule(&mut pairs, Rule::alias).map(|pair| parse_alias(pair, pc));
    let name = parse_name(&pairs.next().unwrap(), pc);
    let arguments = next_if_rule(&mut pairs, Rule::arguments).map(|pair| parse_arguments(pair, pc));
    let directives =
        next_if_rule(&mut pairs, Rule::directives).map(|pair| parse_directives(pair, pc));
    let selection_set =
        next_if_rule(&mut pairs, Rule::selection_set).map(|pair| parse_selection_set(pair, pc));

    Positioned::new(
        Field {
            alias,
            name,
            arguments: arguments.unwrap_or_default(),
            directives: directives.unwrap_or_default(),
            selection_set: selection_set.unwrap_or_default(),
        },
        pos,
    )
}

fn parse_alias(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<String> {
    debug_assert_eq!(pair.as_rule(), Rule::alias);
    parse_name(&exactly_one(pair.into_inner()), pc)
}

fn parse_arguments(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Vec<(Positioned<String>, Positioned<Value>)> {
    debug_assert_eq!(pair.as_rule(), Rule::arguments);
    pair.into_inner()
        .map(|pair| parse_name_value(pair, pc))
        .collect()
}

fn parse_fragment_spread(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<FragmentSpread> {
    debug_assert_eq!(pair.as_rule(), Rule::fragment_spread);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let fragment_name = parse_name(&pairs.next().unwrap(), pc);
    let directives = pairs.next().map(|pair| parse_directives(pair, pc));

    debug_assert_eq!(pairs.peek(), None);

    Positioned::new(
        FragmentSpread {
            fragment_name,
            directives: directives.unwrap_or_default(),
        },
        pos,
    )
}

fn parse_inline_fragment(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<InlineFragment> {
    debug_assert_eq!(pair.as_rule(), Rule::inline_fragment);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let type_condition =
        next_if_rule(&mut pairs, Rule::type_condition).map(|pair| parse_type_condition(pair, pc));
    let directives =
        next_if_rule(&mut pairs, Rule::directives).map(|pair| parse_directives(pair, pc));
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc);

    debug_assert_eq!(pairs.next(), None);

    Positioned::new(
        InlineFragment {
            type_condition,
            directives: directives.unwrap_or_default(),
            selection_set,
        },
        pos,
    )
}

fn parse_fragment_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<FragmentDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::fragment_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(&pairs.next().unwrap(), pc);
    let type_condition = parse_type_condition(pairs.next().unwrap(), pc);
    let directives =
        next_if_rule(&mut pairs, Rule::directives).map(|pair| parse_directives(pair, pc));
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc);

    debug_assert_eq!(pairs.next(), None);

    Positioned::new(
        FragmentDefinition {
            name,
            type_condition,
            directives: directives.unwrap_or_default(),
            selection_set,
        },
        pos,
    )
}

fn parse_type_condition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Positioned<TypeCondition> {
    debug_assert_eq!(pair.as_rule(), Rule::type_condition);

    let pos = pc.step(&pair);
    Positioned::new(
        TypeCondition {
            on: parse_name(&exactly_one(pair.into_inner()), pc),
        },
        pos,
    )
}

fn parse_directives(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Vec<Positioned<Directive>> {
    debug_assert_eq!(pair.as_rule(), Rule::directives);

    pair.into_inner()
        .map(|pair| parse_directive(pair, pc))
        .collect()
}

fn parse_directive(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<Directive> {
    debug_assert_eq!(pair.as_rule(), Rule::directive);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(&pairs.next().unwrap(), pc);
    let arguments = pairs.next().map(|pair| parse_arguments(pair, pc));

    debug_assert_eq!(pairs.peek(), None);

    Positioned::new(
        Directive {
            name,
            arguments: arguments.unwrap_or_default(),
        },
        pos,
    )
}

fn parse_name(pair: &Pair<Rule>, pc: &mut PositionCalculator) -> Positioned<String> {
    debug_assert_eq!(pair.as_rule(), Rule::name);
    Positioned::new(pair.as_str().to_owned(), pc.step(&pair))
}

// Parser helper functions

fn next_if_rule<'a>(pairs: &mut Pairs<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    if pairs.peek().map_or(false, |pair| pair.as_rule() == rule) {
        Some(pairs.next().unwrap())
    } else {
        None
    }
}
fn exactly_one<T>(iter: impl IntoIterator<Item = T>) -> T {
    let mut iter = iter.into_iter();
    let res = iter.next().unwrap();
    debug_assert!(matches!(iter.next(), None));
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parser() {
        for entry in fs::read_dir("tests/queries").unwrap() {
            if let Ok(entry) = entry {
                GraphQLParser::parse(Rule::document, &fs::read_to_string(entry.path()).unwrap())
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

    #[test]
    fn test_parse_overflowing_int() {
        let query_ok = format!("mutation {{ add(big: {}) }} ", std::i32::MAX);
        let query_overflow = format!("mutation {{ add(big: {}0000) }} ", std::i32::MAX);
        assert!(parse_query(query_ok).is_ok());
        assert!(parse_query(query_overflow).is_ok());
    }
}
