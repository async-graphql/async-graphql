use super::*;

const MAX_RECURSION_DEPTH: usize = 64;

macro_rules! recursion_depth {
    ($remaining_depth:ident) => {{
        if $remaining_depth == 0 {
            return Err(Error::RecursionLimitExceeded);
        }
        $remaining_depth - 1
    }};
}

/// Parse a GraphQL query document.
///
/// # Errors
///
/// Fails if the query is not a valid GraphQL document.
pub fn parse_query<T: AsRef<str>>(input: T) -> Result<ExecutableDocument> {
    let mut pc = PositionCalculator::new(input.as_ref());

    let pairs = GraphQLParser::parse(Rule::executable_document, input.as_ref())?;
    let items = parse_definition_items(exactly_one(pairs), &mut pc)?;

    let mut operations = None;
    let mut fragments: HashMap<_, Positioned<FragmentDefinition>> = HashMap::new();

    for item in items {
        match item {
            DefinitionItem::Operation(item) => {
                if let Some(name) = item.node.name {
                    let operations = operations
                        .get_or_insert_with(|| DocumentOperations::Multiple(HashMap::new()));
                    let operations = match operations {
                        DocumentOperations::Single(anonymous) => {
                            return Err(Error::MultipleOperations {
                                anonymous: anonymous.pos,
                                operation: item.pos,
                            })
                        }
                        DocumentOperations::Multiple(operations) => operations,
                    };

                    match operations.entry(name.node) {
                        hash_map::Entry::Occupied(entry) => {
                            let (name, first) = entry.remove_entry();
                            return Err(Error::OperationDuplicated {
                                operation: name,
                                first: first.pos,
                                second: item.pos,
                            });
                        }
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(Positioned::new(item.node.definition, item.pos));
                        }
                    }
                } else {
                    match operations {
                        Some(operations) => {
                            return Err(Error::MultipleOperations {
                                anonymous: item.pos,
                                operation: match operations {
                                    DocumentOperations::Single(single) => single.pos,
                                    DocumentOperations::Multiple(map) => {
                                        map.values().next().unwrap().pos
                                    }
                                },
                            });
                        }
                        None => {
                            operations = Some(DocumentOperations::Single(Positioned::new(
                                item.node.definition,
                                item.pos,
                            )));
                        }
                    }
                }
            }
            DefinitionItem::Fragment(item) => match fragments.entry(item.node.name.node) {
                hash_map::Entry::Occupied(entry) => {
                    let (name, first) = entry.remove_entry();
                    return Err(Error::FragmentDuplicated {
                        fragment: name,
                        first: first.pos,
                        second: item.pos,
                    });
                }
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(Positioned::new(item.node.definition, item.pos));
                }
            },
        }
    }

    Ok(ExecutableDocument {
        operations: operations.ok_or(Error::MissingOperation)?,
        fragments,
    })
}

fn parse_definition_items(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<DefinitionItem>> {
    debug_assert_eq!(pair.as_rule(), Rule::executable_document);

    Ok(pair
        .into_inner()
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| parse_definition_item(pair, pc))
        .collect::<Result<_>>()?)
}

enum DefinitionItem {
    Operation(Positioned<OperationDefinitionItem>),
    Fragment(Positioned<FragmentDefinitionItem>),
}

fn parse_definition_item(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<DefinitionItem> {
    debug_assert_eq!(pair.as_rule(), Rule::executable_definition);

    let pair = exactly_one(pair.into_inner());
    Ok(match pair.as_rule() {
        Rule::operation_definition => {
            DefinitionItem::Operation(parse_operation_definition_item(pair, pc)?)
        }
        Rule::fragment_definition => {
            DefinitionItem::Fragment(parse_fragment_definition_item(pair, pc)?)
        }
        _ => unreachable!(),
    })
}

struct OperationDefinitionItem {
    name: Option<Positioned<Name>>,
    definition: OperationDefinition,
}

fn parse_operation_definition_item(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<OperationDefinitionItem>> {
    debug_assert_eq!(pair.as_rule(), Rule::operation_definition);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());
    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::named_operation_definition => parse_named_operation_definition(pair, pc)?,
            Rule::selection_set => OperationDefinitionItem {
                name: None,
                definition: OperationDefinition {
                    ty: OperationType::Query,
                    variable_definitions: Vec::new(),
                    directives: Vec::new(),
                    selection_set: parse_selection_set(pair, pc, MAX_RECURSION_DEPTH)?,
                },
            },
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_named_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<OperationDefinitionItem> {
    debug_assert_eq!(pair.as_rule(), Rule::named_operation_definition);

    let mut pairs = pair.into_inner();

    let ty = parse_operation_type(pairs.next().unwrap(), pc)?;
    let name = parse_if_rule(&mut pairs, Rule::name, |pair| parse_name(pair, pc))?;
    let variable_definitions = parse_if_rule(&mut pairs, Rule::variable_definitions, |pair| {
        parse_variable_definitions(pair, pc)
    })?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc, MAX_RECURSION_DEPTH)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(OperationDefinitionItem {
        name,
        definition: OperationDefinition {
            ty: ty.node,
            variable_definitions: variable_definitions.unwrap_or_default(),
            directives,
            selection_set,
        },
    })
}

fn parse_variable_definitions(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<VariableDefinition>>> {
    debug_assert_eq!(pair.as_rule(), Rule::variable_definitions);

    pair.into_inner()
        .map(|pair| parse_variable_definition(pair, pc))
        .collect()
}

fn parse_variable_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<VariableDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::variable_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let variable = parse_variable(pairs.next().unwrap(), pc)?;
    let var_type = parse_type(pairs.next().unwrap(), pc)?;

    let directives = parse_opt_directives(&mut pairs, pc)?;
    let default_value = parse_if_rule(&mut pairs, Rule::default_value, |pair| {
        parse_default_value(pair, pc)
    })?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        VariableDefinition {
            name: variable,
            var_type,
            directives,
            default_value,
        },
        pos,
    ))
}

