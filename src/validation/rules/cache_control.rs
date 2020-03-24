use crate::registry::Type;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::CacheControl;
use graphql_parser::query::{Field, SelectionSet};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut CacheControl,
}

impl<'ctx, 'a> Visitor<'ctx> for CacheControlCalculate<'a> {
    fn enter_selection_set(&mut self, ctx: &mut VisitorContext<'_>, _selection_set: &SelectionSet) {
        if let Type::Object { cache_control, .. } = ctx.current_type() {
            self.cache_control.merge(cache_control);
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_>, field: &Field) {
        if let Some(registry_field) = ctx.parent_type().unwrap().field_by_name(&field.name) {
            self.cache_control.merge(&registry_field.cache_control);
        }
    }
}
