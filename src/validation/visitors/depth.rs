use crate::parser::types::{FragmentSpread, InlineFragment, SelectionSet};
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

pub struct DepthCalculate<'a> {
    max_depth: &'a mut i32,
    current_depth: i32,
}

impl<'a> DepthCalculate<'a> {
    pub fn new(max_depth: &'a mut i32) -> Self {
        *max_depth = -1;
        Self {
            max_depth,
            current_depth: -1,
        }
    }
}

impl<'ctx, 'a> Visitor<'ctx> for DepthCalculate<'a> {
    fn enter_selection_set(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _selection_set: &'ctx Positioned<SelectionSet>,
    ) {
        self.current_depth += 1;
        *self.max_depth = (*self.max_depth).max(self.current_depth);
    }

    fn exit_selection_set(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _selection_set: &'ctx Positioned<SelectionSet>,
    ) {
        self.current_depth -= 1;
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _fragment_spread: &'ctx Positioned<FragmentSpread>,
    ) {
        self.current_depth -= 1;
    }

    fn exit_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _fragment_spread: &'ctx Positioned<FragmentSpread>,
    ) {
        self.current_depth += 1;
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _inline_fragment: &'ctx Positioned<InlineFragment>,
    ) {
        self.current_depth -= 1;
    }

    fn exit_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _inline_fragment: &'ctx Positioned<InlineFragment>,
    ) {
        self.current_depth += 1;
    }
}
