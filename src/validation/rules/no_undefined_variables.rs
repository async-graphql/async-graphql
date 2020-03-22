use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{OperationDefinition, VariableDefinition};
use graphql_parser::schema::Value;
use graphql_parser::Pos;
use std::collections::HashSet;

#[derive(Default)]
pub struct NoUndefinedVariables<'a> {
    vars: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for NoUndefinedVariables<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.vars.clear();
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut VisitorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        self.vars.insert(&variable_definition.name);
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        pos: Pos,
        _name: &str,
        value: &'a Value,
    ) {
        if let Value::Variable(var_name) = value {
            if !self.vars.contains(var_name.as_str()) {
                ctx.report_error(
                    vec![pos],
                    format!("Variable \"${}\" is not defined", var_name),
                );
            }
        }
    }
}
