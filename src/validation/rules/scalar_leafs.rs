use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::Field;

#[derive(Default)]
pub struct ScalarLeafs;

impl<'a> Visitor<'a> for ScalarLeafs {
    fn enter_field(&mut self, ctx: &mut ValidatorContext<'a>, field: &'a Field) {
        if let Some(ty) = ctx.parent_type() {
            if let Some(schema_field) = ty.field_by_name(&field.name) {
                if let Some(ty) = ctx.registry.get_basic_type(&schema_field.ty) {
                    if ty.is_leaf() && !field.selection_set.items.is_empty() {
                        ctx.report_error(vec![field.position], format!(
                            "Field \"{}\" must not have a selection since type \"{}\" has no subfields",
                            field.name, ty.name()
                        ))
                    }
                }
            }
        }
    }
}
