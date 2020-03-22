use crate::registry::InputValue;
use crate::validation::utils::is_valid_input_value;
use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::Field;
use graphql_parser::schema::{Directive, Value};
use graphql_parser::Pos;
use std::collections::HashMap;

#[derive(Default)]
pub struct ArgumentsOfCorrectType<'a> {
    current_args: Option<&'a HashMap<&'static str, InputValue>>,
}

impl<'a> Visitor<'a> for ArgumentsOfCorrectType<'a> {
    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Directive) {
        self.current_args = ctx
            .registry
            .directives
            .get(&directive.name)
            .map(|d| &d.args);
    }

    fn exit_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Directive) {
        self.current_args = None;
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        pos: Pos,
        name: &str,
        value: &'a Value,
    ) {
        if let Some(arg) = self
            .current_args
            .and_then(|args| args.get(name).map(|input| input))
        {
            if let Some(validator) = &arg.validator {
                if let Some(reason) = validator.is_valid(value) {
                    ctx.report_error(
                        vec![pos],
                        format!("Invalid value for argument \"{}\", {}", arg.name, reason,),
                    );
                    return;
                }
            }

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

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
        self.current_args = ctx
            .parent_type()
            .and_then(|p| p.field_by_name(&field.name))
            .map(|f| &f.args);
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Field) {
        self.current_args = None;
    }
}
