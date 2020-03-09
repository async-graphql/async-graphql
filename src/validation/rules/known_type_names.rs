use crate::registry::TypeName;
use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{
    FragmentDefinition, InlineFragment, TypeCondition, VariableDefinition,
};
use graphql_parser::Pos;

#[derive(Default)]
pub struct KnownTypeNames;

impl<'a> Visitor<'a> for KnownTypeNames {
    fn enter_fragment_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        fragment_definition: &'a FragmentDefinition,
    ) {
        let TypeCondition::On(name) = &fragment_definition.type_condition;
        validate_type(ctx, &name, fragment_definition.position);
    }

    fn enter_variable_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        validate_type(
            ctx,
            TypeName::get_basic_typename(&variable_definition.name),
            variable_definition.position,
        );
    }

    fn enter_inline_fragment(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        inline_fragment: &'a InlineFragment,
    ) {
        if let Some(TypeCondition::On(name)) = &inline_fragment.type_condition {
            validate_type(ctx, &name, inline_fragment.position);
        }
    }
}

fn validate_type(ctx: &mut ValidatorContext<'_>, type_name: &str, pos: Pos) {
    if ctx.registry.types.get(type_name).is_none() {
        ctx.report_error(vec![pos], format!(r#"Unknown type "{}""#, type_name));
    }
}
