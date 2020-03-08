use crate::registry::InputValue;
use crate::validation::context::ValidatorContext;
use crate::validation::utils::is_valid_input_value;
use crate::validation::visitor::Visitor;
use graphql_parser::query::Field;
use graphql_parser::schema::{Directive, Value};
use graphql_parser::Pos;
use std::collections::HashMap;

#[derive(Default)]
pub struct ArgumentsOfCorrectType<'a> {
    current_args: Option<(&'a HashMap<&'static str, InputValue>, Pos)>,
}

impl<'a> Visitor<'a> for ArgumentsOfCorrectType<'a> {
    fn enter_directive(&mut self, ctx: &mut ValidatorContext<'a>, directive: &'a Directive) {
        self.current_args = ctx
            .registry
            .directives
            .get(&directive.name)
            .map(|d| (&d.args, directive.position));
    }

    fn exit_directive(&mut self, _ctx: &mut ValidatorContext<'a>, _directive: &'a Directive) {
        self.current_args = None;
    }

    fn enter_argument(&mut self, ctx: &mut ValidatorContext<'a>, name: &str, value: &'a Value) {
        if let Some((arg, pos)) = self
            .current_args
            .and_then(|(args, pos)| args.get(name).map(|input| (input, pos)))
        {
            if !is_valid_input_value(ctx.registry, &arg.ty, value) {
                ctx.report_error(
                    vec![pos],
                    format!(
                        "Invalid value for argument \"{}\", expected type \"{}\"",
                        arg.name, arg.ty,
                    ),
                );
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut ValidatorContext<'a>, field: &'a Field) {
        self.current_args = ctx
            .parent_type()
            .field_by_name(&field.name)
            .map(|f| (&f.args, field.position));
    }

    fn exit_field(&mut self, _ctx: &mut ValidatorContext<'a>, _field: &'a Field) {
        self.current_args = None;
    }
}
