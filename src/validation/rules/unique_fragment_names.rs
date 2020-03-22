use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::FragmentDefinition;
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueFragmentNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueFragmentNames<'a> {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_definition: &'a FragmentDefinition,
    ) {
        if !self.names.insert(&fragment_definition.name) {
            ctx.report_error(
                vec![fragment_definition.position],
                format!(
                    "There can only be one fragment named \"{}\"",
                    fragment_definition.name
                ),
            )
        }
    }
}
