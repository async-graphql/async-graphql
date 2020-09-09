use super::*;

/// Parse a GraphQL query document.
///
/// # Errors
///
/// Fails if the query is not a valid GraphQL document.
pub fn parse_query<T: AsRef<str>>(input: T) -> Result<ExecutableDocument> {
    let mut pc = PositionCalculator::new(input.as_ref());
    Ok(parse_executable_document(
        exactly_one(GraphQLParser::parse(
            Rule::executable_document,
            input.as_ref(),
        )?),
        &mut pc,
    )?)
}

fn parse_executable_document(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<ExecutableDocument> {
    debug_assert_eq!(pair.as_rule(), Rule::executable_document);

    Ok(ExecutableDocument {
        definitions: pair
            .into_inner()
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| parse_executable_definition(pair, pc))
            .collect::<Result<_>>()?,
    })
}

fn parse_executable_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<ExecutableDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::executable_definition);

    let pair = exactly_one(pair.into_inner());
    Ok(match pair.as_rule() {
        Rule::operation_definition => {
            ExecutableDefinition::Operation(parse_operation_definition(pair, pc)?)
        }
        Rule::fragment_definition => {
            ExecutableDefinition::Fragment(parse_fragment_definition(pair, pc)?)
        }
        _ => unreachable!(),
    })
}

fn parse_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<OperationDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::operation_definition);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());
    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::named_operation_definition => parse_named_operation_definition(pair, pc)?,
            Rule::selection_set => OperationDefinition {
                ty: OperationType::Query,
                name: None,
                variable_definitions: Vec::new(),
                directives: Vec::new(),
                selection_set: parse_selection_set(pair, pc)?,
            },
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_named_operation_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<OperationDefinition> {
    debug_assert_eq!(pair.as_rule(), Rule::named_operation_definition);

    let mut pairs = pair.into_inner();

    let ty = parse_operation_type(pairs.next().unwrap(), pc)?;
    let name = parse_if_rule(&mut pairs, Rule::name, |pair| parse_name(pair, pc))?;
    let variable_definitions = parse_if_rule(&mut pairs, Rule::variable_definitions, |pair| {
        parse_variable_definitions(pair, pc)
    })?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(OperationDefinition {
        ty: ty.node,
        name,
        variable_definitions: variable_definitions.unwrap_or_default(),
        directives,
        selection_set,
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
    let default_value = parse_if_rule(&mut pairs, Rule::default_value, |pair| {
        parse_default_value(pair, pc)
    })?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        VariableDefinition {
            name: variable,
            var_type,
            default_value,
        },
        pos,
    ))
}

fn parse_selection_set(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<SelectionSet>> {
    debug_assert_eq!(pair.as_rule(), Rule::selection_set);

    let pos = pc.step(&pair);

    Ok(Positioned::new(
        SelectionSet {
            items: pair
                .into_inner()
                .map(|pair| parse_selection(pair, pc))
                .collect::<Result<_>>()?,
        },
        pos,
    ))
}

fn parse_selection(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Selection>> {
    debug_assert_eq!(pair.as_rule(), Rule::selection);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::field => Selection::Field(parse_field(pair, pc)?),
            Rule::fragment_spread => Selection::FragmentSpread(parse_fragment_spread(pair, pc)?),
            Rule::inline_fragment => Selection::InlineFragment(parse_inline_fragment(pair, pc)?),
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_field(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Field>> {
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
        parse_selection_set(pair, pc)
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
) -> Result<Positioned<InlineFragment>> {
    debug_assert_eq!(pair.as_rule(), Rule::inline_fragment);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let type_condition = parse_if_rule(&mut pairs, Rule::type_condition, |pair| {
        parse_type_condition(pair, pc)
    })?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc)?;

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

fn parse_fragment_definition(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<FragmentDefinition>> {
    debug_assert_eq!(pair.as_rule(), Rule::fragment_definition);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(pairs.next().unwrap(), pc)?;
    let type_condition = parse_type_condition(pairs.next().unwrap(), pc)?;
    let directives = parse_opt_directives(&mut pairs, pc)?;
    let selection_set = parse_selection_set(pairs.next().unwrap(), pc)?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        FragmentDefinition {
            name,
            type_condition,
            directives,
            selection_set,
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
    use super::*;
    use std::fs;

    #[test]
    fn test_parser() {
        for entry in fs::read_dir("tests/executables").unwrap() {
            if let Ok(entry) = entry {
                GraphQLParser::parse(
                    Rule::executable_document,
                    &fs::read_to_string(entry.path()).unwrap(),
                )
                .unwrap();
            }
        }
    }

    #[test]
    fn test_parser_ast() {
        for entry in fs::read_dir("tests/executables").unwrap() {
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
