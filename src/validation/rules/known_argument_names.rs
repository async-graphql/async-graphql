use crate::registry::InputValue;
use crate::validation::suggestion::make_suggestion;
use crate::validation::visitor::{Visitor, VisitorContext};
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
    current_args: Option<(&'a HashMap<&'static str, InputValue>, ArgsType<'a>)>,
}

impl<'a> KnownArgumentNames<'a> {
    fn get_suggestion(&self, name: &str) -> String {
        make_suggestion(
            " Did you mean",
            self.current_args
                .iter()
                .map(|(args, _)| args.iter().map(|arg| *arg.0))
                .flatten(),
            name,
        )
        .unwrap_or_default()
    }
}

impl<'a> Visitor<'a> for KnownArgumentNames<'a> {
    fn enter_directive(&mut self, ctx: &mut VisitorContext<'a>, directive: &'a Directive) {
        self.current_args = ctx
            .registry
            .directives
            .get(&directive.name)
            .map(|d| (&d.args, ArgsType::Directive(&directive.name)));
    }

    fn exit_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Directive) {
        self.current_args = None;
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        pos: Pos,
        name: &str,
        _value: &'a Value,
    ) {
        if let Some((args, arg_type)) = &self.current_args {
            if !args.contains_key(name) {
                match arg_type {
                    ArgsType::Field {
                        field_name,
                        type_name,
                    } => {
                        ctx.report_error(
                            vec![pos],
                            format!(
                                "Unknown argument \"{}\" on field \"{}\" of type \"{}\".{}",
                                name,
                                field_name,
                                type_name,
                                self.get_suggestion(name)
                            ),
                        );
                    }
                    ArgsType::Directive(directive_name) => {
                        ctx.report_error(
                            vec![pos],
                            format!(
                                "Unknown argument \"{}\" on directive \"{}\".{}",
                                name,
                                directive_name,
                                self.get_suggestion(name)
                            ),
                        );
                    }
                }
            }
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
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
                )
            });
    }

    fn exit_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Field) {
        self.current_args = None;
    }
}
