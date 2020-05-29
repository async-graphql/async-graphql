use crate::parser::query::VariableDefinition;
use crate::validation::visitor::{Visitor, VisitorContext};
use crate::Positioned;

#[derive(Default)]
pub struct VariablesAreInputTypes;

impl<'a> Visitor<'a> for VariablesAreInputTypes {
    fn enter_variable_definition(
        &mut self,
        ctx: &mut VisitorContext<'a>,
        variable_definition: &'a Positioned<VariableDefinition>,
    ) {
        if let Some(ty) = ctx
            .registry
            .concrete_type_by_parsed_type(&variable_definition.var_type)
        {
            if !ty.is_input() {
                ctx.report_error(
                    vec![variable_definition.position()],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expect_fails_rule, expect_passes_rule};

    pub fn factory() -> VariablesAreInputTypes {
        VariablesAreInputTypes
    }

    #[test]
    fn input_types_are_valid() {
        expect_passes_rule!(
            factory,
            r#"
          query Foo($a: String, $b: [Boolean!]!, $c: ComplexInput) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }

    #[test]
    fn output_types_are_invalid() {
        expect_fails_rule!(
            factory,
            r#"
          query Foo($a: Dog, $b: [[CatOrDog!]]!, $c: Pet) {
            field(a: $a, b: $b, c: $c)
          }
        "#,
        );
    }
}
