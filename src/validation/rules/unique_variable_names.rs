use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{OperationDefinition, VariableDefinition};
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueVariableNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueVariableNames<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.names.clear();
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        if !self.names.insert(variable_definition.name.as_str()) {
            ctx.report_error(
                vec![variable_definition.position],
                format!(
                    "There can only be one variable named \"${}\"",
                    variable_definition.name
                ),
            );
        }
    }
}
