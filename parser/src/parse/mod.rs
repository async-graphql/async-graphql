//! Parsing module.
//!
//! This module's structure mirrors `types`.

use crate::pos::{PositionCalculator, Positioned};
use crate::types::*;
use crate::{Error, Result};
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use utils::*;

mod executable;
mod service;
mod utils;

pub use executable::parse_query;
pub use service::parse_schema;

#[derive(Parser)]
#[grammar = "graphql.pest"]
struct GraphQLParser;

fn parse_operation_type(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<OperationType>> {
    debug_assert_eq!(pair.as_rule(), Rule::operation_type);

    let pos = pc.step(&pair);

    Ok(Positioned::new(
        match pair.as_str() {
            "query" => OperationType::Query,
            "mutation" => OperationType::Mutation,
            "subscription" => OperationType::Subscription,
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_default_value(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<ConstValue>> {
    debug_assert_eq!(pair.as_rule(), Rule::default_value);

    parse_const_value(exactly_one(pair.into_inner()), pc)
}

fn parse_type(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Type>> {
    debug_assert_eq!(pair.as_rule(), Rule::type_);

    Ok(Positioned::new(
        Type::new(pair.as_str()).unwrap(),
        pc.step(&pair),
    ))
}

fn parse_const_value(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<ConstValue>> {
    debug_assert_eq!(pair.as_rule(), Rule::const_value);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::number => ConstValue::Number(parse_number(pair, pc)?.node),
            Rule::string => ConstValue::String(parse_string(pair, pc)?.node),
            Rule::boolean => ConstValue::Boolean(parse_boolean(pair, pc)?.node),
            Rule::null => ConstValue::Null,
            Rule::enum_value => ConstValue::Enum(parse_enum_value(pair, pc)?.node),
            Rule::const_list => ConstValue::List(
                pair.into_inner()
                    .map(|pair| Ok(parse_const_value(pair, pc)?.node))
                    .collect::<Result<_>>()?,
            ),
            Rule::const_object => ConstValue::Object(
                pair.into_inner()
                    .map(|pair| {
                        debug_assert_eq!(pair.as_rule(), Rule::const_object_field);

                        let mut pairs = pair.into_inner();

                        let name = parse_name(pairs.next().unwrap(), pc)?;
                        let value = parse_const_value(pairs.next().unwrap(), pc)?;

                        debug_assert_eq!(pairs.next(), None);

                        Ok((name.node, value.node))
                    })
                    .collect::<Result<_>>()?,
            ),
            _ => unreachable!(),
        },
        pos,
    ))
}
fn parse_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Value>> {
    debug_assert_eq!(pair.as_rule(), Rule::value);

    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());

    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::variable => Value::Variable(parse_variable(pair, pc)?.node),
            Rule::number => Value::Number(parse_number(pair, pc)?.node),
            Rule::string => Value::String(parse_string(pair, pc)?.node),
            Rule::boolean => Value::Boolean(parse_boolean(pair, pc)?.node),
            Rule::null => Value::Null,
            Rule::enum_value => Value::Enum(parse_enum_value(pair, pc)?.node),
            Rule::list => Value::List(
                pair.into_inner()
                    .map(|pair| Ok(parse_value(pair, pc)?.node))
                    .collect::<Result<_>>()?,
            ),
            Rule::object => Value::Object(
                pair.into_inner()
                    .map(|pair| {
                        debug_assert_eq!(pair.as_rule(), Rule::object_field);
                        let mut pairs = pair.into_inner();

                        let name = parse_name(pairs.next().unwrap(), pc)?;
                        let value = parse_value(pairs.next().unwrap(), pc)?;

                        debug_assert_eq!(pairs.next(), None);

                        Ok((name.node, value.node))
                    })
                    .collect::<Result<_>>()?,
            ),
            _ => unreachable!(),
        },
        pos,
    ))
}

fn parse_variable(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Name>> {
    debug_assert_eq!(pair.as_rule(), Rule::variable);
    parse_name(exactly_one(pair.into_inner()), pc)
}
fn parse_number(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Number>> {
    debug_assert_eq!(pair.as_rule(), Rule::number);
    let pos = pc.step(&pair);
    Ok(Positioned::new(
        pair.as_str().parse().expect("failed to parse number"),
        pos,
    ))
}
fn parse_string(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<String>> {
    debug_assert_eq!(pair.as_rule(), Rule::string);
    let pos = pc.step(&pair);
    let pair = exactly_one(pair.into_inner());
    Ok(Positioned::new(
        match pair.as_rule() {
            Rule::block_string_content => block_string_value(pair.as_str()),
            Rule::string_content => string_value(pair.as_str()),
            _ => unreachable!(),
        },
        pos,
    ))
}
fn parse_boolean(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<bool>> {
    debug_assert_eq!(pair.as_rule(), Rule::boolean);
    let pos = pc.step(&pair);
    Ok(Positioned::new(
        match pair.as_str() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        },
        pos,
    ))
}
fn parse_enum_value(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Name>> {
    debug_assert_eq!(pair.as_rule(), Rule::enum_value);
    parse_name(exactly_one(pair.into_inner()), pc)
}

fn parse_opt_const_directives<'a>(
    pairs: &mut Pairs<'a, Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<ConstDirective>>> {
    Ok(parse_if_rule(pairs, Rule::const_directives, |pair| {
        parse_const_directives(pair, pc)
    })?
    .unwrap_or_default())
}
fn parse_opt_directives<'a>(
    pairs: &mut Pairs<'a, Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<Directive>>> {
    Ok(
        parse_if_rule(pairs, Rule::directives, |pair| parse_directives(pair, pc))?
            .unwrap_or_default(),
    )
}
fn parse_const_directives(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<ConstDirective>>> {
    debug_assert_eq!(pair.as_rule(), Rule::const_directives);

    pair.into_inner()
        .map(|pair| parse_const_directive(pair, pc))
        .collect()
}
fn parse_directives(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<Positioned<Directive>>> {
    debug_assert_eq!(pair.as_rule(), Rule::directives);

    pair.into_inner()
        .map(|pair| parse_directive(pair, pc))
        .collect()
}

fn parse_const_directive(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Positioned<ConstDirective>> {
    debug_assert_eq!(pair.as_rule(), Rule::const_directive);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(pairs.next().unwrap(), pc)?;
    let arguments = parse_if_rule(&mut pairs, Rule::const_arguments, |pair| {
        parse_const_arguments(pair, pc)
    })?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        ConstDirective {
            name,
            arguments: arguments.unwrap_or_default(),
        },
        pos,
    ))
}
fn parse_directive(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Directive>> {
    debug_assert_eq!(pair.as_rule(), Rule::directive);

    let pos = pc.step(&pair);
    let mut pairs = pair.into_inner();

    let name = parse_name(pairs.next().unwrap(), pc)?;
    let arguments = parse_if_rule(&mut pairs, Rule::arguments, |pair| {
        parse_arguments(pair, pc)
    })?;

    debug_assert_eq!(pairs.next(), None);

    Ok(Positioned::new(
        Directive {
            name,
            arguments: arguments.unwrap_or_default(),
        },
        pos,
    ))
}

fn parse_const_arguments(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<(Positioned<Name>, Positioned<ConstValue>)>> {
    debug_assert_eq!(pair.as_rule(), Rule::const_arguments);
    pair.into_inner()
        .map(|pair| {
            debug_assert_eq!(pair.as_rule(), Rule::const_argument);
            let mut pairs = pair.into_inner();

            let name = parse_name(pairs.next().unwrap(), pc)?;
            let value = parse_const_value(pairs.next().unwrap(), pc)?;

            debug_assert_eq!(pairs.next(), None);

            Ok((name, value))
        })
        .collect()
}
fn parse_arguments(
    pair: Pair<Rule>,
    pc: &mut PositionCalculator,
) -> Result<Vec<(Positioned<Name>, Positioned<Value>)>> {
    debug_assert_eq!(pair.as_rule(), Rule::arguments);
    pair.into_inner()
        .map(|pair| {
            debug_assert_eq!(pair.as_rule(), Rule::argument);
            let mut pairs = pair.into_inner();

            let name = parse_name(pairs.next().unwrap(), pc)?;
            let value = parse_value(pairs.next().unwrap(), pc)?;

            debug_assert_eq!(pairs.next(), None);

            Ok((name, value))
        })
        .collect()
}

fn parse_name(pair: Pair<Rule>, pc: &mut PositionCalculator) -> Result<Positioned<Name>> {
    debug_assert_eq!(pair.as_rule(), Rule::name);
    Ok(Positioned::new(
        Name::new_unchecked(pair.as_str().to_owned()),
        pc.step(&pair),
    ))
}
