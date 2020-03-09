use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{FragmentDefinition, InlineFragment, TypeCondition};

#[derive(Default)]
pub struct FragmentsOnCompositeTypes;

impl<'a> Visitor<'a> for FragmentsOnCompositeTypes {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        fragment_definition: &'a FragmentDefinition,
    ) {
        if !ctx.current_type().is_composite() {
            let TypeCondition::On(name) = &fragment_definition.type_condition;
            ctx.report_error(
                vec![fragment_definition.position],
                format!(
                    "Fragment \"{}\" cannot condition non composite type \"{}\"",
                    fragment_definition.name, name
                ),
            );
        }
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        inline_fragment: &'a InlineFragment,
    ) {
        if !ctx.current_type().is_composite() {
            ctx.report_error(
                vec![inline_fragment.position],
                format!(
                    "Fragment cannot condition non composite type \"{}\"",
                    ctx.current_type().name()
                ),
            );
        }
    }
}
