use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::Field;

#[derive(Default)]
pub struct NoComposeLeafs;

impl<'a> Visitor<'a> for NoComposeLeafs {
    fn enter_field(&mut self, ctx: &mut VisitorContext<'a>, field: &'a Field) {
        if let Some(ty) = ctx.parent_type() {
            if let Some(schema_field) = ty.field_by_name(&field.name) {
                if let Some(ty) = ctx.registry.basic_type_by_typename(&schema_field.ty) {
                    if ty.is_composite() && field.selection_set.items.is_empty() {
                        ctx.report_error(
                            vec![field.position],
                            format!(
                                "Field \"{}\" of type \"{}\" must have a selection of subfields",
                                field.name,
                                ty.name()
                            ),
                        )
                    }
                }
            }
        }
    }
}
