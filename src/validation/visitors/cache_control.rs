use crate::{
    parser::types::{Field, SelectionSet},
    registry::MetaType,
    validation::visitor::{VisitMode, Visitor, VisitorContext},
    CacheControl, Positioned,
};

pub struct CacheControlCalculate<'a> {
    pub cache_control: &'a mut CacheControl,
}

impl Visitor<'_> for CacheControlCalculate<'_> {
    fn mode(&self) -> VisitMode {
        VisitMode::Inline
    }

    fn enter_selection_set(
        &mut self,
        ctx: &mut VisitorContext<'_>,
        _selection_set: &Positioned<SelectionSet>,
    ) {
        if let Some(MetaType::Object { cache_control, .. }) = ctx.current_type() {
            *self.cache_control = self.cache_control.merge(cache_control);
        }
    }

    fn enter_field(&mut self, ctx: &mut VisitorContext<'_>, field: &Positioned<Field>) {
        if let Some(registry_field) = ctx
            .parent_type()
            .and_then(|parent| parent.field_by_name(&field.node.name.node))
        {
            *self.cache_control = self.cache_control.merge(&registry_field.cache_control);
        }
    }
}
