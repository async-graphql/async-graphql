use crate::validation::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{FragmentSpread, InlineFragment, SelectionSet};

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
        _selection_set: &'ctx SelectionSet,
    ) {
        self.current_depth += 1;
        *self.max_depth = (*self.max_depth).max(self.current_depth);
    }

    fn exit_selection_set(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _selection_set: &'ctx SelectionSet,
    ) {
        self.current_depth -= 1;
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _fragment_spread: &'ctx FragmentSpread,
    ) {
        self.current_depth -= 1;
    }

    fn exit_fragment_spread(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _fragment_spread: &'ctx FragmentSpread,
    ) {
        self.current_depth += 1;
    }

    fn enter_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _inline_fragment: &'ctx InlineFragment,
    ) {
        self.current_depth -= 1;
    }

    fn exit_inline_fragment(
        &mut self,
        _ctx: &mut VisitorContext<'ctx>,
        _inline_fragment: &'ctx InlineFragment,
    ) {
        self.current_depth += 1;
    }
}
