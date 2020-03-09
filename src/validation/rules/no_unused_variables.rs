use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{OperationDefinition, VariableDefinition};
use graphql_parser::schema::Value;
use graphql_parser::Pos;
use std::collections::HashSet;

#[derive(Default)]
pub struct NoUnusedVariables<'a> {
    vars: HashSet<(&'a str, Pos)>,
    used_vars: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for NoUnusedVariables<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.used_vars.clear();
        self.vars.clear();
    }

    fn exit_operation_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        for (name, pos) in &self.vars {
            if !self.used_vars.contains(name) {
                ctx.report_error(vec![*pos], format!("Variable \"${}\" is not used", name));
            }
        }
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        self.vars
            .insert((&variable_definition.name, variable_definition.position));
    }

    fn enter_argument(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        _pos: Pos,
        _name: &'a str,
        value: &'a Value,
    ) {
        if let Value::Variable(var) = value {
            self.used_vars.insert(var);
        }
    }
}
