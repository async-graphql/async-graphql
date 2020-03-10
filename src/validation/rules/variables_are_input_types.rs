use crate::validation::context::ValidatorContext;
use crate::validation::visitor::Visitor;
use graphql_parser::query::VariableDefinition;

#[derive(Default)]
pub struct VariablesAreInputTypes;

impl<'a> Visitor<'a> for VariablesAreInputTypes {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        if let Some(ty) = ctx
            .registry
            .get_basic_type(&variable_definition.var_type.to_string())
        {
            if !ty.is_input() {
                ctx.report_error(
                    vec![variable_definition.position],
                    format!(
                        "Variable \"{}\" cannot be of non-input type \"{}\"",
                        &variable_definition.name,
                        ty.name()
                    ),
                );
            }
        }
    }
}
