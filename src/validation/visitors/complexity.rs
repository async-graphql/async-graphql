use crate::parser::types::Field;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

pub struct ComplexityCalculate<'a> {
    pub complexity: &'a mut usize,
}

impl<'ctx, 'a> Visitor<'ctx> for ComplexityCalculate<'a> {
    fn enter_field(&mut self, _ctx: &mut VisitorContext<'_>, _field: &Positioned<Field>) {
        *self.complexity += 1;
    }
}
