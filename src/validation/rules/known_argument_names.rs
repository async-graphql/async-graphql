use crate::registry::InputValue;
use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use crate::Value;
use graphql_parser::query::{Directive, Field};
use graphql_parser::Pos;
use std::collections::HashMap;

enum ArgsType<'a> {
    Directive(&'a str),
    Field {
        field_name: &'a str,
        type_name: &'a str,
    },
}

#[derive(Default)]
pub struct KnownArgumentNames<'a> {
    current_args: Option<(&'a HashMap<&'static str, InputValue>, ArgsType<'a>, Pos)>,
}

impl<'a> Visitor<'a> for KnownArgumentNames<'a> {
    fn enter_directive(&mut self, ctx: &mut ValidatorContext<'a>, directive: &'a Directive) {
        self.current_args = ctx.registry.directives.get(&directive.name).map(|d| {
            (
                &d.args,
                ArgsType::Directive(&directive.name),
                directive.position,
            )
        });
    }

    fn exit_directive(&mut self, _ctx: &mut ValidatorContext<'a>, _directive: &'a Directive) {
        self.current_args = None;
    }

    fn enter_argument(&mut self, ctx: &mut ValidatorContext<'a>, name: &str, _value: &'a Value) {
        if let Some((args, arg_type, pos)) = &self.current_args {
            if !args.contains_key(name) {
                match arg_type {
                    ArgsType::Field {
                        field_name,
                        type_name,
                    } => {
                        ctx.report_error(
                            vec![*pos],
                            format!(
                                "Unknown argument \"{}\" on field \"{}\" of type \"{}\"",
                                name, field_name, type_name,
                            ),
                        );
                    }
                    ArgsType::Directive(directive_name) => {
                        ctx.report_error(
                            vec![*pos],
                            format!(
                                "Unknown argument \"{}\" on directive \"{}\"",
                                name, directive_name
                            ),
                        );
                    }
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut ValidatorContext<'a>, field: &'a Field) {
        self.current_args = ctx
            .parent_type()
            .and_then(|p| p.field_by_name(&field.name))
            .map(|f| {
                (
                    &f.args,
                    ArgsType::Field {
                        field_name: &field.name,
                        type_name: ctx.parent_type().unwrap().name(),
                    },
                    field.position,
                )
            });
    }

    fn exit_field(&mut self, _ctx: &mut ValidatorContext<'a>, _field: &'a Field) {
        self.current_args = None;
    }
}