fn parse_selection_set(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
    remaining_depth: usize,
) -> Result<Positioned<SelectionSet>> {
    debug_assert_eq!(pair.as_rule(), Rule::selection_set);

    let pos = pc.step(&pair);

    Ok(Positioned::new(
        SelectionSet {
            items: pair
                .into_inner()
                .map(|pair| parse_selection(pair, pc, remaining_depth))
                .collect::<Result<_>>()?,
        },
        pos,
    ))
}

fn parse_selection(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
    remaining_depth: usize,
) -> Result<Positioned<Selection>> {
    debug_assert_eq!(pair.as_rule(), Rule::selection);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::field => Selection::Field(parse_field(pair, pc, remaining_depth)?),
            Rule::fragment_spread => Selection::FragmentSpread(parse_fragment_spread(pair, pc)?),
            Rule::inline_fragment => {
                Selection::InlineFragment(parse_inline_fragment(pair, pc, remaining_depth)?)
            }
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_field(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
    remaining_depth: usize,
) -> Result<Positioned<Field>> {
    debug_assert_eq!(pair.as_rule(), Rule::field);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let alias = parse_if_rule(&mut pairs, Rule::alias, |pair| parse_alias(pair, pc))?;
    let name = parse_name(pairs.next().unwrap(), pc)?;
    let arguments = parse_if_rule(&mut pairs, Rule::arguments, |pair| {
        parse_arguments(pair, pc)
    })?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_if_rule(&mut pairs, Rule::selection_set, |pair| {
        parse_selection_set(pair, pc, recursion_depth!(remaining_depth))
    })?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        Field {
            alias,
            name,
            arguments: arguments.unwrap_or_default(),
            directives,
            selection_set: selection_set.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_alias(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Name>> {
    debug_assert_eq!(pair.as_rule(), Rule::alias);
    parse_name(exactly_one(pair.into_inner()), pc)
}

fn parse_fragment_spread(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FragmentSpread>> {
    debug_assert_eq!(pair.as_rule(), Rule::fragment_spread);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let fragment_name = parse_name(pairs.next().unwrap(), pc)?;
    let directives = parse_opt_directives(&mut pairs, pc)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        FragmentSpread {
            fragment_name,
            directives,
        },
        pos,
    ))
}

fn parse_inline_fragment(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
    remaining_depth: usize,
) -> Result<Positioned<InlineFragment>> {
    debug_assert_eq!(pair.as_rule(), Rule::inline_fragment);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let type_condition = parse_if_rule(&mut pairs, Rule::type_condition, |pair| {
        parse_type_condition(pair, pc)
    })?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set =
        parse_selection_set(pairs.next().unwrap(), pc, recursion_depth!(remaining_depth))?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        InlineFragment {
            type_condition,
            directives,
            selection_set,
        },
        pos,
    ))
}

struct FragmentDefinitionItem {
    name: Positioned<Name>,
    definition: FragmentDefinition,
}

fn parse_fragment_definition_item(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FragmentDefinitionItem>> {
    debug_assert_eq!(pair.as_rule(), Rule::fragment_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(pairs.next().unwrap(), pc)?;
    let type_condition = parse_type_condition(pairs.next().unwrap(), pc)?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc, MAX_RECURSION_DEPTH)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        FragmentDefinitionItem {
            name,
            definition: FragmentDefinition {
                type_condition,
                directives,
                selection_set,
            },
        },
        pos,
    ))
}

fn parse_type_condition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<TypeCondition>> {
    debug_assert_eq!(pair.as_rule(), Rule::type_condition);

    let pos = pc.step(&pair);
    Ok(Positioned::new(
        TypeCondition {
            on: parse_name(exactly_one(pair.into_inner()), pc)?,
        },
        pos,
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_parser() {
        for entry in fs::read_dir("tests/executables").unwrap() {
            let entry = entry.unwrap();
            eprintln!("Parsing file {}", entry.path().display());

            GraphQLParser::parse(
                Rule::executable_document,
                &fs::read_to_string(entry.path()).unwrap(),
            )
            .unwrap();
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/executables").unwrap() {
            let entry = entry.unwrap();
            eprintln!("Parsing and transforming file {}", entry.path().display());
            parse_query(fs::read_to_string(entry.path()).unwrap()).unwrap();
        }
    }

    #[test]
    fn test_parse_overflowing_int() {
        let query_ok = format!("mutation {{ add(big: {}) }} ", i32::MAX);
        let query_overflow = format!("mutation {{ add(big: {}0000) }} ", i32::MAX);
        assert!(parse_query(query_ok).is_ok());
        assert!(parse_query(query_overflow).is_ok());
    }
}
