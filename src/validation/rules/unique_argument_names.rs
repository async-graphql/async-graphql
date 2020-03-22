use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::Field;
use graphql_parser::schema::{Directive, Value};
use graphql_parser::Pos;
use std::collections::HashSet;

#[derive(Default)]
pub struct UniqueArgumentNames<'a> {
    names: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for UniqueArgumentNames<'a> {
    fn enter_directive(&mut self, _ctx: &mut VisitorContext<'a>, _directive: &'a Directive) {
        self.names.clear();
    }

    fn enter_argument(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        pos: Pos,
        name: &'a str,
        _value: &'a Value,
    ) {
        if !self.names.insert(name) {
            ctx.report_error(
                vec![pos],
                format!("There can only be one argument named \"{}\"", name),
            )
        }
    }

    fn enter_field(&mut self, _ctx: &mut VisitorContext<'a>, _field: &'a Field) {
        self.names.clear();
    }
}
