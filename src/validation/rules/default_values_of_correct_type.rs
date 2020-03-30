use crate::context::QueryPathNode;
use crate::validation::utils::is_valid_input_value;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::QueryPathSegment;
use graphql_parser::query::{Type, VariableDefinition};

pub struct DefaultValuesOfCorrectType;

impl<'a> Visitor<'a> for DefaultValuesOfCorrectType {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a VariableDefinition,
    ) {
        if let Some(value) = &variable_definition.default_value {
            if let Type::NonNullType(_) = variable_definition.var_type {
                ctx.report_error(vec![variable_definition.position],format!(
                    "Argument \"{}\" has type \"{}\" and is not nullable, so it't can't have a default value",
                    variable_definition.name, variable_definition.var_type,
                ));
            } else if let Some(reason) = is_valid_input_value(
                ctx.registry,
                &variable_definition.var_type.to_string(),
                value,
                QueryPathNode {
                    parent: None,
                    segment: QueryPathSegment::Name(&variable_definition.name),
                },
            ) {
                ctx.report_error(
                    vec![variable_definition.position],
                    format!("Invalid default value for argument {}", reason),
                )
            }
        }
    }
}
