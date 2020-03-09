use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{Definition, Document, FragmentSpread};
use std::collections::HashSet;

#[derive(Default)]
pub struct NoUnusedFragments<'a> {
    spreads: HashSet<&'a str>,
}

impl<'a> Visitor<'a> for NoUnusedFragments<'a> {
    fn exit_document(&mut self, ctx: &mut ValidatorContext<'a>, doc: &'a Document) {
        for d in &doc.definitions {
            if let Definition::Fragment(fragment) = d {
                if !self.spreads.contains(fragment.name.as_str()) {
                    ctx.report_error(
                        vec![fragment.position],
                        format!(r#"Fragment "{}" is never used"#, fragment.name),
                    );
                }
            }
        }
    }

    fn enter_fragment_spread(
        &mut self,
        _ctx: &mut ValidatorContext<'a>,
        fragment_spread: &'a FragmentSpread,
    ) {
        self.spreads.insert(&fragment_spread.fragment_name);
    }
}
