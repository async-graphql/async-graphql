use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{Field, OperationDefinition, VariableDefinition};
use graphql_parser::schema::{Directive, Value};
use graphql_parser::Pos;
use std::collections::HashSet;

#[derive(Default)]
pub struct NoUndefinedVariables<'a> {
    vars: HashSet<&'a str>,
    pos_stack: Vec<Pos>,
}

impl<'a> Visitor<'a> for NoUndefinedVariables<'a> {
    fn enter_operation_definition(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        _operation_definition: &'a OperationDefinition,
    ) {
        self.vars.clear();
    }

    fn enter_variable_definition(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        self.vars.insert(&variable_definition.name);
    }

    fn enter_directive(&mut self, _ctx: &mut ValidatorContext<'a>, directive: &'a Directive) {
        self.pos_stack.push(directive.position);
    }

    fn exit_directive(&mut self, _ctx: &mut ValidatorContext<'a>, _directive: &'a Directive) {
        self.pos_stack.pop();
    }

    fn enter_argument(&mut self, ctx: &mut ValidatorContext<'a>, _name: &str, value: &'a Value) {
        if let Value::Variable(var_name) = value {
            if !self.vars.contains(var_name.as_str()) {
                ctx.report_error(
                    vec![self.pos_stack.last().cloned().unwrap()],
                    format!("Variable \"${}\" is not defined", var_name),
                );
            }
        }
    }

    fn enter_field(&mut self, _ctx: &mut ValidatorContext<'a>, field: &'a Field) {
        self.pos_stack.push(field.position);
    }

    fn exit_field(&mut self, _ctx: &mut ValidatorContext<'a>, _field: &'a Field) {
        self.pos_stack.pop();
    }
}
