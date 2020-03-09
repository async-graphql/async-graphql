use crate::validation::context::ValidatorContext;
use crate::validation::utils::is_valid_input_value;
use crate::validation::visitor::Visitor;
use graphql_parser::query::{Type, VariableDefinition};

pub struct DefaultValuesOfCorrectType;

impl<'a> Visitor<'a> for DefaultValuesOfCorrectType {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut ValidatorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        if let Some(value) = &variable_definition.default_value {
            if let Type::NonNullType(_) = variable_definition.var_type {
                ctx.report_error(vec![variable_definition.position],format!(
                    "Argument \"#{}\" has type \"{}\" and is not nullable, so it't can't have a default value",
                    variable_definition.name, variable_definition.var_type,
                ));
            } else {
                if !is_valid_input_value(
                    ctx.registry,
                    &variable_definition.var_type.to_string(),
                    value,
                ) {
                    ctx.report_error(
                        vec![variable_definition.position],
                        format!(
                            "Invalid default value for argument \"{}\", expected type \"{}\"",
                            variable_definition.name, variable_definition.var_type
                        ),
                    )
                }
            }
        }
    }
}
