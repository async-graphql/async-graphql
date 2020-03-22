use crate::registry;
use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::Field;

#[derive(Default)]
pub struct FieldsOnCorrectType;

impl<'a> Visitor<'a> for FieldsOnCorrectType {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
        if ctx
            .parent_type()
            .unwrap()
            .field_by_name(&field.name)
            .is_none()
        {
            if let Some(registry::Type::Union { .. }) = ctx.parent_type() {
                if field.name == "__typename" {
                    return;
                }
            }

            ctx.report_error(
                vec![field.position],
                format!(
                    "Unknown field \"{}\" on type \"{}\"",
                    field.name,
                    ctx.parent_type().unwrap().name()
                ),
            );
        }
    }
}
