use crate::visitor::{Visitor, VisitorContext};
use graphql_parser::query::{Definition, Document, FragmentSpread, InlineFragment, TypeCondition};
use std::collections::HashMap;

#[derive(Default)]
pub struct PossibleFragmentSpreads<'a> {
    fragment_types: HashMap<&'a str, &'a str>,
}

impl<'a> Visitor<'a> for PossibleFragmentSpreads<'a> {
    fn enter_document(&mut self, _ctx: &mut VisitorContext<'a>, doc: &'a Document) {
        for d in &doc.definitions {
            if let Definition::Fragment(fragment) = d {
                let TypeCondition::On(type_name) = &fragment.type_condition;
                self.fragment_types
                    .insert(fragment.name.as_str(), type_name);
            }
        }
    }

    fn enter_fragment_spread(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        fragment_spread: &'a FragmentSpread,
    ) {
        if let Some(fragment_type) = self
            .fragment_types
            .get(fragment_spread.fragment_name.as_str())
        {
            if ctx.current_type().name() != *fragment_type {
                ctx.report_error(
                        vec![fragment_spread.position],
                        format!(
                            "Fragment \"{}\" cannot be spread here as objects of type \"{}\" can never be of type \"{}\"",
                            &fragment_spread.fragment_name,  ctx.current_type().name(), fragment_type
                        ),
                    )
            }
        }
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        inline_fragment: &'a InlineFragment,
    ) {
        if let Some(parent_type) = ctx.parent_type() {
            if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
                if !parent_type.is_possible_type(&name) {
                    ctx.report_error(
                        vec![inline_fragment.position],
                        format!(
                            "Fragment cannot be spread here as objects of type \"{}\" \
             can never be of type \"{}\"",
                            parent_type.name(),
                            name
                        ),
                    )
                }
            }
        }
    }
}
