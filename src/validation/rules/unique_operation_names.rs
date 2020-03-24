use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{Mutation, OperationDefinition, Query, Subscription};
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueOperationNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueOperationNames<'a> {
    fn enter_operation_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        operation_definition: &'a OperationDefinition,
    ) {
        let name = match operation_definition {
            OperationDefinition::Query(Query { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::Mutation(Mutation { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::Subscription(Subscription { name, position, .. }) => {
                name.as_ref().map(|name| (name, position))
            }
            OperationDefinition::SelectionSet(_) => None,
        };

        if let Some((name, pos)) = name {
            if !self.names.insert(name.as_str()) {
                ctx.report_error(
                    vec![*pos],
                    format!("There can only be one operation named \"{}\"", name),
                )
            }
        }
    }
}
